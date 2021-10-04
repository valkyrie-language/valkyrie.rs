#!/usr/bin/env node
/**
 * npm publish & Trusted Publisher
 *
 *   node scripts/publish.mjs npm [--dry-run] [--skip legion] [--only vcc]
 *   node scripts/publish.mjs ci --version=0.0.1
 *   node scripts/publish.mjs trust status|configure [--only @valkyrie-language/vcc]
 *
 * Auth: repo-root `.env.npm-trust.local` (NPM_TOTP_SECRET / NPM_TOKEN)
 */

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ENV_PATH, loadLocalEnv, resolveNpmAuth, runNpm, totpCode } from "./lib/npm-auth.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGES_ROOT = path.join(ROOT, "projects", "packages");
const CACHE_PATH = path.join(path.dirname(fileURLToPath(import.meta.url)), "lib", ".npm-trust-cache.json");
const MIN_WASM_BYTES = 1024;

const PUBLISH_ORDER = [
    "vcc",
    "vcc-win32-x64",
    "vcc-linux-x64",
    "vcc-darwin-x64",
    "vcc-darwin-arm64",
    "vcc-unknown-wasm32",
    "vcc-wasm32-wasi",
    "legion",
    "asgard",
];

const TRUST_PACKAGES = PUBLISH_ORDER.map((d) => {
    const pkg = JSON.parse(fs.readFileSync(path.join(PACKAGES_ROOT, d, "package.json"), "utf8"));
    return pkg.name;
});

const TRUST = {
    repo: "valkyrie-language/valkyrie.rs",
    file: "publish-npm.yml",
    env: "NPM_PUBLISH",
};

const argv = process.argv.slice(2);
const command = argv[0] ?? "npm";
const rest = argv.slice(1);

function fail(msg) {
    console.error(`publish: ${msg}`);
    process.exit(1);
}

function takeFlag(args, flag) {
    const i = args.indexOf(flag);
    if (i >= 0 && args[i + 1] && !args[i + 1].startsWith("-")) return args[i + 1];
    const eq = args.find((a) => a.startsWith(`${flag}=`));
    if (eq) return eq.slice(flag.length + 1);
    return undefined;
}

function readJson(p) {
    return JSON.parse(fs.readFileSync(p, "utf8"));
}

function writeJson(p, obj) {
    fs.writeFileSync(p, `${JSON.stringify(obj, null, 2)}\n`);
}

function copyTree(src, dest, skipTests = true) {
    fs.mkdirSync(dest, { recursive: true });
    for (const name of fs.readdirSync(src)) {
        if (name === "node_modules" || name === ".git" || (skipTests && name === "tests")) continue;
        const from = path.join(src, name);
        const to = path.join(dest, name);
        const st = fs.statSync(from);
        if (st.isDirectory()) copyTree(from, to, skipTests);
        else {
            fs.mkdirSync(path.dirname(to), { recursive: true });
            fs.copyFileSync(from, to);
        }
    }
}

function rewriteDeps(value, version) {
    if (Array.isArray(value)) return value.map((item) => rewriteDeps(item, version));
    if (value && typeof value === "object") {
        const out = {};
        for (const [key, item] of Object.entries(value)) out[key] = rewriteDeps(item, version);
        return out;
    }
    if (typeof value === "string" && value === "workspace:*") return version;
    return value;
}

function resolveVersion(args) {
    const fromArg = args.find((a) => a.startsWith("--version="))?.slice("--version=".length);
    if (fromArg) return fromArg.replace(/^v/, "");
    const vcc = readJson(path.join(PACKAGES_ROOT, "vcc", "package.json"));
    return String(vcc.version).replace(/^v/, "");
}

function distTag(version) {
    return version === "0.0.0" ? "seed" : "latest";
}

function resolvePublishList(args) {
    const only = takeFlag(args, "--only");
    const skipRaw = takeFlag(args, "--skip");
    const skip = new Set(
        (skipRaw ? skipRaw.split(",") : [])
            .concat(args.filter((a) => a.startsWith("--skip=")).map((a) => a.slice("--skip=".length)))
            .flatMap((s) => s.split(","))
            .map((s) => s.trim())
            .filter(Boolean),
    );
    if (only) {
        const names = only
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean);
        for (const name of names) {
            if (!PUBLISH_ORDER.includes(name)) fail(`--only ${name} not in publish set`);
        }
        return names;
    }
    return PUBLISH_ORDER.filter((name) => !skip.has(name));
}

