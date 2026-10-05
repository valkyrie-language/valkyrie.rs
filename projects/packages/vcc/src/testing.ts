import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

import type { VccCliSpawnResult, VccHostRunner } from './index.ts';
import { resolveWasmMjs } from './index.ts';

function defaultValkyrieRsRoot(configRoot?: string): string {
    if (configRoot) {
        return configRoot;
    }
    if (process.env.VALKYRIE_RS_ROOT) {
        return process.env.VALKYRIE_RS_ROOT;
    }
    return join(process.cwd(), '..', 'valkyrie.rs');
}

/** Nyar VM 目标三元组（与 `legion build --target nyar` 对齐）。 */
export const NYAR_VM_TARGET = 'nyar-unknown-unknown-managed';

/** Node Wasm GC 目标三元组（与 legion build --target node 对齐）。 */
export const NODE_WASM_TARGET = 'wasm32-node-unknown-wasm';

/** 解析后的 Node 入口产物。 */
export type NodeWasmEntry = {
    legionMjs: string;
    legionWasm: string;
    physicalEntry: string;
};

/** 解析后的 Nyar VM 入口产物。 */
export type NyarVmEntry = {
    nyarPath: string;
    physicalEntry: string;
    logicalEntry: string;
};

/** Wasm collect 是否已装配（入口脚本与同名 `.wasm` 均存在）。 */
export function wasmCollectReady(wasmCollect: string, wasmEntry: string): boolean {
    const mjs = resolveWasmMjs(wasmCollect, wasmEntry);
    const wasm = join(dirname(mjs), wasmEntry.replace(/\.mjs$/i, '.wasm'));
    return existsSync(mjs) && existsSync(wasm);
}

/** 严格集成模式：`LEGION_INTEGRATION=1` 时缺少 collect 视为失败而非 skip。 */
export function integrationRequired(envName = 'LEGION_INTEGRATION'): boolean {
    return process.env[envName] === '1';
}

/** 在 collect 未装配时返回 skip 原因；`integrationRequired()` 时返回 `null`（应 fail）。 */
export function skipUnlessWasmCollectReady(wasmCollect: string, wasmEntry: string, envName = 'LEGION_INTEGRATION'): string | null {
    if (wasmCollectReady(wasmCollect, wasmEntry)) {
        return null;
    }
    if (integrationRequired(envName)) {
        return null;
    }
    return `wasm collect not assembled (${wasmCollect}/${wasmEntry}); run pnpm assemble`;
}

/** 集成测试只接受已装配的 Wasm collect。 */
export function integrationRunnerReady(wasmCollect: string, wasmEntry: string): boolean {
    return wasmCollectReady(wasmCollect, wasmEntry);
}

/**
 * 解析 `legion build -o` 输出目录。
 * 编译产物必须直接位于调用方指定的输出目录。
 */
export function resolveArtifactDir(outputDir: string, targetTriple = NODE_WASM_TARGET): string {
    void targetTriple;
    return outputDir;
}

function readLogicalEntryFromContract(contractPath: string): string | null {
    if (!existsSync(contractPath)) {
        return null;
    }
    const text = readFileSync(contractPath, 'utf8');
    const match = text.match(/logical_entry\s*:\s*"([^"]+)"/);
    return match?.[1] ?? null;
}

/**
 * 在 Node 构建产物目录中定位 `legion.mjs` / `legion.wasm`。
 * 解析 Node 构建产物目录中的规范 `legion.mjs` / `legion.wasm`。
 */
export function resolveNodeEntry(targetDir: string): NodeWasmEntry | null {
    const canonicalMjs = join(targetDir, 'legion.mjs');
    const canonicalWasm = join(targetDir, 'legion.wasm');
    if (existsSync(canonicalMjs) && existsSync(canonicalWasm)) {
        return {
            legionMjs: canonicalMjs,
            legionWasm: canonicalWasm,
            physicalEntry: 'legion.mjs',
        };
    }

    return null;
}

/**
 * 在 Nyar 构建产物目录中定位 `.nyar` 模块与 run-contract 入口。
 */
export function resolveNyarEntry(targetDir: string): NyarVmEntry | null {
    const contractPath = join(targetDir, 'run-contracts.txt');
    if (!existsSync(contractPath)) {
        return null;
    }
    const text = readFileSync(contractPath, 'utf8');
    const physical = text.match(/physical_entry\s*:\s*"([^"]+\.nyar)"/)?.[1];
    if (!physical) {
        return null;
    }
    const nyarPath = join(targetDir, physical);
    return existsSync(nyarPath) ? { nyarPath, physicalEntry: physical, logicalEntry: readLogicalEntryFromContract(contractPath) ?? 'main' } : null;
}

/** 通过组装宿主运行 CLI（供集成测试使用）。 */
export function spawnHostCli(host: VccHostRunner, argv: string[] = []): VccCliSpawnResult {
    return host.spawnCli(argv);
}

/** 通过组装 VCC 宿主运行 CLI（native platform collect 优先，否则 wasm collect）。 */
export function spawnLegionForIntegration(host: VccHostRunner, argv: string[] = []): VccCliSpawnResult {
    return host.spawnCli(argv);
}

/** 通过 `bin/*.js` 启动真实用户入口（stdio 捕获）。 */
export function spawnPackageBin(binPath: string, argv: string[] = []): VccCliSpawnResult {
    const result = spawnSync(process.execPath, [binPath, ...argv], { encoding: 'utf8' });
    return {
        route: 'wasm',
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ''),
        stderr: String(result.stderr ?? ''),
    };
}

/**
 * 解析本机 Rust seed `vcc` 可执行文件。
 * 优先序：`VCC_BIN` → 兼容别名 `LEGION_BIN` → `target/{release,debug}/vcc[.exe]`。
 * 铁律：valkyrie.rs 不得产出 `legion.exe`；本函数也不再查找该文件名。
 */
export function locateNativeLegionBinary(valkyrieRsRoot?: string): string | null {
    for (const key of ['VCC_BIN', 'LEGION_BIN'] as const) {
        const override = process.env[key]?.trim();
        if (override && existsSync(override)) {
            return override;
        }
    }
    const root = defaultValkyrieRsRoot(valkyrieRsRoot);
    const base = process.platform === 'win32' ? 'vcc.exe' : 'vcc';
    for (const profile of ['release', 'debug'] as const) {
        const candidate = join(root, 'target', profile, base);
        if (existsSync(candidate)) {
            return candidate;
        }
    }
    return null;
}

/** 经本机 seed `vcc` 子进程调用 CLI（`nyar` 等需 `legacy-lanes` 的 target 应走此路径）。 */
export function spawnNativeLegion(valkyrieRsRoot: string | undefined, argv: string[]): VccCliSpawnResult {
    const binary = locateNativeLegionBinary(valkyrieRsRoot);
    if (!binary) {
        return {
            route: 'native',
            status: 127,
            stdout: '',
            stderr: 'native vcc not found (set VCC_BIN or cargo build -p legion → target/*/vcc)',
        };
    }
    const result = spawnSync(binary, argv, { encoding: 'utf8' });
    return {
        route: 'native',
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ''),
        stderr: String(result.stderr ?? ''),
    };
}

/** 运行已构建的 Node Wasm 入口（`node legion.mjs …`）。 */
export function spawnBuiltNodeEntry(entryMjs: string, argv: string[] = []): VccCliSpawnResult {
    const result = spawnSync(process.execPath, [entryMjs, ...argv], { encoding: 'utf8' });
    return {
        route: 'wasm',
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ''),
        stderr: String(result.stderr ?? ''),
    };
}
