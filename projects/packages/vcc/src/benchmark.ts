import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

import type { VccCliRoute, VccCliSpawnResult, VccHostRunner } from './index.ts';
import { createHostRunner, locateNativeCollect } from './index.ts';

/** `legion bench` 表格中的一行。 */
export type LegionBenchRow = {
    project: string;
    test: string;
    target: string;
    compileMs: number;
    runtimeMs: number;
};

/** 聚合后的 Wasm 基准计时。 */
export type LegionBenchAggregate = {
    compileMs: number;
    runtimeMs: number;
    rowCount: number;
};

/** 同步参考实现基准选项。 */
export type BenchmarkSyncOptions = {
    iterations?: number;
    warmup?: number;
};

/** 同步参考实现基准结果。 */
export type BenchmarkTimingResult = {
    medianMs: number;
    iterations: number;
    warmup: number;
};

/** 参考实现 vs Legion Wasm 对比。 */
export type BenchmarkComparison = {
    referenceMs: number;
    legionCompileMs: number | null;
    legionRuntimeMs: number | null;
    runtimeRatio: number | null;
    legionRoute: VccCliRoute | 'unavailable' | null;
    error: string | null;
};

export type VccBenchmarkConfig = {
    /** `valkyrie.rs` 根目录；默认 `VALKYRIE_RS_ROOT` 或 `../valkyrie.rs`。 */
    valkyrieRsRoot?: string;
    /** Wasm collect 目录（磁盘路径，避免 Windows 上 package 名解析问题）。 */
    wasmCollectDir?: string;
    /** Wasm 入口脚本名，默认 `legion.mjs`。 */
    wasmEntry?: string;
    /** 可选 wasm 宿主（native 不可用且未指定 `wasmCollectDir` 时使用）。 */
    host?: VccHostRunner;
};

export type LegionBenchProjectOptions = {
    runs?: number;
    target?: string;
};

export type LegionBenchProjectResult = {
    outcome: VccCliSpawnResult;
    rows: LegionBenchRow[];
    aggregate: LegionBenchAggregate | null;
};

export type VccBenchmarkRunner = {
    config: Readonly<
        Required<Pick<VccBenchmarkConfig, 'valkyrieRsRoot' | 'wasmCollectDir' | 'wasmEntry'>> & {
            host?: VccHostRunner;
        }
    >;
    ready: () => boolean;
    skipReason: () => string | null;
    spawnLegion: (argv?: string[]) => VccCliSpawnResult;
    benchProject: (projectDir: string, options?: LegionBenchProjectOptions) => LegionBenchProjectResult;
    compareReference: (referenceMs: number, legion: LegionBenchProjectResult | null, legionError?: string | null) => BenchmarkComparison;
};

/** 批量基准条目（leetcode / project-euler 等 catalog 驱动）。 */
export type BenchmarkSuiteEntry = {
    id: string;
    title?: string;
    projectDir: string;
    /** 参考实现 median 毫秒；抛错则记入 `error`（可 async）。 */
    measureReference?: () => number | Promise<number>;
};

/** 单题对比行（看板 / JSON 报告通用形状）。 */
export type BenchmarkSuiteRow = {
    id: string;
    title: string;
    referenceMs: number | null;
    legionCompileMs: number | null;
    legionRuntimeMs: number | null;
    runtimeRatio: number | null;
    legionRoute: VccCliRoute | null;
    error: string | null;
};

export type BenchmarkSuiteReport = {
    generatedAt: string;
    ready: boolean;
    rows: BenchmarkSuiteRow[];
};

export type RunBenchmarkSuiteOptions = LegionBenchProjectOptions & {
    onProgress?: (row: BenchmarkSuiteRow) => void;
};

const DEFAULT_WASM_ENTRY = 'legion.mjs';
const DEFAULT_WASM_PACKAGE = '@valkyrie-language/vcc-unknown-wasm32';

/**
 * `legion bench --target node`：扫描源码内 `[benchmark]` 函数并计时（valkyrie 工程自测用）。
 * `compileMs` = 每次运行的编译耗时，`runtimeMs` = Wasm 入口执行 `[benchmark]` 函数耗时。
 * leetcode 等外部 harness 应使用 `legion build` + 对 `metadata.tests` 跑产物，勿依赖本 API 的 `runtimeMs`。
 */
export const WASM_NODE_BENCH_TARGET = 'node';

/** 样本中位数（毫秒计时常用）。 */
export function median(values: number[]): number {
    if (values.length === 0) {
        return 0;
    }
    const sorted = [...values].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    if (sorted.length % 2 === 0) {
        return (sorted[mid - 1] + sorted[mid]) / 2;
    }
    return sorted[mid];
}

/** 对同步函数做 warmup + 多次采样，返回 median 毫秒。 */
export function benchmarkSync(fn: () => void, options: BenchmarkSyncOptions = {}): BenchmarkTimingResult {
    const iterations = options.iterations ?? 2000;
    const warmup = options.warmup ?? 200;
    for (let i = 0; i < warmup; i++) {
        fn();
    }
    const samples: number[] = [];
    for (let i = 0; i < iterations; i++) {
        const start = performance.now();
        fn();
        samples.push(performance.now() - start);
    }
    return { medianMs: median(samples), iterations, warmup };
}

