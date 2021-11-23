#!/usr/bin/env node
/**
 * 本地复刻 `.github/workflows/rust.yml` 门禁，单轮跑完并汇总全部失败项。
 *
 *   node scripts/ci-local.mjs
 *   node scripts/ci-local.mjs
 *   node scripts/ci-local.mjs --coverage-only
 *   node scripts/ci-local.mjs --legion-only
 *   node scripts/ci-local.mjs --regenerate-fixtures
 *
 * `--regenerate-fixtures` 会设置 `LEGION_TEST_REGENERATE=1` 并重跑 run/runtime/oop fixture 测试。
 * 默认只验证，不写 sidecar。
 */

import { spawnSync } from 'node:child_process';

const argv = process.argv.slice(2);
const coverageOnly = argv.includes('--coverage-only');
const legionOnly = argv.includes('--legion-only');
const regenerateFixtures = argv.includes('--regenerate-fixtures');

if (regenerateFixtures) {
    process.env.LEGION_TEST_REGENERATE = '1';
}

const coveragePackages = legionOnly
    ? ['legion']
    : ['legion', 'vcc-data', 'vcc-napi', 'vcc-wasm'];

/** @type {{ title: string, cmd: string, args: string[] }[]} */
const steps = coverageOnly || legionOnly
    ? [
          {
              title: `coverage (llvm-cov: ${coveragePackages.join(', ')})`,
              cmd: 'cargo',
              args: ['llvm-cov', ...coveragePackages.flatMap((pkg) => ['-p', pkg]), '--no-report'],
          },
      ]
    : [
          {
              title: 'build release crates',
              cmd: 'cargo',
              args: ['build', '--release', '-p', 'vcc-napi', '-p', 'legion', '-p', 'vcc-data', '-p', 'vcc-wasm', '-p', 'asgard'],
          },
          {
              title: 'legion artifact_formats',
              cmd: 'cargo',
              args: ['test', '--release', '-p', 'legion', '--test', 'artifact_formats'],
          },
          {
              title: 'legion bootstrap_node_entry',
              cmd: 'cargo',
              args: ['test', '-p', 'legion', '--test', 'bootstrap_node_entry', '--release'],
          },
          {
              title: `coverage (llvm-cov: ${coveragePackages.join(', ')})`,
              cmd: 'cargo',
              args: ['llvm-cov', ...coveragePackages.flatMap((pkg) => ['-p', pkg]), '--no-report'],
          },
      ];

/** @type {{ title: string, status: number | null, tail: string }[]} */
const failures = [];

for (const step of steps) {
    process.stdout.write(`\n==> ${step.title}\n`);
    const result = spawnSync(step.cmd, step.args, {
        encoding: 'utf8',
        shell: process.platform === 'win32',
        maxBuffer: 64 * 1024 * 1024,
    });
    const combined = `${result.stdout ?? ''}${result.stderr ?? ''}`;
    const status = result.status;
    if (status !== 0) {
        const lines = combined.trim().split(/\r?\n/);
        const highlights = lines.filter((line) =>
            /^(test .+ \.\.\. FAILED|failures:|thread '.+' panicked at|assertion .+ failed|error: test failed)/.test(line),
        );
        const tail = highlights.length > 0
            ? highlights.slice(0, 40).join('\n')
            : lines.slice(-40).join('\n');
        failures.push({ title: step.title, status, tail });
        process.stderr.write(`FAILED (${status ?? 'signal'})\n`);
    } else {
        process.stdout.write('ok\n');
    }
}

if (failures.length === 0) {
    console.log('\nci-local: all steps passed');
    process.exit(0);
}

console.error(`\nci-local: ${failures.length} step(s) failed\n`);
for (const [index, failure] of failures.entries()) {
    console.error(`--- failure ${index + 1}: ${failure.title} (exit ${failure.status}) ---`);
    console.error(failure.tail);
    console.error('');
}
process.exit(1);
