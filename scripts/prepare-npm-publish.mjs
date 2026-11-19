#!/usr/bin/env node
/**
 * 规范化 npm 发布输入：确保每个包在 `readme.md` 旁存在 `README.md`。
 *
 * Usage:
 *   node scripts/prepare-npm-publish.mjs
 *   node scripts/prepare-npm-publish.mjs --check
 */

import {copyFileSync, existsSync, readdirSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const TS_ROOT = join(ROOT, 'projects', 'ts');
const CHECK_ONLY = process.argv.includes('--check');

function packageDirs() {
    /** @type {string[]} */
    const dirs = [];
    for (const bucket of ['emulators', 'platforms']) {
        const base = join(TS_ROOT, bucket);
        if (!existsSync(base)) continue;
        for (const entry of readdirSync(base, {withFileTypes: true})) {
            if (!entry.isDirectory()) continue;
            const dir = join(base, entry.name);
            if (existsSync(join(dir, 'package.json'))) {
                dirs.push(dir);
            }
        }
    }
    return dirs;
}

let missing = 0;

for (const dir of packageDirs()) {
    const readme = join(dir, 'readme.md');
    const published = join(dir, 'README.md');
    if (!existsSync(readme)) {
        console.error(`missing readme.md: ${dir}`);
        missing++;
        continue;
    }
    if (CHECK_ONLY) {
        if (!existsSync(published)) {
            console.error(`missing README.md (run prepare-npm-publish): ${dir}`);
            missing++;
        }
        continue;
    }
    copyFileSync(readme, published);
    console.log(`prepared ${published}`);
}

if (missing > 0) {
    process.exit(1);
}
