import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

import { ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY } from '@valkyrie-language/vcc/asgard';

import { resolveWasmMjs, runCli } from '../src/index.ts';

const PACKAGE_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');

test('package.json 仅依赖 vcc 与 commander（collect 由 vcc 装配）', () => {
    const pkg = JSON.parse(readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf8'));
    assert.equal(pkg.name, '@valkyrie-language/asgard');
    assert.equal(pkg.bin.asgard, 'bin/asgard.js');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc'], 'workspace:*');
    assert.equal(pkg.dependencies['@vmz/commander'], '^0.1.0');
    assert.equal(pkg.dependencies['@valkyrie-language/vcc-wasm32-wasi'], undefined);
    assert.equal(pkg.optionalDependencies, undefined);
});

test('bin/asgard.js 存在且委托 src/cli/cli.ts', () => {
    const binPath = join(PACKAGE_ROOT, 'bin', 'asgard.js');
    assert.ok(existsSync(binPath));
    const source = readFileSync(binPath, 'utf8');
    assert.match(source, /runCli/);
    assert.match(source, /\.\.\/src\/cli\/cli\.ts/);
});

test('Asgard wasm 路由经 vcc/asgard 子路径解析', () => {
    assert.equal(typeof runCli, 'function');
    assert.equal(ASGARD_WASM_COLLECT, '@valkyrie-language/vcc-wasm32-wasi');
    assert.equal(ASGARD_WASM_ENTRY, 'asgard.mjs');
    assert.ok(resolveWasmMjs().includes('vcc-wasm32-wasi'));
});