function cmdNpm() {
    const dryRun = rest.includes("--dry-run");
    const auth = resolveNpmAuth(rest);
    const version = resolveVersion(rest);
    const publishList = resolvePublishList(rest);

    if (!dryRun && !auth.totpSecretRaw && !auth.token && !auth.currentOtp()) {
        console.warn(`publish npm: no NPM_TOTP_SECRET in ${ENV_PATH} — may EOTP`);
    } else if (!dryRun) {
        console.log(`publish npm: totp=${auth.totpSecretRaw ? "yes" : "no"} token=${auth.token ? "yes" : "no"}`);
    }

    console.log(`publish npm: ${dryRun ? "dry-run" : "live"} ${publishList.length} packages @ ${version}`);

    let published = 0;
    let skipped = 0;
    for (const dirName of publishList) {
        const abs = path.join(PACKAGES_ROOT, dirName);
        const raw = readJson(path.join(abs, "package.json"));
        const name = raw.name;
        const stage = path.join(os.tmpdir(), `valkyrie-publish-${name.replace("/", "-")}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        copyTree(abs, stage);
        const pkg = rewriteDeps(raw, version);
        pkg.version = version;
        pkg.publishConfig = { ...(raw.publishConfig ?? {}), access: "public" };
        delete pkg.private;
        delete pkg.devDependencies;
        writeJson(path.join(stage, "package.json"), pkg);

        const view = runNpm(["view", `${name}@${version}`, "version"], { token: auth.token });
        if (!dryRun && view.status === 0 && view.stdout === version) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }

        const args = ["publish", "--access", "public", "--tag", distTag(version)];
        if (dryRun) args.push("--dry-run");
        const otp = auth.currentOtp();
        if (otp) args.push(`--otp=${otp}`);
        console.log(`\n=== ${name}@${version} ===`);
        const r = runNpm(args, { cwd: stage, token: auth.token });
        if (r.stdout) process.stdout.write(`${r.stdout}\n`);
        if (r.stderr) process.stderr.write(`${r.stderr}\n`);
        if (r.status !== 0) {
            const blob = `${r.stdout}\n${r.stderr}`;
            if (/already been published|cannot publish over|EPUBLISHCONFLICT/i.test(blob)) {
                console.log(` ✓ ${name}@${version} already on registry — skip`);
                skipped += 1;
                continue;
            }
            fail(`publish failed for ${name}@${version}`);
        }
        published += dryRun ? 0 : 1;
    }
    console.log(`\npublish npm: done (published=${published} skipped=${skipped})`);
}

function assertVccAssembled(abs) {
    for (const f of ["legion.wasm", "legion.mjs", "run-contracts.txt", "provenance.json"]) {
        if (!fs.existsSync(path.join(abs, f))) fail(`missing assembled file ${f}`);
    }
    if (fs.statSync(path.join(abs, "legion.wasm")).size < MIN_WASM_BYTES) {
        fail("legion.wasm too small");
    }
    const contracts = fs.readFileSync(path.join(abs, "run-contracts.txt"), "utf8");
    if (/placeholder-minimal-wasm/i.test(contracts)) fail("run-contracts.txt is placeholder");
    const provenance = readJson(path.join(abs, "provenance.json"));
    if (provenance?.build?.fixture === true) fail("provenance.build.fixture is true");
    if (/smoke-out|smoke-legion|smoke-/i.test(String(provenance?.build?.from ?? ""))) {
        fail("provenance.build.from looks like fixture path");
    }
}

function stagePackage(dirName, version, includeTests) {
    const abs = path.join(PACKAGES_ROOT, dirName);
    const raw = readJson(path.join(abs, "package.json"));
    const name = raw.name;
    const stage = path.join(os.tmpdir(), `valkyrie-publish-${name.replace("/", "-")}-${version}`);
    fs.rmSync(stage, { recursive: true, force: true });
    copyTree(abs, stage, !includeTests);
    const pkg = rewriteDeps(raw, version);
    pkg.version = version;
    pkg.publishConfig = { ...(raw.publishConfig ?? {}), access: "public" };
    delete pkg.private;
    delete pkg.devDependencies;
    writeJson(path.join(stage, "package.json"), pkg);
    return { name, stage };
}

function cmdCi() {
    const version = resolveVersion(rest);
    const publishList = resolvePublishList(rest);
    const tag = distTag(version);

    assertVccAssembled(path.join(ROOT, "projects/packages/vcc-unknown-wasm32"));

    delete process.env.NODE_AUTH_TOKEN;
    delete process.env.NPM_TOKEN;

    console.log(`publish ci: OIDC ${publishList.length} packages @ ${version} (tag=${tag})`);

    let published = 0;
    let skipped = 0;
    for (const dirName of publishList) {
        const includeTests = dirName === "legion";
        const { name, stage } = stagePackage(dirName, version, includeTests);

        const exists = runNpm(["view", `${name}@${version}`, "version"]);
        if (exists.status === 0 && exists.stdout === version) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }

        console.log(`\n=== ${name}@${version} (OIDC) ===`);
        const r = runNpm(["publish", "--access", "public", "--tag", tag], { cwd: stage });
        if (r.stdout) process.stdout.write(`${r.stdout}\n`);
        if (r.stderr) process.stderr.write(`${r.stderr}\n`);
        if (r.status !== 0) {
            const blob = `${r.stdout}\n${r.stderr}`;
            if (/already been published|cannot publish over|EPUBLISHCONFLICT/i.test(blob)) {
                console.log(` ✓ ${name}@${version} already on registry — skip`);
                skipped += 1;
                continue;
            }
            if (/ENEEDAUTH|OIDC|trusted publisher/i.test(blob)) {
                fail("OIDC failed — Trusted Publisher: publish-npm.yml + NPM_PUBLISH");
            }
            fail(`ci publish failed for ${name}@${version}`);
        }
        published += 1;
    }
    console.log(`\npublish ci: done (published=${published} skipped=${skipped})`);
}

function trustAuth() {
    const localEnv = loadLocalEnv();
    const token = takeFlag(rest, "--token") ?? process.env.NPM_TOKEN ?? localEnv.NPM_TOKEN;
    const otpFlag = takeFlag(rest, "--otp") ?? process.env.NPM_OTP ?? localEnv.NPM_OTP;
    const totpSecretRaw =
        takeFlag(rest, "--totp-secret") ??
        process.env.NPM_TOTP_SECRET ??
        localEnv.NPM_TOTP_SECRET ??
        (otpFlag && !/^\d{6}$/.test(otpFlag.trim()) ? otpFlag : undefined);
    const otpStatic = otpFlag && /^\d{6}$/.test(otpFlag.trim()) ? otpFlag.trim() : undefined;
    return {
        token,
        currentOtp() {
            if (totpSecretRaw) return totpCode(totpSecretRaw);
            return otpStatic;
        },
        hasOtp: Boolean(totpSecretRaw || otpStatic),
    };
}

function trustFields(cfg) {
    const claims = cfg?.claims ?? {};
    return {
        repo: cfg?.repository ?? claims.repository ?? claims.repo ?? "",
        file: cfg?.file ?? claims.workflow_ref?.file ?? claims.file ?? "",
        env: cfg?.environment ?? claims.environment ?? claims.env ?? "",
    };
}

function trustMatches(cfg) {
    if (cfg?.raw && typeof cfg.raw === "string") {
        return cfg.raw.includes(TRUST.repo) && cfg.raw.includes(TRUST.file);
    }
    const { repo, file, env } = trustFields(cfg);
    return repo === TRUST.repo && file === TRUST.file && (env === TRUST.env || env === "");
}

function classifyConfigs(configs) {
    if (configs.find((c) => trustMatches(c) && trustFields(c).env === TRUST.env)) {
        return { matches: true, matchKind: "exact" };
    }
    if (configs.find(trustMatches)) return { matches: true, matchKind: "loose" };
    if (configs.length === 0) return { matches: false, matchKind: "none" };
    return { matches: false, matchKind: "mismatch" };
}

function listTrustLive(name, auth) {
    const args = ["trust", "list", name, "--json"];
    const code = auth.currentOtp();
    if (code) args.push(`--otp=${code}`);
    const r = runNpm(args, { token: auth.token });
    if (/EOTP|one-time password/i.test(`${r.stdout}\n${r.stderr}`)) {
        return { configs: [], authRequired: true };
    }
    if (r.status !== 0) return { configs: [], error: r.stderr || r.stdout };
    try {
        const data = JSON.parse(r.stdout || "[]");
        if (Array.isArray(data)) return { configs: data };
        if (Array.isArray(data?.configurations)) return { configs: data.configurations };
        if (data?.type || data?.claims) return { configs: [data] };
        return { configs: [] };
    } catch {
        return { configs: [] };
    }
}

function trustTargets() {
    const only = takeFlag(rest, "--only");
    if (!only) return TRUST_PACKAGES;
    if (!TRUST_PACKAGES.includes(only)) fail(`--only ${only} not in package set`);
    return [only];
}

function cmdTrustStatus() {
    const auth = trustAuth();
    const cache = fs.existsSync(CACHE_PATH) ? readJson(CACHE_PATH) : { packages: {} };
    console.log("publish trust: status\n");
    let ok = 0;
    let bad = 0;
    for (const name of trustTargets()) {
        const ver = runNpm(["view", name, "version"], { token: auth.token }).stdout;
        if (!ver) {
            console.log(`  ? ${name}  not on registry`);
            bad += 1;
            continue;
        }
        const entry = auth.hasOtp
            ? (() => {
                  const live = listTrustLive(name, auth);
                  if (live.configs) {
                      cache.packages[name] = { ...classifyConfigs(live.configs), listedAt: new Date().toISOString() };
                      writeJson(CACHE_PATH, cache);
                  }
                  return cache.packages[name];
              })()
            : cache.packages[name];
        if (entry?.matches) {
            console.log(`  ok ${name}@${ver}`);
            ok += 1;
        } else {
            console.log(`  ~ ${name}@${ver}  trust missing`);
            bad += 1;
        }
    }
    console.log(`\n${ok} ok, ${bad} need configure`);
    process.exit(bad > 0 ? 1 : 0);
}

function cmdTrustConfigure() {
    const auth = trustAuth();
    if (!auth.hasOtp) fail(`need NPM_TOTP_SECRET in ${ENV_PATH}`);
    const dryRun = rest.includes("--dry-run");
    let configured = 0;
    let skipped = 0;
    for (const name of trustTargets()) {
        if (!runNpm(["view", name, "version"], { token: auth.token }).stdout) {
            console.log(`  skip ${name} (not on registry)`);
            continue;
        }
        const live = listTrustLive(name, auth);
        if (live.authRequired) fail(`EOTP listing ${name}`);
        const entry = classifyConfigs(live.configs ?? []);
        if (entry.matches) {
            console.log(`  skip ${name} (already configured)`);
            skipped += 1;
            continue;
        }
        if (entry.matchKind === "mismatch") fail(`${name}: trust mismatch — revoke manually`);
        const code = auth.currentOtp();
        const args = [
            "trust",
            "github",
            name,
            `--file=${TRUST.file}`,
            `--repo=${TRUST.repo}`,
            `--env=${TRUST.env}`,
            "--allow-publish",
            "--allow-stage-publish",
            "--yes",
            `--otp=${code}`,
        ];
        if (dryRun) {
            console.log(`  dry-run: npm ${args.join(" ")}`);
            continue;
        }
        console.log(`\n=== ${name} ===`);
        const r = runNpm(args, { token: auth.token });
        if (r.stdout) process.stdout.write(String(r.stdout));
        if (r.status !== 0) fail(`trust create failed: ${name}`);
        configured += 1;
    }
    console.log(`\npublish trust: done (configured=${configured} skipped=${skipped})`);
}

switch (command) {
    case "npm":
        cmdNpm();
        break;
    case "ci":
        cmdCi();
        break;
    case "trust": {
        const sub = rest.find((a) => !a.startsWith("-")) ?? "status";
        if (sub === "status" || sub === "check") cmdTrustStatus();
        else if (sub === "configure" || sub === "trust") cmdTrustConfigure();
        else fail(`unknown trust subcommand \`${sub}\``);
        break;
    }
    default:
        fail(`unknown command \`${command}\`. Use: npm | ci | trust`);
}
