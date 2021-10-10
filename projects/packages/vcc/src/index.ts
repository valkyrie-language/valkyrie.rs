import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

const require = createRequire(import.meta.url);

/** 可选 native platform collect 包（`optionalDependencies` 路由）。 */
export const NATIVE_PACKAGES = [
    "@valkyrie-language/vcc-win32-x64",
    "@valkyrie-language/vcc-linux-x64",
    "@valkyrie-language/vcc-darwin-arm64",
    "@valkyrie-language/vcc-darwin-x64",
] as const;

/** platform 包 short → npm triple（与 `scripts/build.mjs` / `@vmz/vmz-*` 对齐）。 */
export const NATIVE_PACKAGE_TRIPLES: Readonly<Record<(typeof NATIVE_PACKAGES)[number], string>> = {
    "@valkyrie-language/vcc-win32-x64": "win32-x64-msvc",
    "@valkyrie-language/vcc-linux-x64": "linux-x64-musl",
    "@valkyrie-language/vcc-darwin-x64": "darwin-x64",
    "@valkyrie-language/vcc-darwin-arm64": "darwin-arm64",
};

const LEGACY_NATIVE_LIB_NAMES = ["vcc_napi.dll", "libvcc_napi.so", "libvcc_napi.dylib", "vcc.node"] as const;

/** 当前进程的 npm triple（用于 `vcc.${triple}.node` 文件名）。 */
export function resolveNativeNpmTriple(platform = process.platform, arch = process.arch): string {
    if (platform === "win32" && arch === "x64") return "win32-x64-msvc";
    if (platform === "win32" && arch === "arm64") return "win32-arm64-msvc";
    if (platform === "darwin" && arch === "arm64") return "darwin-arm64";
    if (platform === "darwin" && arch === "x64") return "darwin-x64";
    if (platform === "linux" && arch === "x64") return "linux-x64-musl";
    if (platform === "linux" && arch === "arm64") return "linux-arm64-musl";
    return `${platform}-${arch}`;
}

/** platform-named N-API 二进制文件名。 */
export function nativeCollectBinaryName(npmTriple: string): string {
    return `vcc.${npmTriple}.node`;
}

/** 组装 CLI 宿主配置：native cdylib + wasm collect 入口。 */
export type VccHostConfig = {
    wasmCollect: string;
    wasmEntry: string;
    nativePackages?: readonly string[];
};

/** `spawnCli` 路由结果。 */
export type VccCliRoute = "native" | "wasm";

/** 可测试的 CLI 子进程结果（不调用 `process.exit`）。 */
export type VccCliSpawnResult = {
    route: VccCliRoute;
    status: number;
    stdout: string;
    stderr: string;
};

/** 由 `createHostRunner` 返回的组装 CLI 宿主。 */
export type VccHostRunner = {
    config: Readonly<Required<VccHostConfig>>;
    locateNativeCollect: () => string | null;
    resolveWasmMjs: () => string;
    spawnCli: (argv?: string[]) => VccCliSpawnResult;
    runCli: (argv?: string[]) => never;
};

/**
 * 定位已安装的 VCC native `.node` collect；未安装或未构建时返回 `null`。
 *
 * @param nativePackages
 */
export function locateNativeCollect(nativePackages: readonly string[] = NATIVE_PACKAGES): string | null {
    const hostTriple = resolveNativeNpmTriple();
    for (const name of nativePackages) {
        try {
            const entry = require.resolve(join(name, "package.json"));
            const pkgDir = dirname(entry);
            const pkg = JSON.parse(readFileSync(entry, "utf8")) as { main?: string };
            const candidates = [
                typeof pkg.main === "string" ? join(pkgDir, pkg.main) : null,
                join(pkgDir, nativeCollectBinaryName(NATIVE_PACKAGE_TRIPLES[name as (typeof NATIVE_PACKAGES)[number]] ?? hostTriple)),
                join(pkgDir, nativeCollectBinaryName(hostTriple)),
                ...LEGACY_NATIVE_LIB_NAMES.map((file) => join(pkgDir, file)),
            ].filter((value): value is string => Boolean(value));
            for (const candidate of candidates) {
                if (existsSync(candidate)) {
                    return candidate;
                }
            }
        } catch {
            // optional platform package not installed
        }
    }
    return null;
}

/**
 * 解析 Wasm collect 入口脚本的绝对路径。
 *
 * @param wasmCollect
 * @param wasmEntry
 */
export function resolveWasmMjs(wasmCollect: string, wasmEntry: string): string {
    const pkgJson = require.resolve(join(wasmCollect, "package.json"));
    return join(dirname(pkgJson), wasmEntry);
}

/**
 * 创建组装 CLI 宿主：优先 native cdylib（待 N-API 接线），否则 wasm fallback。
 *
 * @param config
 */
export function createHostRunner(config: VccHostConfig): VccHostRunner {
    const resolved: Required<VccHostConfig> = {
        nativePackages: config.nativePackages ?? NATIVE_PACKAGES,
        wasmCollect: config.wasmCollect,
        wasmEntry: config.wasmEntry,
    };

    const locate = () => locateNativeCollect(resolved.nativePackages);
    const resolveWasm = () => resolveWasmMjs(resolved.wasmCollect, resolved.wasmEntry);

    function tryNative(_argv: string[]): number | null {
        if (locate() === null) {
            return null;
        }
        // TODO: `#[napi]` 导出落地后在此 `require()` platform `.node` 并 dispatch 到对应 Rust CLI。
        return null;
    }

    function spawnCli(argv: string[] = []): VccCliSpawnResult {
        const native = tryNative(argv);
        if (native !== null) {
            return { route: "native", status: native, stdout: "", stderr: "" };
        }
        const mjs = resolveWasm();
        const result = spawnSync(process.execPath, [mjs, ...argv], { encoding: "utf8" });
        return {
            route: "wasm",
            status: result.status ?? 1,
            stdout: String(result.stdout ?? ""),
            stderr: String(result.stderr ?? ""),
        };
    }

    function runCli(argv: string[] = process.argv.slice(2)): never {
        const outcome = spawnCli(argv);
        process.exit(outcome.status);
    }

    return {
        config: resolved,
        locateNativeCollect: locate,
        resolveWasmMjs: resolveWasm,
        spawnCli,
        runCli,
    };
}
