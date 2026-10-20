import { spawnSync } from 'node:child_process';

import { resolveWasmMjs } from './native.ts';

/** 默认 wasm 子进程超时（毫秒）。 */
export const DEFAULT_SPAWN_TIMEOUT_MS = 30_000;
/** 默认 wasm 子进程 stdout/stderr 缓冲上限（字节）。 */
export const DEFAULT_SPAWN_MAX_BUFFER_BYTES = 8 * 1024 * 1024;

export type VccCliRoute = 'native' | 'wasm';

/** 可测试的 CLI 子进程结果（不调用 `process.exit`）。 */
export type VccCliSpawnResult = {
    route: VccCliRoute;
    status: number;
    stdout: string;
    stderr: string;
};

export type SpawnWasmCollectOptions = {
    timeoutMs?: number;
    maxBufferBytes?: number;
};

/** 将程序化 build 结果转为 spawn 结果形态。 */
export function legionBuildOutcomeToSpawnResult(outcome: import('./native.ts').LegionBuildOutcome): VccCliSpawnResult {
    return {
        route: 'native',
        status: outcome.ok ? 0 : 1,
        stdout: outcome.ok ? `artifacts: ${outcome.artifacts.join(', ')}` : '',
        stderr: outcome.error ?? '',
    };
}

/** 将程序化 plan 结果转为 spawn 结果形态。 */
export function legionPlanOutcomeToSpawnResult(outcome: import('./native.ts').LegionPlanOutcome): VccCliSpawnResult {
    const groups = outcome.groups.map((group) => `${group.dependencyKey}=${group.fileCount}`).join(', ');
    return {
        route: 'native',
        status: outcome.ok ? 0 : 1,
        stdout: outcome.ok ? `plan ok: project=${outcome.project} sources=${outcome.sourceCount} groups=${groups}` : '',
        stderr: outcome.error ?? '',
    };
}

/**
 * 在 Wasm collect 入口上执行 argv（正交：仅启动 wasm 脚本，不经 N-API argv 聚合）。
 */
export function spawnWasmCollect(
    wasmCollect: string,
    wasmEntry: string,
    argv: string[] = [],
    options: SpawnWasmCollectOptions = {},
): VccCliSpawnResult {
    const timeoutMs = options.timeoutMs ?? DEFAULT_SPAWN_TIMEOUT_MS;
    const maxBufferBytes = options.maxBufferBytes ?? DEFAULT_SPAWN_MAX_BUFFER_BYTES;
    const mjs = resolveWasmMjs(wasmCollect, wasmEntry);
    const result = spawnSync(process.execPath, [mjs, ...argv], {
        encoding: 'utf8',
        timeout: timeoutMs,
        maxBuffer: maxBufferBytes,
    });
    if (result.error?.message.includes('ETIMEDOUT') || result.signal === 'SIGTERM') {
        return {
            route: 'wasm',
            status: 124,
            stdout: String(result.stdout ?? ''),
            stderr: `${String(result.stderr ?? '')}\nspawnWasmCollect timed out after ${timeoutMs}ms`,
        };
    }
    return {
        route: 'wasm',
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ''),
        stderr: String(result.stderr ?? ''),
    };
}
