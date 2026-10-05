import { spawnSync } from 'node:child_process';
/** 组装 CLI 宿主配置：唯一的 Node/Wasm collect 入口。 */
export type VccHostConfig = {
    wasmCollect: string;
    wasmEntry: string;
};

/** `legion spy` 目标参数（与 Rust `SpyTargetRunOptions` 对齐）。 */
export type VccSpyTargetRunOptions = {
    input?: string;
    func?: string;
    method?: string;
    offset?: number;
    list?: boolean;
    context?: number;
    targetPlatform?: string;
    json?: boolean;
    hex?: boolean;
    types?: boolean;
    gcAudit?: boolean;
    glueAudit?: boolean;
};

/** `spawnCli` 路由结果。 */
export type VccCliRoute = 'wasm';

/** 可测试的 CLI 子进程结果（不调用 `process.exit`）。 */
export type VccCliSpawnResult = {
    route: VccCliRoute;
    status: number;
    stdout: string;
    stderr: string;
};

/** 由 `createHostRunner` 返回的组装 CLI 宿主。 */
export type VccHostRunner = {
    config: Readonly<Required<VccHostConfig>>;
    resolveWasmMjs: () => string;
    spawnCli: (argv?: string[]) => VccCliSpawnResult;
    spawnSpy: (mode: string, options?: VccSpyTargetRunOptions) => VccCliSpawnResult;
    runCli: (argv?: string[]) => never;
};

/**
 * 解析 Wasm collect 入口脚本的绝对路径。
 *
 * @param wasmCollect
 * @param wasmEntry
 */
export function resolveWasmMjs(wasmCollect: string, wasmEntry: string): string {
    const pkgJson = require.resolve(join(wasmCollect, 'package.json'));
    return join(dirname(pkgJson), wasmEntry);
}

/** 创建唯一的 Node/Wasm collect 宿主。 */
export function createHostRunner(config: VccHostConfig): VccHostRunner {
    const resolved: Required<VccHostConfig> = {
        wasmCollect: config.wasmCollect,
        wasmEntry: config.wasmEntry,
    };

    const resolveWasm = () => resolveWasmMjs(resolved.wasmCollect, resolved.wasmEntry);

    function spawnCli(argv: string[] = []): VccCliSpawnResult {
        const mjs = resolveWasm();
        const result = spawnSync(process.execPath, [mjs, ...argv], { encoding: 'utf8' });
        return {
            route: 'wasm',
            status: result.status ?? 1,
            stdout: String(result.stdout ?? ''),
            stderr: String(result.stderr ?? ''),
        };
    }

    function spawnSpy(mode: string, options: VccSpyTargetRunOptions = {}): VccCliSpawnResult {
        const argv = ['spy', mode];
        if (options.input) {
            argv.push(options.input);
        }
        if (options.func) {
            argv.push('--func', options.func);
        }
        if (options.method) {
            argv.push('--method', options.method);
        }
        if (options.offset !== undefined) {
            argv.push('--offset', String(options.offset));
        }
        if (options.list) {
            argv.push('--list');
        }
        if (options.context !== undefined) {
            argv.push('--context', String(options.context));
        }
        if (options.targetPlatform) {
            argv.push('--target', options.targetPlatform);
        }
        if (options.json) {
            argv.push('--json');
        }
        if (options.hex) {
            argv.push('--hex');
        }
        if (options.types) {
            argv.push('--types');
        }
        if (options.gcAudit) {
            argv.push('--gc-audit');
        }
        if (options.glueAudit) {
            argv.push('--glue-audit');
        }
        return spawnCli(argv);
    }

    function runCli(argv: string[] = process.argv.slice(2)): never {
        const outcome = spawnCli(argv);
        process.exit(outcome.status);
    }

    return {
        config: resolved,
        resolveWasmMjs: resolveWasm,
        spawnCli,
        spawnSpy,
        runCli,
    };
}
