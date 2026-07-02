#!/usr/bin/env node
/**
 * `@valkyrie-language/legion` CLI entry.
 *
 * Loads package-local `legion.wasm` and dispatches --version / --help / build.
 * Does not read Rust source, Cargo target/, or LEGION_SEED host bridges.
 */

import { loadLegionModule } from "../lib/load-wasm.mjs";
import { dispatchCli } from "../lib/cli.mjs";

async function main() {
  try {
    const { exports, host } = await loadLegionModule();
    const code = dispatchCli(exports, host, process.argv.slice(2));
    process.exit(code);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`legion: ${message}\n`);
    process.exit(1);
  }
}

await main();
