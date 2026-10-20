import { createRequire } from 'node:module';

import { createCli } from '@vmz/commander';

import { ASGARD_WASM_COLLECT, ASGARD_WASM_ENTRY } from '@valkyrie-language/vcc/asgard';

import { ASGARD_CLI_CATALOG } from './catalog.ts';
import { spawnCli } from '../spawn.ts';

const require = createRequire(import.meta.url);
const pkg = require('../../package.json') as { version: string };

function buildCli() {
    const cli = createCli('asgard').catalog(() => ASGARD_CLI_CATALOG).intro('cli.intro');

    cli.command('version', 'cli.cmd.version').action(() => {
        console.log(`@valkyrie-language/asgard ${pkg.version}`);
        console.log(`route wasm (${ASGARD_WASM_COLLECT})`);
        return 0;
    });

    cli.command('host-info', 'cli.cmd.host-info').action(() => {
        console.log(
            JSON.stringify(
                {
                    version: pkg.version,
                    wasmCollect: ASGARD_WASM_COLLECT,
                    wasmEntry: ASGARD_WASM_ENTRY,
                    route: 'wasm',
                },
                null,
                2,
            ),
        );
        return 0;
    });

    return cli;
}

function forwardWasm(argv: string[]): number {
    const outcome = spawnCli(argv);
    process.stdout.write(outcome.stdout);
    process.stderr.write(outcome.stderr);
    return outcome.status;
}

/** 解析 argv 并执行 Asgard CLI（commander 编排；wasm collect 经 @valkyrie-language/vcc）。 */
export async function runCli(argv: string[] = process.argv.slice(2)): Promise<number> {
    const [cmd] = argv;
    if (cmd && cmd !== 'version' && cmd !== 'host-info' && cmd !== 'help' && cmd !== '-h' && cmd !== '--help') {
        return forwardWasm(argv);
    }
    return buildCli().parse(argv);
}
