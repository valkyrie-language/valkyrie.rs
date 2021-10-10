import assert from "node:assert/strict";
import test from "node:test";

import { NATIVE_PACKAGES, createHostRunner, locateNativeCollect, resolveWasmMjs } from "../src/index.ts";

test("NATIVE_PACKAGES 覆盖四个 optional collect", () => {
    assert.deepEqual(NATIVE_PACKAGES, [
        "@valkyrie-language/vcc-win32-x64",
        "@valkyrie-language/vcc-linux-x64",
        "@valkyrie-language/vcc-darwin-arm64",
        "@valkyrie-language/vcc-darwin-x64",
    ]);
});

test("locateNativeCollect 不抛错（无 collect 时为 null）", () => {
    const path = locateNativeCollect();
    assert.ok(path === null || path.endsWith(".node") || path.includes("vcc_napi"));
});

test("resolveWasmMjs 解析 legion wasm collect", () => {
    const mjs = resolveWasmMjs("@valkyrie-language/vcc-unknown-wasm32", "legion.mjs");
    assert.ok(mjs.endsWith("legion.mjs"));
    assert.ok(mjs.includes("vcc-unknown-wasm32"));
});

test("resolveWasmMjs 解析 asgard wasm collect", () => {
    const mjs = resolveWasmMjs("@valkyrie-language/vcc-wasm32-wasi", "asgard.mjs");
    assert.ok(mjs.endsWith("asgard.mjs"));
    assert.ok(mjs.includes("vcc-wasm32-wasi"));
});

test("createHostRunner 绑定 wasm 配置", () => {
    const host = createHostRunner({
        wasmCollect: "@valkyrie-language/vcc-unknown-wasm32",
        wasmEntry: "legion.mjs",
    });
    assert.equal(host.config.wasmCollect, "@valkyrie-language/vcc-unknown-wasm32");
    assert.equal(host.config.wasmEntry, "legion.mjs");
    assert.equal(typeof host.runCli, "function");
    assert.equal(typeof host.locateNativeCollect, "function");
    assert.equal(typeof host.resolveWasmMjs, "function");
    assert.ok(host.resolveWasmMjs().endsWith("legion.mjs"));
});
