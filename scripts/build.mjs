#!/usr/bin/env node
/**
 * VCC platform collect build & wasm assemble.
 *
 *   node scripts/build.mjs                 # napi + wasm
 *   node scripts/build.mjs napi|wasm
 *   node scripts/build.mjs assemble --from <dir> --source-project <dir>
 *   node scripts/build.mjs capability --valkyrie-v valkyrie.v
 */

import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { runAssemble } from "./lib/assemble.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGES_ROOT = join(ROOT, "projects", "packages");

/** @typedef {{ triple: string, packageDir: string, tool: "cargo" | "zigbuild" }} NativeTarget */
/** @typedef {{ triple: string, packageDir: string, libName: string, destName: string }} WasmTarget */

const NATIVE_TARGETS = [
    { triple: "x86_64-pc-windows-msvc", packageDir: "vcc-win32-x64", tool: "cargo" },
    { triple: "x86_64-unknown-linux-musl", packageDir: "vcc-linux-x64", tool: "zigbuild" },
    { triple: "x86_64-apple-darwin", packageDir: "vcc-darwin-x64", tool: "zigbuild" },
    { triple: "aarch64-apple-darwin", packageDir: "vcc-darwin-arm64", tool: "zigbuild" },
];

const WASM_TARGETS = [
    {
        triple: "wasm32-wasip1",
        packageDir: "vcc-wasm32-wasi",
        libName: "vcc_wasm",
        destName: "vcc_wasm.wasm",
    },
];

const MODES = new Set(["napi", "wasm", "assemble", "capability"]);

/**
 * @param {string[]} argv
 */
function parseArgs(argv) {
    const flags = argv.filter((arg) => arg.startsWith("-"));
    const positionals = argv.filter((arg) => !arg.startsWith("-"));
    const mode = positionals.find((arg) => MODES.has(arg)) ?? "all";
    return {
        mode,
        all: flags.includes("--all"),
        debug: flags.includes("--debug"),
        rest: argv,
    };
}

function takeFlag(args, flag) {
    const i = args.indexOf(flag);
    if (i >= 0 && args[i + 1] && !args[i + 1].startsWith("-")) return args[i + 1];
    const eq = args.find((a) => a.startsWith(`${flag}=`));
    if (eq) return eq.slice(flag.length + 1);
    return undefined;
}

function gitRev(cwd) {
    const r = spawnSync("git", ["rev-parse", "HEAD"], { cwd, encoding: "utf8" });
    return r.status === 0 ? String(r.stdout).trim() : null;
}

function findToolsProject(valkyrieV) {
    const candidates = [
        join(valkyrieV, "projects/legion._/projects/legion.tools"),
        join(valkyrieV, "projects/legion.tools"),
    ];
    for (const p of candidates) {
        if (existsSync(join(p, "legion.von"))) return p;
    }
    return null;
}

function cmdCapability(argv) {
    const valkyrieV = resolve(ROOT, takeFlag(argv, "--valkyrie-v") ?? "valkyrie.v");
    if (!existsSync(valkyrieV)) fail(`--valkyrie-v not found: ${valkyrieV}`);
    const tools = findToolsProject(valkyrieV);
    if (!tools) fail(`legion.tools project not found under ${valkyrieV}`);

    const outRoot = join(ROOT, "dist", "legion-node-capability");
    mkdirSync(outRoot, { recursive: true });
    console.log("build capability: cargo test -p legion assemble_vcc_unknown_wasm32_capability (library build::run, no native bin)");
    run(
        "cargo",
        ["test", "-p", "legion", "--release", "assemble_vcc_unknown_wasm32_capability", "--", "--exact", "--nocapture"],
        {
            VALKYRIE_V: valkyrieV,
            LEGION_CAPABILITY_OUT: outRoot,
        },
    );

    const nested = join(outRoot, "wasm32-node-unknown-wasm");
    const artifactDir = existsSync(join(nested, "legion.wasm")) ? nested : outRoot;
    runAssemble([
        "assemble",
        "--from",
        artifactDir,
        "--source-project",
        tools,
        ...(gitRev(valkyrieV) ? ["--v-commit", gitRev(valkyrieV)] : []),
    ]);
}

function fail(msg) {
    console.error(`build: ${msg}`);
    process.exit(1);
}

/**
 * @param {string} command
 * @param {string[]} args
 * @param {Record<string, string>} [envExtra]
 */
function run(command, args, envExtra) {
    const result = spawnSync(command, args, {
        cwd: ROOT,
        stdio: "inherit",
        shell: process.platform === "win32",
        env: envExtra ? { ...process.env, ...envExtra } : process.env,
    });
    if ((result.status ?? 1) !== 0) {
        process.exit(result.status ?? 1);
    }
}

function hostTriple() {
    if (process.platform === "win32") {
        return "x86_64-pc-windows-msvc";
    }
    if (process.platform === "darwin") {
        return process.arch === "arm64" ? "aarch64-apple-darwin" : "x86_64-apple-darwin";
    }
    return "x86_64-unknown-linux-gnu";
}

/**
 * @param {NativeTarget[]} targets
 * @param {boolean} all
 */
function selectNativeTargets(targets, all) {
    if (all) {
        return targets;
    }
    const host = hostTriple();
    const exact = targets.find((target) => target.triple === host);
    if (exact) {
        return [exact];
    }
    if (process.platform === "linux") {
        const linux = targets.find((target) => target.packageDir === "vcc-linux-x64");
        if (linux) {
            return [{ ...linux, triple: host }];
        }
    }
    return targets.slice(0, 1);
}

