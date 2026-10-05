import { host } from './host.ts';

export const WASM_COLLECT = host.config.wasmCollect;
export const WASM_ENTRY = host.config.wasmEntry;
export const resolveWasmMjs = host.resolveWasmMjs;
export const spawnCli = host.spawnCli;
export const spawnSpy = host.spawnSpy;
export const runCli = host.runCli;
