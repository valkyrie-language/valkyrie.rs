#!/usr/bin/env node
/**
 * Publish @valkyrie-language/legion from packages/legion-wasm.
 *
 * Auth (never commit secrets):
 *   - valkyrie.rs/.env.placeholder.local  (NPM_TOTP_SECRET / NPM_TOKEN / NPM_OTP)
 *   - or vos-language/.env.placeholder.local when VALKYRIE_NPM_ENV is unset
 *
 * Usage:
 *   node scripts/publish-legion-wasm.mjs
 *   node scripts/publish-legion-wasm.mjs --otp 123456
 */

import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(SCRIPT_DIR, "..");
const PACKAGE_DIR = path.join(REPO_ROOT, "packages", "legion-wasm");

function fail(message) {
  process.stderr.write(`publish-legion-wasm: ${message}\n`);
  process.exit(1);
}

function loadLocalEnv(filePath) {
  /** @type {Record<string, string>} */
  const out = {};
  try {
    for (const raw of fs.readFileSync(filePath, "utf8").split(/\r?\n/)) {
      const line = raw.trim();
      if (!line || line.startsWith("#")) continue;
      const m = line.match(/^(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/);
      if (!m) continue;
      let v = m[2].trim();
      if (
        (v.startsWith('"') && v.endsWith('"')) ||
        (v.startsWith("'") && v.endsWith("'"))
      ) {
        v = v.slice(1, -1);
      }
      out[m[1]] = v;
    }
  } catch {
    /* optional */
  }
  return out;
}

function resolveEnvFile() {
  if (process.env.VALKYRIE_NPM_ENV) return process.env.VALKYRIE_NPM_ENV;
  const local = path.join(REPO_ROOT, ".env.placeholder.local");
  if (fs.existsSync(local)) return local;
  const vmzRoots = fs.readdirSync("E:\\", { withFileTypes: true }).filter((d) => d.isDirectory() && d.name.startsWith("vmz"));
  for (const root of vmzRoots) {
    const candidate = path.join("E:\\", root.name, "vos-language", ".env.placeholder.local");
    if (fs.existsSync(candidate)) return candidate;
  }
  return local;
}

/** @param {string[]} args @param {string} flag */
function takeFlag(args, flag) {
  const i = args.indexOf(flag);
  if (i >= 0 && args[i + 1] && !args[i + 1].startsWith("-")) return args[i + 1];
  const eq = args.find((a) => a.startsWith(`${flag}=`));
  if (eq) return eq.slice(flag.length + 1);
  return undefined;
}

/** @param {string} secret */
function decodeBase32(secret) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  const cleaned = secret.replace(/[\s=-]/g, "").toUpperCase();
  let bits = "";
  for (const ch of cleaned) {
    const v = alphabet.indexOf(ch);
    if (v < 0) fail("invalid base32 in TOTP secret");
    bits += v.toString(2).padStart(5, "0");
  }
  const bytes = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) {
    bytes.push(Number.parseInt(bits.slice(i, i + 8), 2));
  }
  if (!bytes.length) fail("TOTP secret decoded empty");
  return Buffer.from(bytes);
}

function totpCode(secret, atMs = Date.now()) {
  const key = decodeBase32(secret);
  const counter = Math.floor(atMs / 1000 / 30);
  const buf = Buffer.alloc(8);
  buf.writeUInt32BE(Math.floor(counter / 0x100000000), 0);
  buf.writeUInt32BE(counter & 0xffffffff, 4);
  const hmac = crypto.createHmac("sha1", key).update(buf).digest();
  const offset = hmac[hmac.length - 1] & 0x0f;
  const code =
    ((hmac[offset] & 0x7f) << 24) |
    ((hmac[offset + 1] & 0xff) << 16) |
    ((hmac[offset + 2] & 0xff) << 8) |
    (hmac[offset + 3] & 0xff);
  return String(code % 1_000_000).padStart(6, "0");
}

function runNpm(args, token) {
  /** @type {NodeJS.ProcessEnv} */
  const env = { ...process.env };
  let userConfig;
  if (token) {
    userConfig = path.join(os.tmpdir(), `valkyrie-legion-npmrc-${process.pid}`);
    fs.writeFileSync(userConfig, `//registry.npmjs.org/:_authToken=${token}\n`, "utf8");
    env.NPM_CONFIG_USERCONFIG = userConfig;
    env.NODE_AUTH_TOKEN = token;
  }
  try {
    const r = spawnSync("npm", args, {
      cwd: PACKAGE_DIR,
      encoding: "utf8",
      shell: process.platform === "win32",
      stdio: "pipe",
      env,
    });
    return {
      status: r.status ?? 1,
      stdout: String(r.stdout ?? ""),
      stderr: String(r.stderr ?? ""),
    };
  } finally {
    if (userConfig) {
      try {
        fs.unlinkSync(userConfig);
      } catch {
        /* ignore */
      }
    }
  }
}

function main() {
  const rest = process.argv.slice(2);
  const envFile = resolveEnvFile();
  const localEnv = loadLocalEnv(envFile);
  const token =
    takeFlag(rest, "--token") ??
    process.env.NPM_TOKEN ??
    localEnv.NPM_TOKEN ??
    localEnv.TOKEN;
  const otpFlag =
    takeFlag(rest, "--otp") ??
    process.env.NPM_OTP ??
    localEnv.NPM_OTP ??
    localEnv.OTP;
  const totpSecretRaw =
    takeFlag(rest, "--totp-secret") ??
    process.env.NPM_TOTP_SECRET ??
    localEnv.NPM_TOTP_SECRET ??
    localEnv.TOTP_SECRET ??
    (otpFlag && !/^\d{6}$/.test(otpFlag.trim()) ? otpFlag : undefined);
  const otp =
    (otpFlag && /^\d{6}$/.test(otpFlag.trim()) ? otpFlag.trim() : undefined) ??
    (totpSecretRaw ? totpCode(totpSecretRaw) : undefined);

  if (!fs.existsSync(path.join(PACKAGE_DIR, "package.json"))) {
    fail(`missing ${PACKAGE_DIR}/package.json`);
  }
  if (!fs.existsSync(path.join(PACKAGE_DIR, "legion.wasm"))) {
    fail("missing legion.wasm — run generate-seed-wasm + assemble-legion-wasm first");
  }

  const pkg = JSON.parse(fs.readFileSync(path.join(PACKAGE_DIR, "package.json"), "utf8"));
  process.stdout.write(`publish ${pkg.name}@${pkg.version} (env: ${envFile})\n`);

  const view = runNpm(["view", `${pkg.name}@${pkg.version}`, "version"], token);
  if (view.status === 0 && view.stdout.trim() === pkg.version) {
    process.stdout.write(`already published ${pkg.name}@${pkg.version}\n`);
    return;
  }

  const args = ["publish", "--access", "public"];
  if (otp) args.push(`--otp=${otp}`);
  else if (!token) {
    fail("need NPM_TOTP_SECRET in env file or --otp / NPM_TOKEN for publish");
  }

  const r = runNpm(args, token);
  if (r.stdout) process.stdout.write(r.stdout);
  if (r.stderr) process.stderr.write(r.stderr);
  if (r.status !== 0) fail(`npm publish failed (status ${r.status})`);

  process.stdout.write(`published ${pkg.name}@${pkg.version}\n`);
}

main();