/** 解析 `legion bench`  stdout 表格行。 */
export function parseLegionBenchTable(stdout: string): LegionBenchRow[] {
    const rows: LegionBenchRow[] = [];
    for (const line of stdout.split(/\r?\n/)) {
        const match = line.match(/^(\S+)\s+(\S+)\s+(\S+)\s+([\d.]+)\s+([\d.]+)\s*$/);
        if (!match) {
            continue;
        }
        rows.push({
            project: match[1],
            test: match[2],
            target: match[3],
            compileMs: Number(match[4]),
            runtimeMs: Number(match[5]),
        });
    }
    return rows;
}

/** 聚合多行 `legion bench` 结果（默认均值）。 */
export function aggregateLegionBenchRows(rows: LegionBenchRow[], mode: 'mean' | 'sum' | 'max' = 'mean'): LegionBenchAggregate | null {
    if (rows.length === 0) {
        return null;
    }
    if (mode === 'sum') {
        return rows.reduce(
            (acc, row) => ({
                compileMs: acc.compileMs + row.compileMs,
                runtimeMs: acc.runtimeMs + row.runtimeMs,
                rowCount: acc.rowCount + 1,
            }),
            { compileMs: 0, runtimeMs: 0, rowCount: 0 },
        );
    }
    if (mode === 'max') {
        return rows.reduce(
            (acc, row) => ({
                compileMs: Math.max(acc.compileMs, row.compileMs),
                runtimeMs: Math.max(acc.runtimeMs, row.runtimeMs),
                rowCount: acc.rowCount + 1,
            }),
            { compileMs: 0, runtimeMs: 0, rowCount: 0 },
        );
    }
    const totals = rows.reduce(
        (acc, row) => ({
            compileMs: acc.compileMs + row.compileMs,
            runtimeMs: acc.runtimeMs + row.runtimeMs,
        }),
        { compileMs: 0, runtimeMs: 0 },
    );
    return {
        compileMs: totals.compileMs / rows.length,
        runtimeMs: totals.runtimeMs / rows.length,
        rowCount: rows.length,
    };
}

/** 格式化 Legion CLI 失败输出。 */
export function formatLegionCliError(label: string, outcome: VccCliSpawnResult): string {
    return `${label} exited ${outcome.status}\nstdout:\n${outcome.stdout}\nstderr:\n${outcome.stderr}`;
}

function defaultValkyrieRsRoot(configRoot?: string): string {
    if (configRoot) {
        return configRoot;
    }
    if (process.env.VALKYRIE_RS_ROOT) {
        return process.env.VALKYRIE_RS_ROOT;
    }
    return join(process.cwd(), '..', 'valkyrie.rs');
}

function wasmCollectReadyFromDir(wasmCollectDir: string, wasmEntry: string): boolean {
    const mjs = join(wasmCollectDir, wasmEntry);
    const wasm = join(wasmCollectDir, wasmEntry.replace(/\.mjs$/i, '.wasm'));
    return existsSync(mjs) && existsSync(wasm);
}

function spawnWasmLegionFromDir(wasmCollectDir: string, wasmEntry: string, argv: string[]): VccCliSpawnResult {
    const mjs = join(wasmCollectDir, wasmEntry);
    const result = spawnSync(process.execPath, [mjs, ...argv], { encoding: 'utf8' });
    return {
        route: 'wasm',
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ''),
        stderr: String(result.stderr ?? ''),
    };
}

/**
 * 创建基准测试 runner：经 VCC 宿主路由（native platform collect 优先，否则 wasm collect）。
 * 供 leetcode / project-euler 等 conformance 仓对比参考实现与 Valkyrie Wasm。
 */
