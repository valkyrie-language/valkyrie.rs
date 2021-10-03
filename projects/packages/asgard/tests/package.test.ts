import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { NATIVE_PACKAGES, WASM_COLLECT, WASM_ENTRY, resolveWasmMjs, runCli } from "../src/index.ts";

const PACKAGE_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");

test("package.json 暴露 asgard bin 与 workspace 依赖", () => {
    const pkg = JSON.parse(readFileSync(join(PACKAGE_ROOT, "package.json"), "utf8"));
    assert.equal(pkg.name, "@valkyrie-language/asgard");
    assert.equal(pkg.bin.asgard, "bin/asgard.js");
    assert.equal(pkg.dependencies["@valkyrie-language/vcc"], "workspace:*");
    assert.equal(pkg.dependencies["@valkyrie-language/vcc-wasm32-wasi"], "workspace:*");
});

test("bin/asgard.js 存在且委托 src/index.ts", () => {
    const binPath = join(PACKAGE_ROOT, "bin", "asgard.js");
    assert.ok(existsSync(binPath));
    const source = readFileSync(binPath, "utf8");
    assert.match(source, /runCli/);
    assert.match(source, /\.\.\/src\/index\.ts/);
});

test("Asgard 宿主绑定 vcc-wasm32-wasi / asgard.mjs", () => {
    assert.equal(typeof runCli, "function");
    assert.equal(WASM_COLLECT, "@valkyrie-language/vcc-wasm32-wasi");
    assert.equal(WASM_ENTRY, "asgard.mjs");
    assert.ok(NATIVE_PACKAGES.length === 4);
    assert.ok(resolveWasmMjs().includes("vcc-wasm32-wasi"));
});
