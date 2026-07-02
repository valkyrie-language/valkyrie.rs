#!/usr/bin/env node
/**
 * Minimal Node CLI seed wasm for @valkyrie-language/legion package smoke tests.
 */

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const outDir = process.argv[2]
  ? isAbsolute(process.argv[2])
    ? process.argv[2]
    : join(process.cwd(), process.argv[2])
  : join(scriptDir, "..", "packages", "legion-wasm");

function uleb128(value) {
  const bytes = [];
  let n = value >>> 0;
  do {
    let byte = n & 0x7f;
    n >>>= 7;
    if (n !== 0) byte |= 0x80;
    bytes.push(byte);
  } while (n !== 0);
  return bytes;
}

function vec(items) {
  return [...uleb128(items.length), ...items.flat()];
}

function section(id, payload) {
  return [id, ...uleb128(payload.length), ...payload];
}

function exportEntry(name, kind, index) {
  const nameBytes = [...name].map((c) => c.charCodeAt(0));
  return [...uleb128(nameBytes.length), ...nameBytes, kind, ...uleb128(index)];
}

// One function type: () -> i32
const typePayload = vec([[0x60, 0x00, 0x01, 0x7f]]);
const typeSection = section(1, typePayload);

// Three functions, all type index 0
const functionPayload = vec([[0], [0], [0]]);
const functionSection = section(3, functionPayload);

const exportPayload = vec([
  exportEntry("help", 0x00, 0),
  exportEntry("version", 0x00, 1),
  exportEntry("build", 0x00, 2),
]);
const exportSection = section(7, exportPayload);

// Each body: local count 0, i32.const 0, end
const funcBody = [0x00, 0x41, 0x00, 0x0b];
const codePayload = vec([
  [...uleb128(funcBody.length), ...funcBody],
  [...uleb128(funcBody.length), ...funcBody],
  [...uleb128(funcBody.length), ...funcBody],
]);
const codeSection = section(10, codePayload);

const bytes = new Uint8Array([
  0x00,
  0x61,
  0x73,
  0x6d,
  0x01,
  0x00,
  0x00,
  0x00,
  ...typeSection,
  ...functionSection,
  ...exportSection,
  ...codeSection,
]);

mkdirSync(outDir, { recursive: true });
const wasmPath = join(outDir, "legion.wasm");
writeFileSync(wasmPath, bytes);

writeFileSync(
  join(outDir, "run-contracts.txt"),
  [
    'physical_entry: "bin/legion.mjs"',
    'wasm_module: "legion.wasm"',
    "seed: placeholder-minimal-wasm",
    "exports: help,version,build",
  ].join("\n") + "\n",
  "utf8",
);

// Validate in Node before writing provenance
const module = await WebAssembly.compile(bytes);
const instance = await WebAssembly.instantiate(module, { env: {} });
for (const name of ["help", "version", "build"]) {
  if (typeof instance.exports[name] !== "function") {
    throw new Error(`missing export ${name}`);
  }
}

process.stdout.write(`wrote ${wasmPath} (${bytes.length} bytes, validated)\n`);
