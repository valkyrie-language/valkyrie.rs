#!/usr/bin/env node
/**
 * `valkyrie-napi` 原生绑定构建入口。
 *
 * 只构建 `projects/napi/valkyrie-napi` 并把产物复制到
 * `projects/ts/platforms/valkyrie-<short>/lib/valkyrie.<triple>.node`。
 *
 * Usage:
 *   node scripts/build.mjs napi
 *   node scripts/build.mjs napi --debug
 *   node scripts/build.mjs napi --all
 */

import {spawnSync} from 'node:child_process';
import {copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/**
 * 先让 cargo 结算依赖图，再读 Cargo.lock。
 *
 * `NYAR_COMPILER_BUILD_ID` 必须在 legion 编译前注入，而 `Cargo.lock` 只有在
 * cargo 解析依赖后才会写入新 pin。直接读锁会拿到上一次构建的 rev（本轮已发生
 * 的内嵌身份错位）：addon 是新的，身份却是旧的。`cargo metadata` 在这里完成
 * 锁更新，失败即终止，绝不带着待定身份继续编译。
 */
function syncCargoLock() {
    const result = spawnSync('cargo', ['metadata', '--format-version', '1', '--quiet'], {
        cwd: ROOT,
        stdio: 'inherit',
        shell: process.platform === 'win32',
    });
    if ((result.status ?? 1) !== 0) {
        console.error('cargo metadata 失败：无法结算 Cargo.lock，拒绝以过期构建身份编译');
        process.exit(result.status ?? 1);
    }
}

/** 从 Cargo.toml 读取声明的 Boost `package-manager-legion` rev。 */
function declaredBoostRev() {
    const manifest = readFileSync(join(ROOT, 'Cargo.toml'), 'utf8');
    const entry = manifest.match(/^package-manager-legion\s*=\s*\{[^}]*rev\s*=\s*"([0-9a-f]+)"/m);
    return entry?.[1];
}

/**
 * 从已结算的 Cargo.lock 解析 Boost `package-manager-legion` 的固定 rev，作为
 * `NYAR_COMPILER_BUILD_ID` 注入编译环境：git 依赖 checkout 没有 `.git`，
 * legion 侧 `option_env!` 否则退化为 `unknown`，native lane 的构建身份门
 * 会拒绝正式复用。注入后 addon 内嵌身份与产物清单 `compiler=` 段同源一致。
 */
function boostBuildId() {
    if (process.env.NYAR_COMPILER_BUILD_ID) return process.env.NYAR_COMPILER_BUILD_ID;
    syncCargoLock();
    const lock = readFileSync(join(ROOT, 'Cargo.lock'), 'utf8');
    const entry = lock.match(/\[\[package\]\]\nname = "package-manager-legion"\nversion = "[^"]*"\nsource = "git\+https:\/\/github\.com\/oovm\/rust-boost\?rev=([0-9a-f]+)#([0-9a-f]+)"/);
    if (!entry) {
        console.warn('warning: Cargo.lock 未含 package-manager-legion 的 git rev 条目，编译器构建身份将退化为 unknown');
        return undefined;
    }
    const locked = entry[2];
    const declared = declaredBoostRev();
    if (declared && declared !== locked) {
        console.error(`Cargo.lock rev ${locked.slice(0, 12)} 与 Cargo.toml 声明 ${declared.slice(0, 12)} 不一致：拒绝注入错位身份`);
        process.exit(1);
    }
    return locked;
}

/** @typedef {{ triple: string, packageDir: string, npmTriple: string, short: string }} NativeTarget */

const NATIVE_TARGETS = [
    {triple: 'x86_64-pc-windows-msvc', packageDir: 'valkyrie-win32-x64', npmTriple: 'win32-x64-msvc', short: 'win32-x64'},
    {triple: 'x86_64-unknown-linux-gnu', packageDir: 'valkyrie-linux-x64', npmTriple: 'linux-x64-gnu', short: 'linux-x64'},
    {triple: 'x86_64-apple-darwin', packageDir: 'valkyrie-darwin-x64', npmTriple: 'darwin-x64', short: 'darwin-x64'},
    {triple: 'aarch64-apple-darwin', packageDir: 'valkyrie-darwin-arm64', npmTriple: 'darwin-arm64', short: 'darwin-arm64'},
];

function parseArgs(argv) {
    const flags = argv.filter((arg) => arg.startsWith('-'));
    const positionals = argv.filter((arg) => !arg.startsWith('-'));
    return {
        mode: positionals.find((arg) => arg === 'napi') ?? 'napi',
        all: flags.includes('--all'),
        debug: flags.includes('--debug'),
    };
}

function run(command, args, extraEnv) {
    const result = spawnSync(command, args, {
        cwd: ROOT,
        stdio: 'inherit',
        shell: process.platform === 'win32',
        env: {...process.env, ...extraEnv},
    });
    if ((result.status ?? 1) !== 0) {
        process.exit(result.status ?? 1);
    }
}

function hostTriple() {
    if (process.platform === 'win32') return 'x86_64-pc-windows-msvc';
    if (process.platform === 'darwin') return process.arch === 'arm64' ? 'aarch64-apple-darwin' : 'x86_64-apple-darwin';
    return process.arch === 'arm64' ? 'aarch64-unknown-linux-gnu' : 'x86_64-unknown-linux-gnu';
}

function selectTargets(targets, all) {
    if (all) return targets;
    const forced = process.env.VALKYRIE_NATIVE_TARGET;
    if (forced) {
        const hit = targets.find((target) => target.short === forced || target.triple === forced);
        if (hit) return [hit];
        console.warn(`VALKYRIE_NATIVE_TARGET=${forced} did not match any native target; falling back to host selection`);
    }
    const host = hostTriple();
    const exact = targets.find((target) => target.triple === host);
    return exact ? [exact] : [targets[0]];
}

function nativeLibCandidates(triple, release, libName) {
    const profile = release ? 'release' : 'debug';
    const dir = join(ROOT, 'target', triple, profile);
    if (triple.includes('windows')) return [join(dir, `${libName}.dll`)];
    if (triple.includes('apple')) return [join(dir, `lib${libName}.dylib`)];
    return [join(dir, `lib${libName}.so`), join(dir, `${libName}.so`)];
}

function findNativeArtifact(triple, release, libName) {
    const candidates = nativeLibCandidates(triple, release, libName);
    for (const candidate of candidates) {
        if (existsSync(candidate)) return candidate;
    }
    const profile = release ? 'release' : 'debug';
    const depsDir = join(ROOT, 'target', triple, profile, 'deps');
    if (existsSync(depsDir)) {
        for (const name of readdirSync(depsDir)) {
            if (
                (name === libName || name.startsWith(`${libName}.`) || name.startsWith(`lib${libName}.`)) &&
                (name.endsWith('.dll') || name.endsWith('.so') || name.endsWith('.dylib') || name.endsWith('.node'))
            ) {
                return join(depsDir, name);
            }
        }
    }
    throw new Error(`missing build artifact (tried: ${candidates.join(', ')})`);
}

function buildNapi(opts) {
    const release = !opts.debug;
    const targets = selectTargets(NATIVE_TARGETS, opts.all);
    const buildId = boostBuildId();
    const extraEnv = buildId ? {NYAR_COMPILER_BUILD_ID: buildId} : {};
    console.log(`build:napi → valkyrie-napi cdylib (${release ? 'release' : 'debug'}, ${targets.length} target(s))`);
    if (buildId) {
        console.log(`compiler build id ← Cargo.lock rev ${buildId.slice(0, 12)}`);
        // option_env! 不参与 cargo 指纹，身份变化必须强制重编 legion 及其下游。
        run('cargo', ['clean', '-p', 'package-manager-legion']);
    }

    for (const target of targets) {
        const triple = process.env.VALKYRIE_NATIVE_TARGET_TRIPLE ?? target.triple;
        const profile = release ? ['--release'] : [];
        console.log(`\n→ ${triple} → projects/ts/platforms/${target.packageDir}`);
        run('cargo', ['build', ...profile, '--target', triple, '-p', 'valkyrie-napi'], extraEnv);

        const source = findNativeArtifact(triple, release, 'valkyrie_napi');
        const destination = join(ROOT, 'projects', 'ts', 'platforms', target.packageDir, 'lib', `valkyrie.${target.npmTriple}.node`);
        mkdirSync(dirname(destination), {recursive: true});
        copyFileSync(source, destination);
        console.log(`  copied lib/valkyrie.${target.npmTriple}.node`);
    }
}

const opts = parseArgs(process.argv.slice(2));
if (opts.mode === 'napi') {
    buildNapi(opts);
} else {
    console.error(`unsupported mode '${opts.mode}' (only 'napi' is published from this repository)`);
    process.exit(1);
}

console.log('\nbuild complete');
