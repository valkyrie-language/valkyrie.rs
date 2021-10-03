import { NATIVE_PACKAGES } from "@valkyrie-language/vcc";

import { host } from "./host.ts";

export const WASM_COLLECT = host.config.wasmCollect;
export const WASM_ENTRY = host.config.wasmEntry;
export const locateNativeCollect = host.locateNativeCollect;
export const resolveWasmMjs = host.resolveWasmMjs;
export const spawnCli = host.spawnCli;
export const runCli = host.runCli;

export { NATIVE_PACKAGES };
