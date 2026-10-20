import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

import { locateNativeCollect, resolveWasmMjs } from './native.ts';
import type { VccCliSpawnResult } from './spawn-wasm.ts';
import type { CliHostRunner } from './types.ts';

import { MIN_EXECUTION_MANIFEST_SCHEMA_VERSION, readExecutionManifest } from './execution-manifest.ts';

export {
    COMPILE_EVIDENCE_FILE,
    EXECUTION_MANIFEST_FILE,
    MIN_EXECUTION_MANIFEST_SCHEMA_VERSION,
    isArtifactCacheFresh,
    parseExecutionManifest,
    readExecutionManifest,
    verifyExecutionManifestFresh,
    type ArtifactCacheFreshness,
    type CompileEvidenceDigest,
    type ExecutionArtifactDigest,
    type ExecutionInputDigest,
    type ExecutionManifest,
    type RunContractDigest,
} from './execution-manifest.ts';

/** Nyar VM 目标三元组（与 `legion build --target nyar` 对齐）。 */
export const NYAR_VM_TARGET = 'nyar-unknown-unknown-managed';

/** Node Wasm GC 目标三元组（与 legion build --target node 对齐）。 */
export const NODE_WASM_TARGET = 'wasm32-node-unknown-wasm';

/** 解析后的 Node 入口产物。 */
export type NodeWasmEntry = {
    legionMjs: string;
    legionWasm: string;
    physicalEntry: string;
    legacy: boolean;
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

/** 集成测试可用：已装配 wasm collect，或已安装 VCC native platform collect。 */
export function integrationRunnerReady(wasmCollect: string, wasmEntry: string): boolean {
    return wasmCollectReady(wasmCollect, wasmEntry) || locateNativeCollect() !== null;
}

/**
 * 解析 `legion build -o` 输出目录。
 * 兼容 `{out}/` 与 `{out}/{targetTriple}/` 两种布局。
 */
export function resolveArtifactDir(outputDir: string, targetTriple = NODE_WASM_TARGET): string {
    const nested = join(outputDir, targetTriple);
    return existsSync(nested) ? nested : outputDir;
}

function readLogicalEntryFromContract(contractPath: string): string | null {
    if (!existsSync(contractPath)) {
        return null;
    }
    const text = readFileSync(contractPath, 'utf8');
    // oak-von 的对象条目同时接受 `=` 与 `:`；Rust `write_von_indented` 实际写出 `=`。
    const match = text.match(/logical_entry\s*[:=]\s*"([^"]+)"/);
    return match?.[1] ?? null;
}

function wasmSiblingPath(physicalMjs: string): string {
    return physicalMjs.replace(/\.mjs$/i, '.wasm');
}

/**
 * 从已解析的 execution manifest 定位 Node wasm/js 入口。
 * 只接受 manifest 列出的 physical_entry 与 artifacts，不做目录扫描或 legacy fallback。
 */
export function resolveNodeEntryFromManifest(manifest: ExecutionManifest | null, targetDir: string): NodeWasmEntry | null {
    if (!manifest || manifest.schema_version !== MIN_EXECUTION_MANIFEST_SCHEMA_VERSION) {
        return null;
    }
    const contract = manifest.run_contracts.find((entry) => entry.physical_entry.endsWith('.mjs'));
    if (!contract) {
        return null;
    }
    const physical = contract.physical_entry;
    const wasmRelative = wasmSiblingPath(physical);
    const artifactPaths = new Set(manifest.artifacts.map((artifact) => artifact.path));
    if (!artifactPaths.has(physical) || !artifactPaths.has(wasmRelative)) {
        return null;
    }
    const legionMjs = join(targetDir, physical);
    const legionWasm = join(targetDir, wasmRelative);
    if (!existsSync(legionMjs) || !existsSync(legionWasm)) {
        return null;
    }
    return {
        legionMjs,
        legionWasm,
        physicalEntry: physical,
        legacy: false,
    };
}

/**
 * 在 Node 构建产物目录中定位 wasm/js 入口（必须存在 schema 与当前基线相等的 execution manifest）。
 */
export function resolveNodeEntry(targetDir: string): NodeWasmEntry | null {
    return resolveNodeEntryFromManifest(readExecutionManifest(targetDir), targetDir);
}

/** 用 Node 内置 WebAssembly 解析模块导出，不依赖 `format-spy` / `vcc-data`。 */
export function listWasmModuleExports(wasmPath: string): string[] {
    try {
        const bytes = readFileSync(wasmPath);
        const module = new WebAssembly.Module(bytes);
        return WebAssembly.Module.exports(module)
            .map((entry) => entry.name)
            .filter((name): name is string => typeof name === 'string' && name.length > 0);
    } catch {
        return [];
    }
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
    const physical = text.match(/physical_entry\s*[:=]\s*"([^"]+\.nyar)"/)?.[1];
    if (!physical) {
        return null;
    }
    const nyarPath = join(targetDir, physical);
    return existsSync(nyarPath)
        ? { nyarPath, physicalEntry: physical, logicalEntry: readLogicalEntryFromContract(contractPath) ?? 'main' }
        : null;
}

/** 通过产品宿主运行 CLI（供集成测试使用）。 */
export function spawnHostCli(host: CliHostRunner, argv: string[] = []): VccCliSpawnResult {
    return host.spawnCli(argv);
}

/** 通过 Legion 宿主运行 CLI（native platform collect 优先，否则 wasm collect）。 */
export function spawnLegionForIntegration(host: CliHostRunner, argv: string[] = []): VccCliSpawnResult {
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
