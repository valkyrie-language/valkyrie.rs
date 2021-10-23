import { createHostRunner } from '@valkyrie-language/vcc';

export const host = createHostRunner({
    wasmCollect: '@valkyrie-language/vcc-unknown-wasm32',
    wasmEntry: 'legion.mjs',
});
