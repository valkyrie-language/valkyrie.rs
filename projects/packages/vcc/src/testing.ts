import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

import type { VccCliSpawnResult, VccHostRunner } from "./index.ts";
import { locateNativeCollect } from "./index.ts";
import { resolveWasmMjs } from "./index.ts";

/** Node Wasm GC 目标三元组（与 legion build --target node 对齐）。 */
export const NODE_WASM_TARGET = "wasm32-node-unknown-wasm";

/** 解析后的 Node 入口产物。 */
export type NodeWasmEntry = {
    legionMjs: string;
    legionWasm: string;
    physicalEntry: string;
    legacy: boolean;
};

/** Wasm collect 是否已装配（入口脚本与同名 `.wasm` 均存在）。 */
export function wasmCollectReady(wasmCollect: string, wasmEntry: string): boolean {
    const mjs = resolveWasmMjs(wasmCollect, wasmEntry);
    const wasm = join(dirname(mjs), wasmEntry.replace(/\.mjs$/i, ".wasm"));
    return existsSync(mjs) && existsSync(wasm);
}

/** 严格集成模式：`LEGION_INTEGRATION=1` 时缺少 collect 视为失败而非 skip。 */
export function integrationRequired(envName = "LEGION_INTEGRATION"): boolean {
    return process.env[envName] === "1";
}

/** 在 collect 未装配时返回 skip 原因；`integrationRequired()` 时返回 `null`（应 fail）。 */
export function skipUnlessWasmCollectReady(wasmCollect: string, wasmEntry: string, envName = "LEGION_INTEGRATION"): string | null {
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

function readPhysicalEntryFromContract(contractPath: string): string | null {
    if (!existsSync(contractPath)) {
        return null;
    }
    const text = readFileSync(contractPath, "utf8");
    const match = text.match(/physical_entry\s*:\s*"([^"]+\.mjs)"/);
    return match?.[1] ?? null;
}

/**
 * 在 Node 构建产物目录中定位 `legion.mjs` / `legion.wasm`。
 * 解析 Node 构建产物目录中的 `legion.mjs` / `legion.wasm`（含 run-contracts 与 legacy 命名）。
 */
export function resolveNodeEntry(targetDir: string): NodeWasmEntry | null {
    const canonicalMjs = join(targetDir, "legion.mjs");
    const canonicalWasm = join(targetDir, "legion.wasm");
    if (existsSync(canonicalMjs) && existsSync(canonicalWasm)) {
        return {
            legionMjs: canonicalMjs,
            legionWasm: canonicalWasm,
            physicalEntry: "legion.mjs",
            legacy: false,
        };
    }

    for (const contractName of ["run-contracts.txt", "run-contract.txt"]) {
        const physical = readPhysicalEntryFromContract(join(targetDir, contractName));
        if (!physical) {
            continue;
        }
        const mjs = join(targetDir, physical);
        const wasm = join(targetDir, physical.replace(/\.mjs$/i, ".wasm"));
        if (existsSync(mjs) && existsSync(wasm)) {
            return { legionMjs: mjs, legionWasm: wasm, physicalEntry: physical, legacy: false };
        }
    }

    const legacyMjs = join(targetDir, "legion_tools.mjs");
    const legacyWasm = join(targetDir, "legion_tools.wasm");
    if (existsSync(legacyMjs) && existsSync(legacyWasm)) {
        return {
            legionMjs: legacyMjs,
            legionWasm: legacyWasm,
            physicalEntry: "legion_tools.mjs",
            legacy: true,
        };
    }

    return null;
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
    const result = spawnSync(process.execPath, [binPath, ...argv], { encoding: "utf8" });
    return {
        route: "wasm",
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ""),
        stderr: String(result.stderr ?? ""),
    };
}

/** 运行已构建的 Node Wasm 入口（`node legion.mjs …`）。 */
export function spawnBuiltNodeEntry(entryMjs: string, argv: string[] = []): VccCliSpawnResult {
    const result = spawnSync(process.execPath, [entryMjs, ...argv], { encoding: "utf8" });
    return {
        route: "wasm",
        status: result.status ?? 1,
        stdout: String(result.stdout ?? ""),
        stderr: String(result.stderr ?? ""),
    };
}
