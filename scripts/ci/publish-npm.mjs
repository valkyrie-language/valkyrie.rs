/**
 * GitHub Actions: publish @valkyrie-language/legion (OIDC Trusted Publisher).
 *
 * - tag v0.0.5 → version 0.0.5 only (this seed line)
 * - Idempotent: skip when version already on registry
 * - No NPM_TOKEN; permissions.id-token: write + env NPM_PUBLISH
 * - Contract: file=publish-npm.yml env=NPM_PUBLISH repo=valkyrie-language/valkyrie.rs
 *
 * Prereq: packages/legion-wasm already assembled (real legion.wasm + legion.mjs).
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const PACKAGE_DIR = 'packages/legion-wasm';
const FIXED_VERSION = '0.0.5';
const MIN_WASM_BYTES = 1024;

function fail(msg) {
  console.error(`ci-publish-npm: ${msg}`);
  process.exit(1);
}

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    cwd: opts.cwd ?? ROOT,
    encoding: 'utf8',
    shell: process.platform === 'win32',
    env: opts.env ?? process.env,
    stdio: opts.stdio ?? 'pipe',
  });
  return {
    status: r.status ?? 1,
    stdout: String(r.stdout ?? '').trim(),
    stderr: String(r.stderr ?? '').trim(),
  };
}

function resolveVersion() {
  const fromArg = process.argv.find((a) => a.startsWith('--version='))?.slice('--version='.length);
  if (fromArg) return fromArg.replace(/^v/, '');
  const ref = process.env.GITHUB_REF ?? '';
  const m = ref.match(/^refs\/tags\/v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/);
  if (m) return m[1];
  fail('need --version=0.0.5 or GITHUB_REF=refs/tags/v0.0.5');
}

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function writeJson(p, obj) {
  fs.writeFileSync(p, `${JSON.stringify(obj, null, 2)}\n`);
}

function copyTree(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const name of fs.readdirSync(src)) {
    if (name === 'node_modules' || name === '.git') continue;
    const from = path.join(src, name);
    const to = path.join(dest, name);
    const st = fs.statSync(from);
    if (st.isDirectory()) copyTree(from, to);
    else {
      fs.mkdirSync(path.dirname(to), { recursive: true });
      fs.copyFileSync(from, to);
    }
  }
}

function isAlreadyPublished(blob) {
  return /cannot publish over existing|EPUBLISHCONFLICT|previously published versions|version already exists|cannot publish.*same version|you cannot publish over/i.test(
    blob,
  );
}

function isAuthFailure(blob) {
  return /ENEEDAUTH|Unable to authenticate|not authorized|OIDC|trusted publisher|two-factor|need to be logged|login|identity token|do not have permission to access it|Access token expired or revoked/i.test(
    blob,
  );
}

function versionExists(name, version) {
  const r = run('npm', ['view', `${name}@${version}`, 'version']);
  return r.status === 0 && r.stdout === version;
}

/**
 * @returns {'published'|'exists'|'auth'|'other'}
 */
function npmPublish(stagingDir, name, version) {
  const args = ['publish', '--access', 'public'];
  console.log(`\n=== ${name}@${version} npm ${args.join(' ')} ===`);
  const r = run('npm', args, { cwd: stagingDir });
  if (r.stdout) process.stdout.write(`${r.stdout}\n`);
  if (r.stderr) process.stderr.write(`${r.stderr}\n`);
  const blob = `${r.stdout}\n${r.stderr}`;
  if (r.status === 0) return 'published';
  if (isAlreadyPublished(blob) || versionExists(name, version)) return 'exists';
  if (isAuthFailure(blob)) return 'auth';
  if (versionExists(name, version)) return 'exists';
  console.error(blob.slice(0, 1200));
  return 'other';
}

