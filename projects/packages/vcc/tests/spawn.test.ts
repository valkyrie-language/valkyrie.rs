import assert from 'node:assert/strict';
import test from 'node:test';

import { spawnWasmCollect } from '../src/index.ts';
import { wasmCollectReady } from '../src/testing.ts';

test('spawnWasmCollect 在 wasm collect 已装配时可执行 argv', (t) => {
    if (!wasmCollectReady('@valkyrie-language/vcc-unknown-wasm32', 'legion.mjs')) {
        t.skip('wasm collect not assembled');
        return;
    }

    const outcome = spawnWasmCollect('@valkyrie-language/vcc-unknown-wasm32', 'legion.mjs', ['--version']);
    assert.equal(outcome.route, 'wasm');
    assert.equal(typeof outcome.status, 'number');
    assert.equal(typeof outcome.stdout, 'string');
    assert.equal(typeof outcome.stderr, 'string');
});