/**
 * @param {string} triple
 * @param {boolean} release
 * @param {"cargo" | "zigbuild"} tool
 * @param {string} crate
 */
function cargoBuildTarget(triple, release, tool, crate) {
    const profile = release ? ["--release"] : [];
    if (tool === "zigbuild") {
        run("cargo", ["zigbuild", "build", ...profile, "--target", triple, "-p", crate]);
        return;
    }
    run("cargo", ["build", ...profile, "--target", triple, "-p", crate]);
}

/**
 * @param {string} triple
 * @param {boolean} release
 * @param {string} libName
 */
function nativeLibCandidates(triple, release, libName) {
    const profile = release ? "release" : "debug";
    const dir = join(ROOT, "target", triple, profile);
    if (triple.includes("windows")) {
        return [join(dir, `${libName}.dll`)];
    }
    if (triple.includes("apple")) {
        return [join(dir, `lib${libName}.dylib`)];
    }
    return [join(dir, `lib${libName}.so`), join(dir, `${libName}.so`)];
}

/**
 * @param {string} triple
 * @param {boolean} release
 * @param {string} libName
 */
function wasmLibCandidates(triple, release, libName) {
    const profile = release ? "release" : "debug";
    const dir = join(ROOT, "target", triple, profile);
    return [join(dir, `${libName}.wasm`)];
}

/**
 * @param {string[]} candidates
 */
function resolveArtifact(candidates) {
    for (const candidate of candidates) {
        if (existsSync(candidate)) {
            return candidate;
        }
    }
    throw new Error(`missing build artifact (tried: ${candidates.join(", ")})`);
}

/**
 * @param {string} src
 * @param {string} dest
 */
function copyArtifact(src, dest) {
    mkdirSync(dirname(dest), { recursive: true });
    copyFileSync(src, dest);
}

/**
 * @param {string} triple
 * @param {string} libName
 */
function nativeCollectDestName(triple, libName) {
    if (triple.includes("windows")) {
        return `${libName}.dll`;
    }
    if (triple.includes("apple")) {
        return `lib${libName}.dylib`;
    }
    return `lib${libName}.so`;
}

/**
 * @param {{ all: boolean, debug: boolean }} opts
 */
function buildNapi(opts) {
    const release = !opts.debug;
    const targets = selectNativeTargets(NATIVE_TARGETS, opts.all);
    console.log(`build:napi → vcc-napi cdylib (${release ? "release" : "debug"}, ${targets.length} target(s))`);

    for (const target of targets) {
        console.log(`\n→ ${target.triple} → projects/packages/${target.packageDir}`);
        cargoBuildTarget(target.triple, release, target.tool, "vcc-napi");

        const src = resolveArtifact(nativeLibCandidates(target.triple, release, "vcc_napi"));
        const destName = nativeCollectDestName(target.triple, "vcc_napi");
        const dest = join(PACKAGES_ROOT, target.packageDir, destName);
        copyArtifact(src, dest);
        console.log(`  copied ${destName}`);
    }
}

/**
 * @param {string} packageDir
 * @param {string} wasmPath
 */
function stageAsgardWasmCollect(packageDir, wasmPath) {
    const pkgRoot = join(PACKAGES_ROOT, packageDir);
    const asgardWasm = join(pkgRoot, "asgard.wasm");
    const asgardMjs = join(pkgRoot, "asgard.mjs");

    copyArtifact(wasmPath, asgardWasm);

    const bootstrap = `#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const wasmBytes = readFileSync(join(here, "asgard.wasm"));

async function main() {
    const { WASI, instantiate } = await import("node:wasi");
    const wasi = new WASI({ args: process.argv, env: process.env, stdout: process.stdout, stderr: process.stderr });
    const { instance } = await WebAssembly.instantiate(wasmBytes, {
        ...wasi.getImportObject(),
    });
    wasi.initialize(instance);
    if (typeof instance.exports._start === "function") {
        instance.exports._start();
        return;
    }
    if (typeof instance.exports.main === "function") {
        process.exit(Number(instance.exports.main() ?? 0));
    }
    throw new Error("asgard.wasm has no _start or main export");
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
`;

    writeFileSync(asgardMjs, bootstrap, "utf8");
    console.log("  staged asgard.mjs + asgard.wasm");
}

/**
 * @param {{ all: boolean, debug: boolean }} opts
 */
function buildWasm(opts) {
    const release = !opts.debug;
    const targets = opts.all ? WASM_TARGETS : WASM_TARGETS;
    console.log(`build:wasm → vcc-wasm cdylib (${release ? "release" : "debug"}, ${targets.length} target(s))`);

    for (const target of targets) {
        console.log(`\n→ ${target.triple} → projects/packages/${target.packageDir}`);
        cargoBuildTarget(target.triple, release, "cargo", "vcc-wasm");

        const src = resolveArtifact(wasmLibCandidates(target.triple, release, target.libName));
        const dest = join(PACKAGES_ROOT, target.packageDir, target.destName);
        copyArtifact(src, dest);
        console.log(`  copied ${target.destName}`);
        stageAsgardWasmCollect(target.packageDir, src);
    }
}

const opts = parseArgs(process.argv.slice(2));

if (opts.mode === "assemble") {
    runAssemble(opts.rest);
} else if (opts.mode === "capability") {
    cmdCapability(opts.rest);
} else {
    if (opts.mode === "napi") {
        buildNapi(opts);
    } else if (opts.mode === "wasm") {
        buildWasm(opts);
    } else {
        buildNapi(opts);
        buildWasm(opts);
    }
    console.log("\nbuild complete");
}
