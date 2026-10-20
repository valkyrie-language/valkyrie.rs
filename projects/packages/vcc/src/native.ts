import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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

/** platform 包 short → npm triple（与 `scripts/build.mjs` 对齐）。 */
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

/** N-API 绑定导出的捕获结果。 */
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

/** platform `.node` collect 暴露的正交 Legion 宿主 API（无 argv 聚合 CLI）。 */
export type VccNativeBinding = {
    legionPlan?: (options: { projectDir: string; target?: string }) => LegionPlanOutcome;
    legionBuild?: (options: { projectDir: string; target?: string; outputDir?: string }) => LegionBuildOutcome;
    legionSpyRun?: (mode: string, options: VccSpyTargetRunOptions) => VccNativeLegionOutcome;
};

/**
 * 定位已安装的 VCC native `.node` collect；未安装或未构建时返回 `null`。
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
 */
export function resolveWasmMjs(wasmCollect: string, wasmEntry: string): string {
    const pkgJson = resolveFromRequireRoots(join(wasmCollect, 'package.json')) ?? require.resolve(join(wasmCollect, 'package.json'));
    return join(dirname(pkgJson), wasmEntry);
}

/**
 * 加载 platform `.node` N-API 绑定；未安装或未构建时返回 `null`。
 */
export function loadNativeBindingAt(nativePath: string): VccNativeBinding | null {
    try {
        const binding = require(nativePath) as Partial<VccNativeBinding>;
        const hasProgrammaticApi =
            typeof binding.legionBuild === 'function' ||
            typeof binding.legionPlan === 'function' ||
            typeof binding.legionSpyRun === 'function';
        if (!hasProgrammaticApi) {
            return null;
        }
        return binding as VccNativeBinding;
    } catch {
        return null;
    }
}

/** 加载当前平台 native 绑定。 */
export function loadNativeBinding(nativePackages: readonly string[] = NATIVE_PACKAGES): VccNativeBinding | null {
    const nativePath = locateNativeCollect(nativePackages);
    if (nativePath === null) {
        return null;
    }
    return loadNativeBindingAt(nativePath);
}

/** 程序化 plan（native N-API）。 */
export function planProject(projectDir: string, target?: string, nativePackages?: readonly string[]): LegionPlanOutcome | null {
    const binding = loadNativeBinding(nativePackages);
    if (!binding?.legionPlan) {
        return null;
    }
    return binding.legionPlan({ projectDir, target });
}

/** 程序化 build（native N-API）。 */
export function buildProject(
    projectDir: string,
    target?: string,
    outputDir?: string,
    nativePackages?: readonly string[],
): LegionBuildOutcome | null {
    const binding = loadNativeBinding(nativePackages);
    if (!binding?.legionBuild) {
        return null;
    }
    return binding.legionBuild({ projectDir, target, outputDir });
}

/** 程序化 spy（native N-API）。 */
export function spyRun(
    mode: string,
    options: VccSpyTargetRunOptions = {},
    nativePackages?: readonly string[],
): VccNativeLegionOutcome | null {
    const binding = loadNativeBinding(nativePackages);
    if (!binding?.legionSpyRun) {
        return null;
    }
    return binding.legionSpyRun(mode, options);
}
