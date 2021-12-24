import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

/** 默认 `spawnCli` 超时（毫秒）；超时后终止隔离子进程。 */
export const DEFAULT_SPAWN_TIMEOUT_MS = 30_000;
/** 默认 `spawnCli` stdout/stderr 缓冲上限（字节）。 */
export const DEFAULT_SPAWN_MAX_BUFFER_BYTES = 8 * 1024 * 1024;

const require = createRequire(import.meta.url);

/** 可选 collect 的 `require.resolve` 搜索根（vcc 包 + 调用方 cwd，兼容 path link 本地测试）。 */
const REQUIRE_ROOTS = [fileURLToPath(import.meta.url), join(process.cwd(), 'package.json')];

function resolveFromRequireRoots(subpath: string): string | null {
    for (const root of REQUIRE_ROOTS) {
        try {
            return createRequire(root).resolve(subpath);
        } catch {
            // optional package not visible from this resolution root
        }
    }
    return null;
}

/** 可选 native platform collect 包（`optionalDependencies` 路由）。 */
export const NATIVE_PACKAGES = [
    '@valkyrie-language/vcc-win32-x64',
    '@valkyrie-language/vcc-linux-x64',
    '@valkyrie-language/vcc-darwin-arm64',
    '@valkyrie-language/vcc-darwin-x64',
] as const;

/** platform 包 short → npm triple（与 `scripts/build.mjs` / `@vmz/vmz-*` 对齐）。 */
export const NATIVE_PACKAGE_TRIPLES: Readonly<Record<(typeof NATIVE_PACKAGES)[number], string>> = {
    '@valkyrie-language/vcc-win32-x64': 'win32-x64-msvc',
    '@valkyrie-language/vcc-linux-x64': 'linux-x64-musl',
    '@valkyrie-language/vcc-darwin-x64': 'darwin-x64',
    '@valkyrie-language/vcc-darwin-arm64': 'darwin-arm64',
};

const LEGACY_NATIVE_LIB_NAMES = ['vcc_napi.dll', 'libvcc_napi.so', 'libvcc_napi.dylib', 'vcc.node'] as const;

/** 当前进程的 npm triple（用于 `vcc.${triple}.node` 文件名）。 */
export function resolveNativeNpmTriple(platform = process.platform, arch = process.arch): string {
    if (platform === 'win32' && arch === 'x64') return 'win32-x64-msvc';
    if (platform === 'win32' && arch === 'arm64') return 'win32-arm64-msvc';
    if (platform === 'darwin' && arch === 'arm64') return 'darwin-arm64';
    if (platform === 'darwin' && arch === 'x64') return 'darwin-x64';
    if (platform === 'linux' && arch === 'x64') return 'linux-x64-musl';
    if (platform === 'linux' && arch === 'arm64') return 'linux-arm64-musl';
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
    /** native 路由是否在隔离子进程执行（可超时终止，避免 N-API 挂死拖垮 Node）。默认 `true`。 */
    isolateNative?: boolean;
    /** `spawnCli` 超时（毫秒）。默认 {@link DEFAULT_SPAWN_TIMEOUT_MS}。 */
    spawnTimeoutMs?: number;
    /** `spawnCli` stdout/stderr 缓冲上限（字节）。默认 {@link DEFAULT_SPAWN_MAX_BUFFER_BYTES}。 */
    spawnMaxBufferBytes?: number;
};

/** N-API 绑定导出的 CLI 结果。 */
export type VccNativeLegionOutcome = {
    status: number;
    stdout: string;
    stderr: string;
};

/** `legion spy` 目标参数（与 Rust `SpyTargetRunOptions` 对齐）。 */
export type VccSpyTargetRunOptions = {
    input?: string;
    func?: string;
    method?: string;
    offset?: number;
    list?: boolean;
    context?: number;
    targetPlatform?: string;
    json?: boolean;
    hex?: boolean;
    types?: boolean;
    gcAudit?: boolean;
    glueAudit?: boolean;
};

/** N-API `legionPlan` 结构化结果。 */
export type LegionPlanOutcome = {
    ok: boolean;
    error?: string;
    project?: string;
    target?: string;
    workspaceRoot?: string;
    outputDir?: string;
    sourceCount: number;
    semanticGroupCount: number;
    dependencyCount: number;
    groups: Array<{ dependencyKey: string; fileCount: number }>;
};

/** N-API `legionBuild` 结构化结果。 */
export type LegionBuildOutcome = {
    ok: boolean;
    error?: string;
    project?: string;
    target?: string;
    outputDir?: string;
    artifacts: string[];
    entrySymbol?: string;
};

