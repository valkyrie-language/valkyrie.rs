import assert from "node:assert/strict";
import test from "node:test";

import {
    aggregateLegionBenchRows,
    benchmarkSync,
    createBenchmarkRunner,
    median,
    parseLegionBenchTable,
    runBenchmarkEntry,
    runBenchmarkSuite,
} from "../src/benchmark.ts";

test("median returns middle sample", () => {
    assert.equal(median([3, 1, 2]), 2);
    assert.equal(median([1, 2, 3, 4]), 2.5);
});

test("benchmarkSync measures synchronous work", () => {
    let counter = 0;
    const result = benchmarkSync(
        () => {
            counter += 1;
        },
        { iterations: 10, warmup: 2 },
    );
    assert.equal(counter, 12);
    assert.equal(result.iterations, 10);
    assert.ok(result.medianMs >= 0);
});

test("parseLegionBenchTable reads bench stdout table", () => {
    const stdout = `
基准结果（3 次运行）：
------------------------------------------------------------------------
项目                 测试             目标       编译(ms)   运行(ms)
------------------------------------------------------------------------
fibonacci            bench_fib        node          12.3       4.5
------------------------------------------------------------------------
`;
    const rows = parseLegionBenchTable(stdout);
    assert.equal(rows.length, 1);
    assert.deepEqual(rows[0], {
        project: "fibonacci",
        test: "bench_fib",
        target: "node",
        compileMs: 12.3,
        runtimeMs: 4.5,
    });
});

test("aggregateLegionBenchRows averages compile and runtime", () => {
    const aggregate = aggregateLegionBenchRows([
        { project: "a", test: "t1", target: "node", compileMs: 10, runtimeMs: 4 },
        { project: "a", test: "t2", target: "node", compileMs: 20, runtimeMs: 6 },
    ]);
    assert.deepEqual(aggregate, { compileMs: 15, runtimeMs: 5, rowCount: 2 });
});

test("compareReference computes runtime ratio", () => {
    const runner = createBenchmarkRunner({ valkyrieRsRoot: process.cwd() });
    const comparison = runner.compareReference(10, {
        outcome: { route: "native", status: 0, stdout: "", stderr: "" },
        rows: [],
        aggregate: { compileMs: 1, runtimeMs: 5, rowCount: 1 },
    });
    assert.equal(comparison.runtimeRatio, 2);
    assert.equal(comparison.legionRoute, "native");
    assert.equal(comparison.error, null);
});

test("runBenchmarkEntry merges reference and legion fields", async () => {
    const runner = createBenchmarkRunner({ valkyrieRsRoot: process.cwd() });
    const row = await runBenchmarkEntry(
        {
            ...runner,
            ready: () => true,
            benchProject: () => ({
                outcome: { route: "native", status: 0, stdout: "", stderr: "" },
                rows: [],
                aggregate: { compileMs: 2, runtimeMs: 4, rowCount: 1 },
            }),
            compareReference: runner.compareReference,
            skipReason: () => null,
        },
        {
            id: "demo",
            title: "Demo",
            projectDir: "/tmp/demo",
            measureReference: () => 8,
        },
    );
    assert.equal(row.referenceMs, 8);
    assert.equal(row.legionRuntimeMs, 4);
    assert.equal(row.runtimeRatio, 2);
    assert.equal(row.error, null);
});

test("runBenchmarkSuite reports skip when runner not ready", async () => {
    const runner = createBenchmarkRunner({ valkyrieRsRoot: process.cwd() });
    const report = await runBenchmarkSuite(
        {
            ...runner,
            ready: () => false,
            skipReason: () => "not ready",
            benchProject: runner.benchProject,
            compareReference: runner.compareReference,
        },
        [{ id: "a", projectDir: "/tmp/a" }],
    );
    assert.equal(report.ready, false);
    assert.equal(report.rows.length, 1);
    assert.match(report.rows[0].error ?? "", /not ready/);
});
