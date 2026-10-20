import type { VccCliSpawnResult } from './spawn-wasm.ts';

/** 集成测试用：仅要求能按 argv 路由到 native / wasm collect。 */
export type CliHostRunner = {
    spawnCli(argv?: string[]): VccCliSpawnResult;
};