/** platform `.node` collect 暴露的 Legion 宿主 API。 */
export type VccNativeBinding = {
    /** 程序化 plan（推荐）。 */
    legionPlan?: (options: { projectDir: string; target?: string }) => LegionPlanOutcome;
    /** 程序化 build（推荐）。 */
    legionBuild?: (options: { projectDir: string; target?: string; outputDir?: string }) => LegionBuildOutcome;
    /** @deprecated wasm 回退与旧脚本；新代码请用 `legionPlan` / `legionBuild`。 */
    legionRun?: (argv: string[]) => number;
    /** @deprecated 同上。 */
    legionRunCaptured?: (argv: string[]) => VccNativeLegionOutcome;
    legionSpyRun?: (mode: string, options: VccSpyTargetRunOptions) => VccNativeLegionOutcome;
    legionParse?: (argv: string[]) => string;
};

/** `spawnCli` 路由结果。 */
export type VccCliRoute = 'native' | 'wasm';

/** 可测试的 CLI 子进程结果（不调用 `process.exit`）。 */
export type VccCliSpawnResult = {
    route: VccCliRoute;
    status: number;
    stdout: string;
    stderr: string;
};

/** 将程序化 build 结果转为兼容 `spawnCli` 的形态。 */
export function legionBuildOutcomeToSpawnResult(outcome: LegionBuildOutcome): VccCliSpawnResult {
    return {
        route: 'native',
        status: outcome.ok ? 0 : 1,
        stdout: outcome.ok ? `artifacts: ${outcome.artifacts.join(', ')}` : '',
        stderr: outcome.error ?? '',
    };
}

/** 将程序化 plan 结果转为兼容 `spawnCli` 的形态。 */
export function legionPlanOutcomeToSpawnResult(outcome: LegionPlanOutcome): VccCliSpawnResult {
    const groups = outcome.groups.map((group) => `${group.dependencyKey}=${group.fileCount}`).join(', ');
    return {
        route: 'native',
        status: outcome.ok ? 0 : 1,
        stdout: outcome.ok ? `plan ok: project=${outcome.project} sources=${outcome.sourceCount} groups=${groups}` : '',
        stderr: outcome.error ?? '',
    };
}

