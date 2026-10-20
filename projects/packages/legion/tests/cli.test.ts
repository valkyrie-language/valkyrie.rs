import assert from 'node:assert/strict';
import test from 'node:test';

import { runCli } from '../src/cli/cli.ts';

test('runCli unknown command exits 1 via @vmz/commander', async () => {
    const code = await runCli(['no-such-command']);
    assert.equal(code, 1);
});

test('runCli help exits 0 without native collect', async () => {
    const code = await runCli(['help']);
    assert.equal(code, 0);
});
