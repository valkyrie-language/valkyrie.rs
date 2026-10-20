import { loadNativeBinding, resolveWasmMjs, spyRun, type VccSpyTargetRunOptions } from '../native.ts';
import {
    legionBuildOutcomeToSpawnResult,
    legionPlanOutcomeToSpawnResult,
    spawnWasmCollect,
    type VccCliSpawnResult,
} from '../spawn-wasm.ts';
import { LEGION_WASM_COLLECT, LEGION_WASM_ENTRY } from '../collects.ts';

export { LEGION_WASM_COLLECT, LEGION_WASM_ENTRY };

/** 仍由 wasm collect 实现的子命令（commander 未覆盖时显式转发）。 */
export const LEGION_WASM_ONLY_COMMANDS = new Set(['bench', 'spy', 'test']);

/** 解析 Legion wasm 入口绝对路径。 */
export function resolveLegionWasmMjs(): string {
    return resolveWasmMjs(LEGION_WASM_COLLECT, LEGION_WASM_ENTRY);
}

/** 在 Legion wasm collect 上执行 argv。 */
export function spawnLegionWasm(argv: string[] = []): VccCliSpawnResult {
    return spawnWasmCollect(LEGION_WASM_COLLECT, LEGION_WASM_ENTRY, argv);
}

function parseFlagValue(argv: string[], flag: string): string | undefined {
    const idx = argv.indexOf(flag);
    if (idx < 0) {
        return undefined;
    }
    return argv[idx + 1];
}

/** Legion CLI：native N-API 优先，否则 wasm collect 回退。 */
export function spawnLegionCli(argv: string[] = []): VccCliSpawnResult {
    const cmd = argv[0];
    const binding = loadNativeBinding();

    if (cmd === 'plan' && binding?.legionPlan) {
        const projectDir = argv[1];
        if (!projectDir) {
            return { route: 'native', status: 1, stdout: '', stderr: 'missing project directory' };
        }
        const target = parseFlagValue(argv, '--target');
        return legionPlanOutcomeToSpawnResult(binding.legionPlan({ projectDir, target }));
    }

    if (cmd === 'build' && binding?.legionBuild) {
        const projectDir = argv[1];
        if (!projectDir) {
            return { route: 'native', status: 1, stdout: '', stderr: 'missing project directory' };
        }
        const target = parseFlagValue(argv, '--target');
        const outputDir = parseFlagValue(argv, '-o') ?? parseFlagValue(argv, '--out-dir');
        return legionBuildOutcomeToSpawnResult(binding.legionBuild({ projectDir, target, outputDir }));
    }

    return spawnLegionWasm(argv);
}

/** `legion spy`：native 优先，否则 wasm。 */
export function spawnLegionSpy(mode: string, options: VccSpyTargetRunOptions = {}): VccCliSpawnResult {
    const native = spyRun(mode, options);
    if (native) {
        return {
            route: 'native',
            status: native.status,
            stdout: native.stdout,
            stderr: native.stderr,
        };
    }
    const argv = ['spy', mode];
    if (options.input) {
        argv.push(options.input);
    }
    return spawnLegionWasm(argv);
}
