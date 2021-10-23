import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
    NODE_WASM_TARGET,
    integrationRequired,
    integrationRunnerReady,
    resolveArtifactDir,
    resolveNodeEntry,
    spawnBuiltNodeEntry,
    spawnLegionForIntegration,
    wasmCollectReady,
} from "@valkyrie-language/vcc/testing";

import { WASM_COLLECT, WASM_ENTRY } from "../src/index.ts";
import { host } from "../src/host.ts";

const PACKAGE_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const MINIMAL_NODE_FIXTURE = join(PACKAGE_ROOT, "tests", "fixtures", "minimal-node");
const BUILD_TIMEOUT_MS = 10 * 60 * 1000;

function gateOrSkip(t: test.TestContext): boolean {
    if (integrationRunnerReady(WASM_COLLECT, WASM_ENTRY)) {
        return true;
    }
    if (integrationRequired()) {
        assert.fail("LEGION_INTEGRATION=1 requires wasm collect (node scripts/build.mjs capability) or a VCC native platform package");
    }
    t.skip("no wasm collect and no VCC native platform collect");
    return false;
}

function formatOutcome(label: string, outcome: { status: number; stdout: string; stderr: string }): string {
    return `${label} exited ${outcome.status}\nstdout:\n${outcome.stdout}\nstderr:\n${outcome.stderr}`;
}

test("integration gate documents wasm or native runner", () => {
    const ready = integrationRunnerReady(WASM_COLLECT, WASM_ENTRY);
    if (ready) {
        assert.ok(true);
        return;
    }
    assert.equal(integrationRequired(), false);
});

test("legion build minimal-node → canonical legion.mjs/wasm → runtime exit 0", { timeout: BUILD_TIMEOUT_MS }, (t) => {
    if (!gateOrSkip(t)) return;

    const outRoot = mkdtempSync(join(tmpdir(), "legion-it-build-"));
    try {
        const build = spawnLegionForIntegration(host, [
            "build",
            MINIMAL_NODE_FIXTURE,
            "--target",
            "node",
            "-o",
            outRoot,
        ]);
        assert.equal(build.status, 0, formatOutcome("legion build", build));
        assert.ok(
            build.route === "wasm" || build.route === "native",
            "integration build must route through VCC host",
        );

        const artifactDir = resolveArtifactDir(outRoot, NODE_WASM_TARGET);
        assert.ok(existsSync(artifactDir), `missing artifact dir under ${outRoot}`);

        const entry = resolveNodeEntry(artifactDir);
        assert.ok(entry, `no node entry under ${artifactDir}`);
        assert.equal(entry.physicalEntry, "legion.mjs");
        assert.equal(entry.legacy, false);

        assert.ok(existsSync(join(artifactDir, "run-contracts.txt")));
        const contracts = readFileSync(join(artifactDir, "run-contracts.txt"), "utf8");
        assert.match(contracts, /physical_entry:\s*"legion\.mjs"/);
        assert.ok(!existsSync(join(artifactDir, "legion_tools.mjs")));

        const runBuilt = spawnBuiltNodeEntry(entry.legionMjs, []);
        assert.equal(runBuilt.status, 0, formatOutcome("node legion.mjs", runBuilt));
    } finally {
        rmSync(outRoot, { recursive: true, force: true });
    }
});

test("wasm collect path: spawnCli --version when assembled", (t) => {
    if (!wasmCollectReady(WASM_COLLECT, WASM_ENTRY)) {
        t.skip("wasm collect not assembled");
        return;
    }
    const outcome = host.spawnCli(["--version"]);
    assert.equal(outcome.route, "wasm");
    assert.equal(outcome.status, 0, formatOutcome("legion --version", outcome));
});
