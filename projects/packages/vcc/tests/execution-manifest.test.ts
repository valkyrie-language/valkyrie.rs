import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { createHash } from 'node:crypto';

import {
    isArtifactCacheFresh,
    MIN_EXECUTION_MANIFEST_SCHEMA_VERSION,
    parseExecutionManifest,
    verifyExecutionManifestFresh,
} from '../src/execution-manifest.ts';

function sha256(text: string): string {
    return createHash('sha256').update(text).digest('hex');
}

test('parseExecutionManifest reads schema, digests and compile evidence', () => {
    const sourcePath = 'C:/tmp/solution.v';
    const manifestSource = `{
        schema_version: ${MIN_EXECUTION_MANIFEST_SCHEMA_VERSION},
        target: "wasm32-node-unknown-wasm",
        project_name: "demo",
        compile_evidence: {
            semantic_closure_hash: "abc",
            canonical_surface_hash: "def",
            host_provider_binding_hash: "ghi"
        },
        inputs: [
            {
                role: "source",
                path: "${sourcePath.replace(/\\/g, '\\\\')}",
                hash: "${sha256('source')}"
            }
        ],
        artifacts: [
            {
                path: "demo.wasm",
                hash: "${sha256('wasm')}"
            }
        ],
        run_contracts: [
            {
                physical_entry: "demo.mjs",
                logical_entry: "main",
                invocation: "node"
            }
        ]
    }`;

    const manifest = parseExecutionManifest(manifestSource);
    assert.ok(manifest);
    assert.equal(manifest?.schema_version, MIN_EXECUTION_MANIFEST_SCHEMA_VERSION);
    assert.equal(manifest?.inputs.length, 1);
    assert.equal(manifest?.artifacts.length, 1);
    assert.equal(manifest?.compile_evidence?.semantic_closure_hash, 'abc');
});

test('verifyExecutionManifestFresh rejects legacy schema without compile evidence', () => {
    const manifest = parseExecutionManifest(`{
        schema_version: 1,
        target: "wasm32-node-unknown-wasm",
        project_name: "legacy",
        inputs: [],
        artifacts: [],
        run_contracts: []
    }`);
    assert.ok(manifest);
    const reason = verifyExecutionManifestFresh(manifest!, '/tmp/out', 'wasm32-node-unknown-wasm');
    assert.match(reason ?? '', /schema 1 </);
});

test('isArtifactCacheFresh validates input and artifact digests', () => {
    const root = mkdtempSync(join(tmpdir(), 'vcc-manifest-'));
    const sourcePath = join(root, 'solution.v');
    const artifactDir = join(root, 'out');
    mkdirSync(artifactDir, { recursive: true });
    writeFileSync(sourcePath, 'micro main() {}', 'utf8');
    const wasmPath = join(artifactDir, 'demo.wasm');
    writeFileSync(wasmPath, 'wasm-bytes', 'utf8');

    const manifestSource = `{
        schema_version: ${MIN_EXECUTION_MANIFEST_SCHEMA_VERSION},
        target: "wasm32-node-unknown-wasm",
        project_name: "demo",
        compile_evidence: {
            semantic_closure_hash: "abc",
            canonical_surface_hash: "def",
            host_provider_binding_hash: "ghi"
        },
        inputs: [
            {
                role: "source",
                path: "${sourcePath.replace(/\\/g, '\\\\')}",
                hash: "${sha256('micro main() {}')}"
            }
        ],
        artifacts: [
            {
                path: "demo.wasm",
                hash: "${sha256('wasm-bytes')}"
            }
        ],
        run_contracts: [
            {
                physical_entry: "demo.mjs",
                logical_entry: "main",
                invocation: "node"
            }
        ]
    }`;
    writeFileSync(join(artifactDir, 'run-contracts.txt'), manifestSource, 'utf8');
    writeFileSync(
        join(artifactDir, 'compile-evidence.txt'),
        `{
            semantic_closure_hash: "abc",
            canonical_surface_hash: "def",
            host_provider_binding_hash: "ghi"
        }`,
        'utf8',
    );

    const fresh = isArtifactCacheFresh(artifactDir, 'wasm32-node-unknown-wasm');
    assert.equal(fresh.fresh, true);
    assert.equal(fresh.reason, null);
});
