import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { NODE_WASM_TARGET, resolveArtifactDir, resolveNodeEntry } from '../src/testing.ts';

test('resolveArtifactDir prefers nested target triple', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-artifact-'));
    try {
        const nested = join(root, NODE_WASM_TARGET);
        mkdirSync(nested, { recursive: true });
        writeFileSync(join(nested, 'legion.mjs'), 'export {}\n');

        assert.equal(resolveArtifactDir(root, NODE_WASM_TARGET), nested);
        assert.equal(resolveArtifactDir(root, 'other-target'), root);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test('resolveNodeEntry reads canonical legion.mjs contract', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-entry-'));
    try {
        writeFileSync(join(root, 'legion.mjs'), 'export {}\n');
        writeFileSync(join(root, 'legion.wasm'), '\0');
        writeFileSync(join(root, 'run-contracts.txt'), 'physical_entry: "legion.mjs"\n');

        const entry = resolveNodeEntry(root);
        assert.ok(entry);
        assert.equal(entry.physicalEntry, 'legion.mjs');
        assert.equal(entry.legacy, false);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});
