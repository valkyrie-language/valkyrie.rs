# @valkyrie-language/vcc

Shared **Valkyrie Compiler Collect (VCC)** orthogonal native N-API bindings and wasm spawn helpers.

This package is a small library used by [`@valkyrie-language/legion`](../legion) and [`@valkyrie-language/asgard`](../asgard).
It exposes programmatic `planProject` / `buildProject` / `spyRun` (native) and `spawnWasmCollect` (wasm). CLI orchestration
lives in the product packages via `@vmz/commander`, not here.

## Install

```bash
npm install @valkyrie-language/vcc
```

Most applications should install a CLI package instead:

```bash
npm install @valkyrie-language/legion
# or
npm install @valkyrie-language/asgard
```

## Requirements

- **Node.js** 20 or newer

## Usage

```js
import {
    buildProject,
    locateNativeCollect,
    planProject,
    spawnWasmCollect,
} from "@valkyrie-language/vcc";

// Native N-API (when a platform collect is installed)
const plan = planProject("./my-project", "wasm32-node-unknown-wasm");
const build = buildProject("./my-project", "wasm32-node-unknown-wasm", "./dist");

// Wasm collect spawn (orthogonal argv → wasm script)
const outcome = spawnWasmCollect(
    "@valkyrie-language/vcc-unknown-wasm32",
    "legion.mjs",
    ["build", "./my-project"],
);

locateNativeCollect(); // path to vcc.<platform>.node or null
```

### Benchmark (`@valkyrie-language/vcc/benchmark`)

对比参考实现（TypeScript 等同算法）与 `legion bench -t node`（V 编译为 Wasm + Wasm 运行）：

```js
import {benchmarkSync, createBenchmarkRunner, WASM_NODE_BENCH_TARGET} from "@valkyrie-language/vcc/benchmark";

const runner = createBenchmarkRunner({
    // 本机检出根：环境变量或相对路径；禁止写死本机绝对路径
    valkyrieRsRoot: process.env.VALKYRIE_RS_ROOT ?? process.cwd(),
});

// TS 侧：benchmarkSync 包裹参考函数
const reference = benchmarkSync(() => solveInTypeScript(), {iterations: 2000});

// V 侧：legion bench -t node → compileMs + runtimeMs（Wasm 入口执行 [benchmark]）
const legion = runner.benchProject("./projects/problems/two-sum/solvers/valkyrie/two_sum", {
    runs: 3,
    target: WASM_NODE_BENCH_TARGET,
});
const comparison = runner.compareReference(reference.medianMs, legion);
```

| Export | Description |
|--------|-------------|
| `median` / `benchmarkSync` | 参考实现 median 毫秒采样 |
| `parseLegionBenchTable` / `aggregateLegionBenchRows` | 解析 `legion bench` stdout |
| `createBenchmarkRunner` | native / wasm Legion 路由 + `benchProject` + `compareReference` |
| `runBenchmarkEntry` / `runBenchmarkSuite` | catalog 批量对比（参考实现 + Legion） |

批量跑题集（如 leetcode / project-euler）：

```js
import {createBenchmarkRunner, runBenchmarkSuite} from "@valkyrie-language/vcc/benchmark";

const runner = createBenchmarkRunner({valkyrieRsRoot: process.env.VALKYRIE_RS_ROOT});
const report = runBenchmarkSuite(runner, catalog.map((p) => ({
    id: p.id,
    title: p.title,
    projectDir: p.path,
    measureReference: () => benchPython(p),
})), {runs: 3, target: "node"});
```

### Exports

| Export                           | Description                                                    |
|----------------------------------|----------------------------------------------------------------|
| `NATIVE_PACKAGES`                | Optional native collect package names (Windows, Linux, macOS). |
| `locateNativeCollect(packages?)` | Find an installed platform `.node` addon, or `null`.           |
| `loadNativeBinding()`            | Load native N-API binding object, or `null`.                   |
| `planProject` / `buildProject`   | Programmatic Legion plan/build via native N-API.               |
| `spyRun`                         | Programmatic `legion spy` via native N-API.                    |
| `spawnWasmCollect`               | Spawn wasm collect entry with argv (no CLI aggregation).       |
| `resolveWasmMjs(collect, entry)` | Resolve the Wasm host script path inside a collect package.    |
| `@valkyrie-language/vcc/legion`  | Legion wasm/native spawn routing (product CLI uses this).        |
| `@valkyrie-language/vcc/asgard`  | Asgard wasm spawn routing (product CLI uses this).             |

## Related packages

| Package                     | Role                                                |
|-----------------------------|-----------------------------------------------------|
| `@valkyrie-language/legion` | Valkyrie compiler CLI (`@vmz/commander`); depends only on `vcc` |
| `@valkyrie-language/asgard` | GUI app CLI (`@vmz/commander`); depends only on `vcc`             |
| `@valkyrie-language/vcc-*`  | Platform collect packages (optional deps of `vcc`, not product CLIs) |

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)
