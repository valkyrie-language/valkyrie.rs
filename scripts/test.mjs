#!/usr/bin/env node
/**
 *   node scripts/test.mjs release
 */

import { spawnSync } from 'node:child_process';

const argv = process.argv.slice(2);
const command = argv[0] ?? 'release';

function run(cmd, args) {
    const r = spawnSync(cmd, args, { stdio: 'inherit', shell: process.platform === 'win32' });
    if ((r.status ?? 1) !== 0) process.exit(r.status ?? 1);
}

if (command === 'release') {
    process.env.LEGION_INTEGRATION = '1';
    run('pnpm', ['test']);
    run('pnpm', ['test:integration']);
} else {
    console.error(`test: unknown command \`${command}\``);
    process.exit(1);
}
