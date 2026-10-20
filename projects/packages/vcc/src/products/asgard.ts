import { resolveWasmMjs } from '../native.ts';
import { spawnWasmCollect, type VccCliSpawnResult } from '../spawn-wasm.ts';
import { ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY } from '../collects.ts';

export { ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY };

/** 解析 Asgard wasm 入口绝对路径。 */
export function resolveAsgardWasmMjs(): string {
    return resolveWasmMjs(ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY);
}

/** Asgard CLI：当前仅 wasm collect 路由。 */
export function spawnAsgardCli(argv: string[] = []): VccCliSpawnResult {
    return spawnWasmCollect(ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY, argv);
}
