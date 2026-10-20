/** Legion wasm collect 包名（由 vcc 装配，产品包不得直接依赖）。 */
export const LEGION_WASM_COLLECT = '@valkyrie-language/vcc-unknown-wasm32';

/** Legion wasm 入口脚本（相对 collect 包根）。 */
export const LEGION_WASM_ENTRY = 'legion.mjs';

/** Asgard wasm collect 包名（由 vcc 装配，产品包不得直接依赖）。 */
export const ASGARD_WASM_COLLECT = '@valkyrie-language/vcc-wasm32-wasi';

/** Asgard wasm 入口脚本（相对 collect 包根）。 */
export const ASGARD_WASM_ENTRY = 'asgard.mjs';
