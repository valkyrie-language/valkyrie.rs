import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

import { MIN_EXECUTION_MANIFEST_SCHEMA_VERSION } from '../src/execution-manifest.ts';
import { NODE_WASM_TARGET, listWasmModuleExports, resolveArtifactDir, resolveNodeEntry } from '../src/testing.ts';

function sha256(text: string): string {
    return createHash('sha256').update(text).digest('hex');
}

function writeNodeManifestV3(root: string, physical: string, mjsBody: string, wasmBody: string): void {
    const wasmRelative = physical.replace(/\.mjs$/i, '.wasm');
    writeFileSync(join(root, physical), mjsBody, 'utf8');
    writeFileSync(join(root, wasmRelative), wasmBody);
    writeFileSync(
        join(root, 'run-contracts.txt'),
        `{
        schema_version: ${MIN_EXECUTION_MANIFEST_SCHEMA_VERSION},
        target: "${NODE_WASM_TARGET}",
        project_name: "demo",
        compile_evidence: {
            semantic_closure_hash: "abc",
            canonical_surface_hash: "def",
            host_provider_binding_hash: "ghi"
        },
        inputs: [
            {
                role: "source",
                path: "solution.v",
                hash: "${sha256('source')}"
            }
        ],
        artifacts: [
            {
                path: "${physical}",
                hash: "${sha256(mjsBody)}"
            },
            {
                path: "${wasmRelative}",
                hash: "${sha256(wasmBody)}"
            }
        ],
        run_contracts: [
            {
                physical_entry: "${physical}",
                logical_entry: "main",
                invocation: "node"
            }
        ]
    }`,
        'utf8',
    );
}

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

test('resolveNodeEntry reads physical_entry from execution manifest', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-entry-'));
    try {
        writeNodeManifestV3(root, 'legion.mjs', 'export {}\n', '\0asm');

        const entry = resolveNodeEntry(root);
        assert.ok(entry);
        assert.equal(entry.physicalEntry, 'legion.mjs');
        assert.equal(entry.legacy, false);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test('resolveNodeEntry ignores legacy files when manifest names another physical_entry', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-entry-legacy-'));
    try {
        writeNodeManifestV3(root, 'demo.mjs', 'export {}\n', '\0asm');
        writeFileSync(join(root, 'legion_tools.mjs'), 'export {}\n');
        writeFileSync(join(root, 'legion_tools.wasm'), '\0legacy');

        const entry = resolveNodeEntry(root);
        assert.ok(entry);
        assert.equal(entry.physicalEntry, 'demo.mjs');
        assert.match(entry.legionMjs, /demo\.mjs$/);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test('resolveNodeEntry rejects directory without execution manifest', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-entry-missing-manifest-'));
    try {
        writeFileSync(join(root, 'legion.mjs'), 'export {}\n');
        writeFileSync(join(root, 'legion.wasm'), '\0asm');
        assert.equal(resolveNodeEntry(root), null);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test('listWasmModuleExports reads export names without format-spy', () => {
    const packagedWasm = join(dirname(fileURLToPath(import.meta.url)), '../node_modules/@valkyrie-language/vcc-unknown-wasm32/legion.wasm');
    const exports = listWasmModuleExports(packagedWasm);
    assert.ok(exports.length > 0, 'packaged legion.wasm must expose at least one export');
});
