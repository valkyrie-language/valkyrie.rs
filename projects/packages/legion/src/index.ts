import { buildProject, planProject, spyRun } from '@valkyrie-language/vcc';

import { runCli } from './cli/cli.ts';
import { resolveLegionWasmMjs, spawnCli, spawnSpy } from './spawn.ts';

export { buildProject, planProject, runCli, spawnCli, spawnSpy, spyRun };

/** 解析 Legion wasm 入口绝对路径。 */
export function resolveWasmMjs(): string {
    return resolveLegionWasmMjs();
}
