import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

import { NATIVE_PACKAGES, WASM_COLLECT, WASM_ENTRY, resolveWasmMjs, runCli, spawnCli } from '../src/index.ts';

const PACKAGE_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');

test('package.json 暴露 legion bin 与 workspace 依赖', () => {
    const pkg = JSON.parse(readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf8'));
    assert.equal(pkg.name, '@valkyrie-language/legion');
    assert.equal(pkg.bin.legion, 'bin/legion.js');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc'], 'workspace:*');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc-unknown-wasm32'], 'workspace:*');
});

test('bin/legion.js 存在且委托 src/index.ts', () => {
    const binPath = join(PACKAGE_ROOT, 'bin', 'legion.js');
    assert.ok(existsSync(binPath));
    const source = readFileSync(binPath, 'utf8');
    assert.match(source, /runCli/);
    assert.match(source, /\.\.\/src\/index\.ts/);
});

test('Legion 宿主绑定 vcc-unknown-wasm32 / legion.mjs', () => {
    assert.equal(typeof runCli, 'function');
    assert.equal(typeof spawnCli, 'function');
    assert.equal(WASM_COLLECT, '@valkyrie-language/vcc-unknown-wasm32');
    assert.equal(WASM_ENTRY, 'legion.mjs');
    assert.ok(NATIVE_PACKAGES.length === 4);
    assert.ok(resolveWasmMjs().includes('vcc-unknown-wasm32'));
});
