#!/usr/bin/env node
/**
 * Assemble packages/legion-wasm from a Node artifact directory produced by:
 *   legion build <legion.tools> --target node -o <out>
 *
 * Usage (from valkyrie.rs root):
 *   node scripts/assemble-legion-wasm.mjs --from <artifact-dir> [--v-commit <sha>]
 *
 * Copies legion.mjs + legion.wasm + run-contracts.txt, writes provenance.json and
 * SHA256SUMS. Rejects placeholder / undersized wasm. Does not publish.
 */

import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");
const PACKAGE_DIR = join(REPO_ROOT, "packages", "legion-wasm");
/** Real seed wasm is far larger than the ~67B generate-seed-wasm stub. */
const MIN_WASM_BYTES = 1024;

function fail(message) {
  process.stderr.write(`assemble-legion-wasm: ${message}\n`);
  process.exit(1);
}

function parseArgs(argv) {
  const out = { from: null, vCommit: null, dryRun: false };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--from") {
      out.from = argv[++i];
      continue;
    }
    if (arg === "--v-commit") {
      out.vCommit = argv[++i];
      continue;
    }
    if (arg === "--dry-run") {
      out.dryRun = true;
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      process.stdout.write(
        "Usage: node scripts/assemble-legion-wasm.mjs --from <artifact-dir> [--v-commit <sha>] [--dry-run]\n",
      );
      process.exit(0);
    }
    fail(`unknown argument: ${arg}`);
  }
  return out;
}

function resolveFrom(raw) {
  if (!raw) fail("missing --from <artifact-dir>");
  return isAbsolute(raw) ? raw : resolve(process.cwd(), raw);
}

function findWasm(artifactDir) {
  const candidates = ["legion.wasm", "legion.tools.wasm"];
  for (const name of candidates) {
    const path = join(artifactDir, name);
    if (existsSync(path)) return { path, name };
  }
  fail(`no legion.wasm (or legion.tools.wasm) under ${artifactDir}`);
}

function requireFile(artifactDir, name) {
  const path = join(artifactDir, name);
  if (!existsSync(path)) fail(`missing required artifact ${name} under ${artifactDir}`);
  return path;
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function gitOutput(args) {
  const result = spawnSync("git", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  if (result.status !== 0) return null;
  return (result.stdout || "").trim() || null;
}

function toolVersion(command, args) {
  const result = spawnSync(command, args, { encoding: "utf8" });
  if (result.status !== 0) return null;
  return (result.stdout || result.stderr || "").trim().split("\n")[0] || null;
}

function readPackageVersion() {
  const pkgPath = join(PACKAGE_DIR, "package.json");
  if (!existsSync(pkgPath)) fail(`missing ${pkgPath}`);
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  return pkg.version;
}

function assertRealSeed(wasmPath, contractsText) {
  const size = statSync(wasmPath).size;
  if (size < MIN_WASM_BYTES) {
    fail(
      `legion.wasm is ${size} bytes (< ${MIN_WASM_BYTES}); refusing placeholder / generate-seed-wasm stub`,
    );
  }
  if (/placeholder-minimal-wasm/i.test(contractsText)) {
    fail('run-contracts.txt marks seed as placeholder-minimal-wasm; refusing assemble');
  }
  if (!/physical_entry:\s*"legion\.mjs"/i.test(contractsText)) {
    fail('run-contracts.txt must declare physical_entry: "legion.mjs"');
  }
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const artifactDir = resolveFrom(args.from);
  if (!existsSync(artifactDir)) fail(`artifact dir not found: ${artifactDir}`);

  const wasm = findWasm(artifactDir);
  if (wasm.name === "legion.tools.wasm") {
    process.stderr.write(
      "assemble-legion-wasm: warning: using legacy legion.tools.wasm; rename to legion.wasm in emitters\n",
    );
  }

  const mjsSrc = requireFile(artifactDir, "legion.mjs");
  const contractsSrc = requireFile(artifactDir, "run-contracts.txt");
  const contractsText = readFileSync(contractsSrc, "utf8");
  assertRealSeed(wasm.path, contractsText);

  const version = readPackageVersion();
  if (version !== "0.0.5") {
    fail(`package.json version must be 0.0.5 (found ${version}); refusing assemble`);
  }

  const rustCommit = gitOutput(["rev-parse", "HEAD"]);
  if (!rustCommit) fail("unable to resolve rust commit (git rev-parse HEAD)");

  const wasmDest = join(PACKAGE_DIR, "legion.wasm");
  const mjsDest = join(PACKAGE_DIR, "legion.mjs");
  const contractsDest = join(PACKAGE_DIR, "run-contracts.txt");
  const provenanceDest = join(PACKAGE_DIR, "provenance.json");
  const sumsDest = join(PACKAGE_DIR, "SHA256SUMS");
  const licenseSrc = join(REPO_ROOT, "LICENSE.md");
  const licenseDest = join(PACKAGE_DIR, "LICENSE.md");

  if (args.dryRun) {
    process.stdout.write(
      JSON.stringify(
        {
          packageDir: PACKAGE_DIR,
          from: artifactDir,
          wasm: wasm.path,
          mjs: mjsSrc,
          contracts: contractsSrc,
          wasmBytes: statSync(wasm.path).size,
          version,
          rustCommit,
        },
        null,
        2,
      ) + "\n",
    );
    return;
  }

  mkdirSync(PACKAGE_DIR, { recursive: true });
  copyFileSync(wasm.path, wasmDest);
  copyFileSync(mjsSrc, mjsDest);
  copyFileSync(contractsSrc, contractsDest);
  if (existsSync(licenseSrc) && !existsSync(licenseDest)) {
    copyFileSync(licenseSrc, licenseDest);
  }

  const digests = {
    "legion.wasm": sha256File(wasmDest),
    "legion.mjs": sha256File(mjsDest),
    "run-contracts.txt": sha256File(contractsDest),
  };

  const provenance = {
    package: "@valkyrie-language/legion",
    version,
    target: "wasm32-node-unknown-wasm",
    rust_repo: "valkyrie.rs",
    rust_commit: rustCommit,
    valkyrie_v_commit: args.vCommit ?? null,
    valkyrie_2020: null,
    toolchain: {
      rustc: toolVersion("rustc", ["--version"]),
      cargo: toolVersion("cargo", ["--version"]),
      node: toolVersion("node", ["--version"]),
    },
    build: {
      from: artifactDir,
      locked: true,
      assembled_at: new Date().toISOString(),
    },
    artifacts: digests,
    stage: "rust-seed",
    bootstrap_claim: "source-bootstrap-not-proven",
  };

  writeFileSync(provenanceDest, `${JSON.stringify(provenance, null, 2)}\n`, "utf8");
  digests["provenance.json"] = sha256File(provenanceDest);

  const sumLines = Object.entries(digests)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, hash]) => `${hash}  ${name}`);
  writeFileSync(sumsDest, `${sumLines.join("\n")}\n`, "utf8");

  process.stdout.write(
    `assembled ${PACKAGE_DIR}\n` +
      `  version: ${version}\n` +
      `  wasm:    ${digests["legion.wasm"]} (${statSync(wasmDest).size} bytes)\n` +
      `  next:    npm pack ./packages/legion-wasm\n`,
  );
}

main();
