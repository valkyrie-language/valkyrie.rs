import { createRequire } from 'node:module';

import type { Cli, ParsedOptions } from '@vmz/commander';
import { createCli } from '@vmz/commander';

import {
    buildProject,
    legionBuildOutcomeToSpawnResult,
    legionPlanOutcomeToSpawnResult,
    locateNativeCollect,
    planProject,
} from '@valkyrie-language/vcc';
import {
    LEGION_WASM_COLLECT,
    LEGION_WASM_ENTRY,
    LEGION_WASM_ONLY_COMMANDS,
} from '@valkyrie-language/vcc/legion';

import { LEGION_CLI_CATALOG } from './catalog.ts';
import { spawnCli, spawnWasmCli } from '../spawn.ts';

const require = createRequire(import.meta.url);
const pkg = require('../../package.json') as { version: string };

function buildCli(): Cli {
    const cli = createCli('legion').catalog(() => LEGION_CLI_CATALOG).intro('cli.intro');

    cli.command('version', 'cli.cmd.version').action(() => {
        const native = locateNativeCollect();
        console.log(`@valkyrie-language/legion ${pkg.version}`);
        console.log(`route ${native ? 'native' : 'wasm'} (${native ?? LEGION_WASM_COLLECT})`);
        return 0;
    });

    cli.command('host-info', 'cli.cmd.host-info').action(() => {
        const native = locateNativeCollect();
        console.log(
            JSON.stringify(
                {
                    version: pkg.version,
                    wasmCollect: LEGION_WASM_COLLECT,
                    wasmEntry: LEGION_WASM_ENTRY,
                    nativeCollect: native,
                },
                null,
                2,
            ),
        );
        return 0;
    });

    cli.command('plan', 'cli.cmd.plan')
        .option('--target <id>', 'cli.opt.target')
        .action((options: ParsedOptions) => {
            const projectDir = options._[0];
            if (!projectDir) {
                console.error('legion plan: missing project directory');
                return 1;
            }
            const target = typeof options.target === 'string' ? options.target : undefined;
            const native = planProject(projectDir, target);
            const outcome = native
                ? legionPlanOutcomeToSpawnResult(native)
                : spawnWasmCli(['plan', projectDir, ...(target ? ['--target', target] : [])]);
            process.stdout.write(outcome.stdout);
            process.stderr.write(outcome.stderr);
            return outcome.status;
        });

    cli.command('build', 'cli.cmd.build')
        .option('--target <id>', 'cli.opt.target')
        .option('--out-dir, -o <dir>', 'cli.opt.out-dir')
        .action((options: ParsedOptions) => {
            const projectDir = options._[0];
            if (!projectDir) {
                console.error('legion build: missing project directory');
                return 1;
            }
            const target = typeof options.target === 'string' ? options.target : undefined;
            const outputDir = typeof options['out-dir'] === 'string' ? options['out-dir'] : undefined;
            const native = buildProject(projectDir, target, outputDir);
            const outcome = native
                ? legionBuildOutcomeToSpawnResult(native)
                : spawnWasmCli([
                      'build',
                      projectDir,
                      ...(target ? ['--target', target] : []),
                      ...(outputDir ? ['-o', outputDir] : []),
                  ]);
            process.stdout.write(outcome.stdout);
            process.stderr.write(outcome.stderr);
            return outcome.status;
        });

    return cli;
}

function forwardWasm(argv: string[]): number {
    const outcome = spawnWasmCli(argv);
    process.stdout.write(outcome.stdout);
    process.stderr.write(outcome.stderr);
    return outcome.status;
}

/** 解析 argv 并执行 Legion CLI（commander 编排；collect 路由经 @valkyrie-language/vcc）。 */
export async function runCli(argv: string[] = process.argv.slice(2)): Promise<number> {
    const [cmd] = argv;
    if (cmd && LEGION_WASM_ONLY_COMMANDS.has(cmd)) {
        return forwardWasm(argv);
    }
    return buildCli().parse(argv);
}