function assertAssembled(abs) {
  const pkg = readJson(path.join(abs, 'package.json'));
  if (pkg.name !== '@valkyrie-language/legion') fail(`unexpected package name ${pkg.name}`);
  if (pkg.version !== FIXED_VERSION) {
    fail(`package.json version must be ${FIXED_VERSION} (found ${pkg.version})`);
  }
  for (const f of ['legion.wasm', 'legion.mjs', 'run-contracts.txt', 'provenance.json']) {
    if (!fs.existsSync(path.join(abs, f))) fail(`missing assembled file ${f}`);
  }
  const wasmBytes = fs.statSync(path.join(abs, 'legion.wasm')).size;
  if (wasmBytes < MIN_WASM_BYTES) {
    fail(`legion.wasm is ${wasmBytes} bytes; refusing placeholder`);
  }
  const contracts = fs.readFileSync(path.join(abs, 'run-contracts.txt'), 'utf8');
  if (/placeholder-minimal-wasm/i.test(contracts)) {
    fail('run-contracts.txt is placeholder; refusing publish');
  }
  if (!/physical_entry:\s*"legion\.mjs"/i.test(contracts)) {
    fail('run-contracts.txt missing physical_entry: "legion.mjs"');
  }
  return pkg;
}

function publishLegion(version) {
  if (version !== FIXED_VERSION) {
    fail(`only ${FIXED_VERSION} may be published on this seed line (got ${version})`);
  }

  const abs = path.join(ROOT, PACKAGE_DIR);
  if (!fs.existsSync(abs)) fail(`missing ${PACKAGE_DIR}`);
  const raw = assertAssembled(abs);
  const name = raw.name;

  if (versionExists(name, version)) {
    console.log(` ✓ ${name}@${version} already on registry — skip`);
    return { published: 0, skipped: 1 };
  }

  const stage = path.join(os.tmpdir(), `valkyrie-legion-${version}`);
  fs.rmSync(stage, { recursive: true, force: true });
  fs.mkdirSync(stage, { recursive: true });

  const files = Array.isArray(raw.files) && raw.files.length ? raw.files : null;
  if (files) {
    for (const f of files) {
      const from = path.join(abs, f);
      if (!fs.existsSync(from)) continue;
      const st = fs.statSync(from);
      const to = path.join(stage, f);
      if (st.isDirectory()) copyTree(from, to);
      else {
        fs.mkdirSync(path.dirname(to), { recursive: true });
        fs.copyFileSync(from, to);
      }
    }
    for (const extra of ['package.json', 'README.md', 'LICENSE.md', 'LICENSE']) {
      const from = path.join(abs, extra);
      if (!fs.existsSync(from)) continue;
      fs.copyFileSync(from, path.join(stage, extra));
    }
  } else {
    copyTree(abs, stage);
  }

  const pkg = { ...raw };
  pkg.version = version;
  delete pkg.private;
  pkg.publishConfig = { ...(pkg.publishConfig ?? {}), access: 'public' };
  if (!pkg.repository) {
    pkg.repository = {
      type: 'git',
      url: 'git+https://github.com/valkyrie-language/valkyrie.rs.git',
      directory: PACKAGE_DIR,
    };
  }
  delete pkg.devDependencies;
  writeJson(path.join(stage, 'package.json'), pkg);

  if (!fs.existsSync(path.join(stage, 'README.md'))) {
    fs.writeFileSync(path.join(stage, 'README.md'), `# ${name}\n\nLegion Node/Wasm seed ${version}.\n`);
  }

  const outcome = npmPublish(stage, name, version);
  if (outcome === 'published') return { published: 1, skipped: 0 };
  if (outcome === 'exists') {
    console.log(` ✓ ${name}@${version} already on registry — skip`);
    return { published: 0, skipped: 1 };
  }
  if (outcome === 'auth') {
    fail(
      `OIDC/auth failed for ${name}. Add Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=valkyrie-language/valkyrie.rs`,
    );
  }
  fail(`publish failed for ${name}@${version}`);
}

const version = resolveVersion();
console.log(`ci-publish-npm: version=${version}`);
console.log(` GITHUB_REF=${process.env.GITHUB_REF ?? '(none)'}`);
console.log(' Trusted Publisher contract: publish-npm.yml + env NPM_PUBLISH\n');

delete process.env.NODE_AUTH_TOKEN;
delete process.env.NPM_TOKEN;

const result = publishLegion(version);
console.log(`\nci-publish-npm: done (published=${result.published} skipped=${result.skipped})`);