export function createBenchmarkRunner(config: VccBenchmarkConfig = {}): VccBenchmarkRunner {
    const valkyrieRsRoot = defaultValkyrieRsRoot(config.valkyrieRsRoot);
    const wasmEntry = config.wasmEntry ?? DEFAULT_WASM_ENTRY;
    const wasmCollectDir = config.wasmCollectDir ?? join(valkyrieRsRoot, 'projects', 'packages', 'vcc-unknown-wasm32');
    const host = config.host;

    function resolveHost(): VccHostRunner {
        return (
            host ??
            createHostRunner({
                wasmCollect: DEFAULT_WASM_PACKAGE,
                wasmEntry,
            })
        );
    }

    function ready(): boolean {
        return wasmCollectReadyFromDir(wasmCollectDir, wasmEntry) || locateNativeCollect() !== null || host !== undefined;
    }

    function skipReason(): string | null {
        if (ready()) {
            return null;
        }
        return 'Valkyrie runner not ready: install a @valkyrie-language/vcc-* platform package (native) or run node scripts/build.mjs capability in valkyrie.rs (wasm collect)';
    }

    function spawnLegion(argv: string[] = []): VccCliSpawnResult {
        if (!ready()) {
            return {
                route: 'wasm',
                status: 127,
                stdout: '',
                stderr: skipReason() ?? 'runner unavailable',
            };
        }
        if (wasmCollectReadyFromDir(wasmCollectDir, wasmEntry)) {
            return spawnWasmLegionFromDir(wasmCollectDir, wasmEntry, argv);
        }
        return resolveHost().spawnCli(argv);
    }

    function benchProject(projectDir: string, options: LegionBenchProjectOptions = {}): LegionBenchProjectResult {
        const runs = options.runs ?? 3;
        const target = options.target ?? WASM_NODE_BENCH_TARGET;
        const outcome = spawnLegion(['bench', projectDir, '-t', target, '-n', String(runs)]);
        const rows = parseLegionBenchTable(outcome.stdout);
        const aggregate = outcome.status === 0 ? aggregateLegionBenchRows(rows) : null;
        return { outcome, rows, aggregate };
    }

    function compareReference(
        referenceMs: number,
        legion: LegionBenchProjectResult | null,
        legionError: string | null = null,
    ): BenchmarkComparison {
        if (!legion) {
            return {
                referenceMs,
                legionCompileMs: null,
                legionRuntimeMs: null,
                runtimeRatio: null,
                legionRoute: null,
                error: legionError ?? skipReason(),
            };
        }
        let error = legionError;
        if (legion.outcome.status !== 0) {
            error = formatLegionCliError('legion bench', legion.outcome);
        } else if (!legion.aggregate) {
            const combined = `${legion.outcome.stdout}\n${legion.outcome.stderr}`;
            if (/未发现\s*\[benchmark\]/.test(combined)) {
                error =
                    'legion bench: no [benchmark] functions in project (legion bench only times source [benchmark] blocks, not external harness)';
            } else if (legion.rows.length === 0) {
                error = 'legion bench: exit 0 but stdout had no parseable timing rows';
            } else {
                error = 'legion bench: failed to aggregate timing rows';
            }
        }
        const legionRuntimeMs = legion.aggregate?.runtimeMs ?? null;
        return {
            referenceMs,
            legionCompileMs: legion.aggregate?.compileMs ?? null,
            legionRuntimeMs,
            runtimeRatio: legionRuntimeMs !== null && legionRuntimeMs > 0 ? referenceMs / legionRuntimeMs : null,
            legionRoute: legion.outcome.route,
            error,
        };
    }

    return {
        config: { valkyrieRsRoot, wasmCollectDir, wasmEntry, host },
        ready,
        skipReason,
        spawnLegion,
        benchProject,
        compareReference,
    };
}

function mergeErrors(left: string | null, right: string | null): string | null {
    if (left && right) {
        return `${left}; ${right}`;
    }
    return left ?? right;
}

/** 单题：参考实现 + `legion bench` 对比。 */
export async function runBenchmarkEntry(
    runner: VccBenchmarkRunner,
    entry: BenchmarkSuiteEntry,
    options: LegionBenchProjectOptions = {},
): Promise<BenchmarkSuiteRow> {
    const title = entry.title ?? entry.id;
    let referenceMs: number | null = null;
    let error: string | null = null;

    if (entry.measureReference) {
        try {
            referenceMs = await entry.measureReference();
        } catch (err) {
            error = `reference: ${String(err)}`;
        }
    }

    if (!runner.ready()) {
        return {
            id: entry.id,
            title,
            referenceMs,
            legionCompileMs: null,
            legionRuntimeMs: null,
            runtimeRatio: null,
            legionRoute: null,
            error: mergeErrors(error, runner.skipReason()),
        };
    }

    const legion = runner.benchProject(entry.projectDir, options);
    const comparison = runner.compareReference(referenceMs ?? 0, legion);
    if (comparison.error) {
        error = mergeErrors(error, comparison.error);
    }

    const runtimeRatio =
        referenceMs !== null && comparison.legionRuntimeMs !== null && comparison.legionRuntimeMs > 0
            ? referenceMs / comparison.legionRuntimeMs
            : null;

    return {
        id: entry.id,
        title,
        referenceMs,
        legionCompileMs: comparison.legionCompileMs,
        legionRuntimeMs: comparison.legionRuntimeMs,
        runtimeRatio,
        legionRoute: comparison.legionRoute,
        error,
    };
}

/**
 * 批量跑 catalog 基准：每题可选参考实现 + `legion bench`。
 * 供 conformance 仓生成 `bench-results.json` 一类产物。
 */
export async function runBenchmarkSuite(
    runner: VccBenchmarkRunner,
    entries: Iterable<BenchmarkSuiteEntry>,
    options: RunBenchmarkSuiteOptions = {},
): Promise<BenchmarkSuiteReport> {
    const { onProgress, ...benchOptions } = options;
    const rows: BenchmarkSuiteRow[] = [];
    for (const entry of entries) {
        const row = await runBenchmarkEntry(runner, entry, benchOptions);
        rows.push(row);
        onProgress?.(row);
    }
    return { generatedAt: new Date().toISOString(), ready: runner.ready(), rows };
}
