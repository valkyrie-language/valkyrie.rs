#!/usr/bin/env node
/**
 * 以本仓 Trusted Publisher 合同启动 `nifty`：
 * - 注入 `NIFTY_TRUST_REPO` / `NIFTY_TRUST_FILE` / `NIFTY_TRUST_ENV`
 * - 从 `.env.placeholder.local` 或兼容 `.env.npm-trust.local` 加载 NPM_* 密钥
 *
 *   node scripts/run-nifty.mjs trust [--dry-run] [--only @valkyrie-language/vcc]
 *   node scripts/run-nifty.mjs publish --dry-run
 */

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadLocalEnv } from './lib/npm-auth.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const TRUST_DEFAULTS = {
    NIFTY_TRUST_REPO: 'valkyrie-language/valkyrie.rs',
    NIFTY_TRUST_FILE: 'publish-npm.yml',
    NIFTY_TRUST_ENV: 'NPM_PUBLISH',
};

function applyLocalAuth(env) {
    const local = {
        ...loadLocalEnv(path.join(ROOT, '.env.placeholder.local')),
        ...loadLocalEnv(path.join(ROOT, '.env.npm-trust.local')),
    };
    for (const key of ['NPM_TOTP_SECRET', 'TOTP_SECRET', 'NPM_OTP', 'OTP', 'NPM_TOKEN', 'TOKEN']) {
        if (local[key] && !env[key]) {
            env[key] = local[key];
        }
    }
    // nifty 原生只读 `.env.placeholder.local`；若仅有旧文件名，把密钥透到 process env。
    if (!env.NPM_TOTP_SECRET && local.NPM_TOTP_SECRET) {
        env.NPM_TOTP_SECRET = local.NPM_TOTP_SECRET;
    }
}

const argv = process.argv.slice(2);
if (argv.length === 0 || argv[0] === '-h' || argv[0] === '--help') {
    console.log(`usage: node scripts/run-nifty.mjs <nifty-args...>

Examples:
  node scripts/run-nifty.mjs trust
  node scripts/run-nifty.mjs trust --dry-run
  node scripts/run-nifty.mjs trust --only @valkyrie-language/vcc
  node scripts/run-nifty.mjs publish --dry-run
`);
    process.exit(argv.length === 0 ? 1 : 0);
}

const env = { ...process.env, ...TRUST_DEFAULTS };
for (const [key, value] of Object.entries(TRUST_DEFAULTS)) {
    if (!process.env[key]) {
        env[key] = value;
    }
}
applyLocalAuth(env);

// 直接跑 cli 入口，避免 Windows 上对含空格路径的 `.cmd` + `shell: true` 拆词失败。
const niftyCli = path.join(ROOT, 'node_modules', '@doki-land', 'nifty', 'cli', 'nifty.mjs');
const result = spawnSync(process.execPath, [niftyCli, ...argv], {
    cwd: ROOT,
    env,
    stdio: 'inherit',
    shell: false,
});
process.exit(result.status ?? 1);
