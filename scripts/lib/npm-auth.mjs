/**
 * Shared npm auth (.env.npm-trust.local: NPM_TOTP_SECRET, NPM_OTP, NPM_TOKEN).
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
export const ENV_PATH = path.join(REPO_ROOT, '.env.npm-trust.local');

export function loadLocalEnv(filePath = ENV_PATH) {
    const out = {};
    try {
        for (const raw of fs.readFileSync(filePath, 'utf8').split(/\r?\n/)) {
            const line = raw.trim();
            if (!line || line.startsWith('#')) continue;
            const m = line.match(/^(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/);
            if (!m) continue;
            let v = m[2].trim();
            if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
                v = v.slice(1, -1);
            }
            out[m[1]] = v;
        }
    } catch {
        /* optional */
    }
    return out;
}

function decodeBase32(secret) {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    const cleaned = secret.replace(/[\s=-]/g, '').toUpperCase();
    let bits = '';
    for (const ch of cleaned) {
        const v = alphabet.indexOf(ch);
        if (v < 0) throw new Error('invalid base32 in NPM_TOTP_SECRET');
        bits += v.toString(2).padStart(5, '0');
    }
    const bytes = [];
    for (let i = 0; i + 8 <= bits.length; i += 8) {
        bytes.push(Number.parseInt(bits.slice(i, i + 8), 2));
    }
    if (!bytes.length) throw new Error('NPM_TOTP_SECRET decoded empty');
    return Buffer.from(bytes);
}

export function totpCode(secret, atMs = Date.now()) {
    const key = decodeBase32(secret);
    const counter = Math.floor(atMs / 1000 / 30);
    const buf = Buffer.alloc(8);
    buf.writeUInt32BE(Math.floor(counter / 0x100000000), 0);
    buf.writeUInt32BE(counter & 0xffffffff, 4);
    const hmac = crypto.createHmac('sha1', key).update(buf).digest();
    const offset = hmac[hmac.length - 1] & 0x0f;
    const code =
        ((hmac[offset] & 0x7f) << 24) | ((hmac[offset + 1] & 0xff) << 16) | ((hmac[offset + 2] & 0xff) << 8) | (hmac[offset + 3] & 0xff);
    return String(code % 1_000_000).padStart(6, '0');
}

/** @param {string[]} argv */
export function resolveNpmAuth(argv = []) {
    const localEnv = loadLocalEnv();
    const takeFlag = (flag) => {
        const i = argv.indexOf(flag);
        if (i >= 0 && argv[i + 1] && !argv[i + 1].startsWith('-')) return argv[i + 1];
        const eq = argv.find((a) => a.startsWith(`${flag}=`));
        if (eq) return eq.slice(flag.length + 1);
        return undefined;
    };

    const token = takeFlag('--token') ?? process.env.NPM_TOKEN ?? localEnv.NPM_TOKEN ?? localEnv.TOKEN;
    const otpFlag = takeFlag('--otp') ?? process.env.NPM_OTP ?? localEnv.NPM_OTP ?? localEnv.OTP;
    const totpSecretRaw =
        takeFlag('--totp-secret') ??
        process.env.NPM_TOTP_SECRET ??
        localEnv.NPM_TOTP_SECRET ??
        localEnv.TOTP_SECRET ??
        (otpFlag && !/^\d{6}$/.test(otpFlag.trim()) ? otpFlag : undefined);
    const otpStatic = otpFlag && /^\d{6}$/.test(otpFlag.trim()) ? otpFlag.trim() : undefined;

    return {
        token,
        totpSecretRaw,
        currentOtp() {
            if (totpSecretRaw) return totpCode(totpSecretRaw);
            return otpStatic;
        },
    };
}

/** @param {string[]} args @param {{ cwd?: string, token?: string, inherit?: boolean }} [opts] */
export function runNpm(args, opts = {}) {
    const env = { ...process.env };
    let userConfig;
    if (opts.token) {
        userConfig = path.join(os.tmpdir(), `valkyrie-npm-auth-${process.pid}`);
        fs.writeFileSync(userConfig, `//registry.npmjs.org/:_authToken=${opts.token}\n`, 'utf8');
        env.NPM_CONFIG_USERCONFIG = userConfig;
        env.NODE_AUTH_TOKEN = opts.token;
    }
    try {
        const r = spawnSync('npm', args, {
            cwd: opts.cwd,
            encoding: 'utf8',
            shell: process.platform === 'win32',
            stdio: opts.inherit ? 'inherit' : 'pipe',
            env,
        });
        return {
            status: r.status ?? 1,
            stdout: opts.inherit ? '' : String(r.stdout ?? '').trim(),
            stderr: opts.inherit ? '' : String(r.stderr ?? '').trim(),
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
