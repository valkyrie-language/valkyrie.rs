import assert from "node:assert/strict";
import test from "node:test";

import { createHostRunner } from "../src/index.ts";
import { wasmCollectReady } from "../src/testing.ts";

test("createHostRunner exposes spawnCli", () => {
    const host = createHostRunner({
        wasmCollect: "@valkyrie-language/vcc-unknown-wasm32",
        wasmEntry: "legion.mjs",
    });

    assert.equal(typeof host.spawnCli, "function");
    assert.equal(typeof host.runCli, "function");
});

test("spawnCli captures stdio when wasm collect is assembled", (t) => {
    if (!wasmCollectReady("@valkyrie-language/vcc-unknown-wasm32", "legion.mjs")) {
        t.skip("wasm collect not assembled");
        return;
    }

    const host = createHostRunner({
        wasmCollect: "@valkyrie-language/vcc-unknown-wasm32",
        wasmEntry: "legion.mjs",
    });

    const outcome = host.spawnCli(["--version"]);
    assert.equal(outcome.route, "wasm");
    assert.equal(typeof outcome.status, "number");
    assert.equal(typeof outcome.stdout, "string");
    assert.equal(typeof outcome.stderr, "string");
});