/** 由 `createHostRunner` 返回的组装 CLI 宿主。 */
export type VccHostRunner = {
    config: Readonly<Required<VccHostConfig>>;
    locateNativeCollect: () => string | null;
    loadNativeBinding: () => VccNativeBinding | null;
    resolveWasmMjs: () => string;
    /** 程序化 plan（native N-API 优先，wasm 回退 `spawnCli plan`）。 */
    planProject: (projectDir: string, target?: string) => LegionPlanOutcome | VccCliSpawnResult;
    /** 程序化 build（native N-API 优先，wasm 回退 `spawnCli build`）。 */
    buildProject: (projectDir: string, target: string, outputDir?: string) => LegionBuildOutcome | VccCliSpawnResult;
    /** wasm collect 与无原生 API 时的 argv 回退。 */
    spawnCli: (argv?: string[]) => VccCliSpawnResult;
    spawnSpy: (mode: string, options?: VccSpyTargetRunOptions) => VccCliSpawnResult;
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
        const entry = resolveFromRequireRoots(join(name, 'package.json'));
        if (entry === null) {
            continue;
        }
        try {
            const pkgDir = dirname(entry);
            const pkg = JSON.parse(readFileSync(entry, 'utf8')) as { main?: string };
            const candidates = [
                typeof pkg.main === 'string' ? join(pkgDir, pkg.main) : null,
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
            // malformed platform package
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
    const pkgJson = resolveFromRequireRoots(join(wasmCollect, 'package.json')) ?? require.resolve(join(wasmCollect, 'package.json'));
    return join(dirname(pkgJson), wasmEntry);
}

/**
 * 加载 platform `.node` N-API 绑定；未安装或未构建时返回 `null`。
 *
 * @param nativePath
 */
export function loadNativeBindingAt(nativePath: string): VccNativeBinding | null {
    try {
        const binding = require(nativePath) as Partial<VccNativeBinding>;
        const hasProgrammaticApi = typeof binding.legionBuild === 'function' || typeof binding.legionPlan === 'function';
        const hasLegacyCli = typeof binding.legionRunCaptured === 'function';
        if (!hasProgrammaticApi && !hasLegacyCli) {
            return null;
        }
        return binding as VccNativeBinding;
    } catch {
        return null;
    }
}

/**
 * 创建组装 CLI 宿主：优先 native N-API，否则 wasm fallback。
 *
 * @param config
 */
export function createHostRunner(config: VccHostConfig): VccHostRunner {
    const resolved: Required<VccHostConfig> = {
        nativePackages: config.nativePackages ?? NATIVE_PACKAGES,
        wasmCollect: config.wasmCollect,
        wasmEntry: config.wasmEntry,
        isolateNative: config.isolateNative ?? true,
        spawnTimeoutMs: config.spawnTimeoutMs ?? DEFAULT_SPAWN_TIMEOUT_MS,
        spawnMaxBufferBytes: config.spawnMaxBufferBytes ?? DEFAULT_SPAWN_MAX_BUFFER_BYTES,
    };

    const locate = () => locateNativeCollect(resolved.nativePackages);
    const resolveWasm = () => resolveWasmMjs(resolved.wasmCollect, resolved.wasmEntry);

    function loadNativeBinding(): VccNativeBinding | null {
        const nativePath = locate();
        if (nativePath === null) {
            return null;
        }
        return loadNativeBindingAt(nativePath);
    }

    function spawnNativeCliInProcess(argv: string[]): VccCliSpawnResult | null {
        const binding = loadNativeBinding();
        if (binding === null || typeof binding.legionRunCaptured !== 'function') {
            return null;
        }
        const outcome = binding.legionRunCaptured(['legion', ...argv]);
        return {
            route: 'native',
            status: outcome.status,
            stdout: outcome.stdout,
            stderr: outcome.stderr,
        };
    }

    function killProcessTree(pid: number): void {
        if (!Number.isFinite(pid) || pid <= 0) {
            return;
        }
        try {
            if (process.platform === 'win32') {
                execSync(`taskkill /T /F /PID ${pid}`, { stdio: 'ignore' });
            } else {
                process.kill(-pid, 'SIGKILL');
            }
        } catch {
            // child may already be gone
        }
    }

    function spawnNativeCliIsolated(argv: string[], nativePath: string): VccCliSpawnResult {
        const isolatedEntry = fileURLToPath(new URL('./spawn-isolated.mjs', import.meta.url));
        const child = spawnSync(process.execPath, [isolatedEntry, JSON.stringify(argv)], {
            encoding: 'utf8',
            env: { ...process.env, VCC_NATIVE_PATH: nativePath },
            timeout: resolved.spawnTimeoutMs,
            maxBuffer: resolved.spawnMaxBufferBytes,
        });
        if (child.pid) {
            // spawnSync 超时时子进程可能仍存活，强制结束整棵进程树。
            if (child.error?.message.includes('ETIMEDOUT') || child.signal === 'SIGTERM') {
                killProcessTree(child.pid);
            }
        }
        if (child.error?.message.includes('ETIMEDOUT') || child.signal === 'SIGTERM') {
            return {
                route: 'native',
                status: 124,
                stdout: String(child.stdout ?? ''),
                stderr: `${String(child.stderr ?? '')}\nlegion spawnCli timed out after ${resolved.spawnTimeoutMs}ms`,
            };
        }
        const stderr = String(child.stderr ?? '');
        if (child.status !== 0 && !child.stdout) {
            const exitHint = child.signal
                ? `signal ${child.signal}`
                : child.status !== null
                  ? `exit ${child.status} (0x${(child.status >>> 0).toString(16)})`
                  : 'unknown exit';
            return {
                route: 'native',
                status: child.status ?? 1,
                stdout: '',
                stderr: stderr || child.error?.message || `legion isolated spawn failed (${exitHint})`,
            };
        }
        try {
            const outcome = JSON.parse(String(child.stdout ?? '{}')) as VccNativeLegionOutcome;
            return {
                route: 'native',
                status: outcome.status ?? child.status ?? 1,
                stdout: outcome.stdout ?? '',
                stderr: [stderr, outcome.stderr ?? ''].filter(Boolean).join('\n'),
            };
        } catch {
            return {
                route: 'native',
                status: child.status ?? 1,
                stdout: String(child.stdout ?? ''),
                stderr: stderr || 'legion isolated spawn returned invalid JSON',
            };
        }
    }

    function spawnNativeCli(argv: string[]): VccCliSpawnResult | null {
        const nativePath = locate();
        if (nativePath === null) {
            return null;
        }
        if (resolved.isolateNative) {
            return spawnNativeCliIsolated(argv, nativePath);
        }
        return spawnNativeCliInProcess(argv);
    }

    function planProject(projectDir: string, target?: string): LegionPlanOutcome | VccCliSpawnResult {
        const binding = loadNativeBinding();
        if (binding?.legionPlan) {
            return binding.legionPlan({ projectDir, target });
        }
        return spawnCli(['plan', projectDir, ...(target ? ['--target', target] : [])]);
    }

    function buildProject(projectDir: string, target: string, outputDir?: string): LegionBuildOutcome | VccCliSpawnResult {
        const binding = loadNativeBinding();
        if (binding?.legionBuild) {
            return binding.legionBuild({ projectDir, target, outputDir });
        }
        const argv = ['build', projectDir, '--target', target];
        if (outputDir) {
            argv.push('-o', outputDir);
        }
        return spawnCli(argv);
    }

    function spawnCli(argv: string[] = []): VccCliSpawnResult {
        const binding = loadNativeBinding();
        if (binding?.legionPlan && argv[0] === 'plan') {
            const targetFlag = argv.indexOf('--target');
            const target = targetFlag >= 0 ? argv[targetFlag + 1] : undefined;
            return legionPlanOutcomeToSpawnResult(binding.legionPlan({ projectDir: argv[1], target }));
        }
        if (binding?.legionBuild && argv[0] === 'build') {
            const targetFlag = argv.indexOf('--target');
            const outputFlag = argv.findIndex((item) => item === '-o' || item === '--output');
            return legionBuildOutcomeToSpawnResult(
                binding.legionBuild({
                    projectDir: argv[1],
                    target: targetFlag >= 0 ? argv[targetFlag + 1] : undefined,
                    outputDir: outputFlag >= 0 ? argv[outputFlag + 1] : undefined,
                }),
            );
        }
        const native = spawnNativeCli(argv);
        if (native !== null) {
            return native;
        }
        const mjs = resolveWasm();
        const result = spawnSync(process.execPath, [mjs, ...argv], {
            encoding: 'utf8',
            timeout: resolved.spawnTimeoutMs,
            maxBuffer: resolved.spawnMaxBufferBytes,
        });
        if (result.error?.message.includes('ETIMEDOUT') || result.signal === 'SIGTERM') {
            return {
                route: 'wasm',
                status: 124,
                stdout: String(result.stdout ?? ''),
                stderr: `${String(result.stderr ?? '')}\nlegion spawnCli timed out after ${resolved.spawnTimeoutMs}ms`,
            };
        }
        return {
            route: 'wasm',
            status: result.status ?? 1,
            stdout: String(result.stdout ?? ''),
            stderr: String(result.stderr ?? ''),
        };
    }

    function spawnSpy(mode: string, options: VccSpyTargetRunOptions = {}): VccCliSpawnResult {
        const binding = loadNativeBinding();
        if (binding?.legionSpyRun) {
            const outcome = binding.legionSpyRun(mode, options);
            return {
                route: 'native',
                status: outcome.status,
                stdout: outcome.stdout,
                stderr: outcome.stderr,
            };
        }
        const argv = ['spy', mode];
        if (options.input) {
            argv.push(options.input);
        }
        if (options.func) {
            argv.push('--func', options.func);
        }
        if (options.method) {
            argv.push('--method', options.method);
        }
        if (options.offset !== undefined) {
            argv.push('--offset', String(options.offset));
        }
        if (options.list) {
            argv.push('--list');
        }
        if (options.context !== undefined) {
            argv.push('--context', String(options.context));
        }
        if (options.targetPlatform) {
            argv.push('--target', options.targetPlatform);
        }
        if (options.json) {
            argv.push('--json');
        }
        if (options.hex) {
            argv.push('--hex');
        }
        if (options.types) {
            argv.push('--types');
        }
        if (options.gcAudit) {
            argv.push('--gc-audit');
        }
        if (options.glueAudit) {
            argv.push('--glue-audit');
        }
        return spawnCli(argv);
    }

    function runCli(argv: string[] = process.argv.slice(2)): never {
        const binding = loadNativeBinding();
        if (binding?.legionRun) {
            process.exit(binding.legionRun(['legion', ...argv]));
        }
        const outcome = spawnCli(argv);
        process.exit(outcome.status);
    }

    return {
        config: resolved,
        locateNativeCollect: locate,
        loadNativeBinding,
        resolveWasmMjs: resolveWasm,
        planProject,
        buildProject,
        spawnCli,
        spawnSpy,
        runCli,
    };
}
