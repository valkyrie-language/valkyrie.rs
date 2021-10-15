# @valkyrie-language/vcc

Shared **Valkyrie Compiler Collect (VCC) host runner** for assembled Node.js CLIs.

This package is a small library used by [`@valkyrie-language/legion`](../legion) and [
`@valkyrie-language/asgard`](../asgard). It resolves platform-specific native artifacts and WebAssembly collect entries,
then dispatches CLI execution (native first, Wasm fallback).

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
import {createHostRunner, NATIVE_PACKAGES} from "@valkyrie-language/vcc";

const host = createHostRunner({
    wasmCollect: "@valkyrie-language/vcc-unknown-wasm32",
    wasmEntry: "legion.mjs",
});

// Programmatic entry (same routing as the `legion` bin)
host.runCli(process.argv.slice(2));

// Inspect routing
host.locateNativeCollect(); // path to vcc.<platform>.node or null
host.resolveWasmMjs();      // absolute path to legion.mjs in the wasm collect
```

### Benchmark (`@valkyrie-language/vcc/benchmark`)

对比参考实现（TypeScript 等同算法）与 `legion bench -t node`（V 编译为 Wasm + Wasm 运行）：

```js
import {benchmarkSync, createBenchmarkRunner, WASM_NODE_BENCH_TARGET} from "@valkyrie-language/vcc/benchmark";

const runner = createBenchmarkRunner({
    valkyrieRsRoot: "E:/victory 胜利女神/valkyrie.rs",
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
| `locateNativeCollect(packages?)` | Find an installed platform `.node` addon, or `null`.                |
| `resolveWasmMjs(collect, entry)` | Resolve the Wasm host script path inside a collect package.    |
| `createHostRunner(config)`       | Bind wasm collect settings and return `runCli` helpers.        |

## Related packages

| Package                     | Role                                                |
|-----------------------------|-----------------------------------------------------|
| `@valkyrie-language/legion` | Valkyrie compiler & workspace CLI                   |
| `@valkyrie-language/asgard` | Cross-platform GUI app framework CLI                |
| `@valkyrie-language/vcc-*`  | Platform collect packages (native / Wasm artifacts) |

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)
