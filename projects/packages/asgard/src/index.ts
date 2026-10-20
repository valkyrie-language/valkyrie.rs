import { runCli } from './cli/cli.ts';
import { resolveAsgardWasmMjs, spawnCli } from './spawn.ts';

export { runCli, spawnCli };

/** 解析 Asgard wasm 入口绝对路径。 */
export function resolveWasmMjs(): string {
    return resolveAsgardWasmMjs();
}
