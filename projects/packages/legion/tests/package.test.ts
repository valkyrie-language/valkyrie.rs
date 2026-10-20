import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

import { LEGION_WASM_COLLECT, LEGION_WASM_ENTRY } from '@valkyrie-language/vcc/legion';

import { resolveWasmMjs, runCli, spawnCli } from '../src/index.ts';

const PACKAGE_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');

test('package.json 仅依赖 vcc 与 commander（collect 由 vcc 装配）', () => {
    const pkg = JSON.parse(readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf8'));
    assert.equal(pkg.name, '@valkyrie-language/legion');
    assert.equal(pkg.bin.legion, 'bin/legion.js');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc'], 'workspace:*');
    assert.equal(pkg.dependencies['@vmz/commander'], '^0.1.0');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc-unknown-wasm32'], undefined);
    assert.equal(pkg.optionalDependencies, undefined);
});

test('bin/legion.js 存在且委托 src/cli/cli.ts', () => {
    const binPath = join(PACKAGE_ROOT, 'bin', 'legion.js');
    assert.ok(existsSync(binPath));
    const source = readFileSync(binPath, 'utf8');
    assert.match(source, /runCli/);
    assert.match(source, /\.\.\/src\/cli\/cli\.ts/);
});

test('Legion wasm 路由经 vcc/legion 子路径解析', () => {
    assert.equal(typeof runCli, 'function');
    assert.equal(typeof spawnCli, 'function');
    assert.equal(LEGION_WASM_COLLECT, '@valkyrie-language/vcc-unknown-wasm32');
    assert.equal(LEGION_WASM_ENTRY, 'legion.mjs');
    assert.ok(resolveWasmMjs().includes('vcc-unknown-wasm32'));
});
