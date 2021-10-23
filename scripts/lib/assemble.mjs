/**
 * Assemble projects/packages/vcc-unknown-wasm32 from legion Node build output.
 */

import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, normalize, resolve, sep } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const PACKAGE_DIR = join(REPO_ROOT, 'projects', 'packages', 'vcc-unknown-wasm32');
const MIN_WASM_BYTES = 1024;
const FORBIDDEN_FROM_FRAGMENTS = [`${sep}smoke-out`, `${sep}smoke-legion`, `${sep}smoke-`, `${sep}generate-seed`, 'placeholder-minimal-wasm'];

function fail(message) {
    process.stderr.write(`build assemble: ${message}\n`);
    process.exit(1);
}

function parseArgs(argv) {
    const out = {
        from: null,
        vCommit: null,
        sourceProject: null,
        dryRun: false,
        allowFixture: false,
    };
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (arg === '--from') {
            out.from = argv[++i];
            continue;
        }
        if (arg === '--v-commit') {
            out.vCommit = argv[++i];
            continue;
        }
        if (arg === '--source-project') {
            out.sourceProject = argv[++i];
            continue;
        }
        if (arg === '--dry-run') {
            out.dryRun = true;
            continue;
        }
        if (arg === '--allow-fixture') {
            out.allowFixture = true;
            continue;
        }
        if (arg === '--help' || arg === '-h') {
            process.stdout.write('Usage: node scripts/build.mjs assemble --from <dir> --source-project <dir> [--v-commit <sha>] [--dry-run]\n');
            process.exit(0);
        }
        if (arg === 'assemble') continue;
        fail(`unknown argument: ${arg}`);
    }
    return out;
}

function resolvePath(raw, label) {
    if (!raw) fail(`missing ${label}`);
    return isAbsolute(raw) ? raw : resolve(process.cwd(), raw);
}

function findWasm(artifactDir) {
    for (const name of ['legion.wasm', 'legion.tools.wasm']) {
        const path = join(artifactDir, name);
        if (existsSync(path)) return { path, name };
    }
    fail(`no legion.wasm under ${artifactDir}`);
}

function requireFile(artifactDir, name) {
    const path = join(artifactDir, name);
    if (!existsSync(path)) fail(`missing ${name} under ${artifactDir}`);
    return path;
}

function sha256File(path) {
    return createHash('sha256').update(readFileSync(path)).digest('hex');
}

function sha256Tree(root) {
    const files = [];
    function walk(dir) {
        for (const name of readdirSync(dir, { withFileTypes: true })) {
            if (name.name === '.git' || name.name === 'node_modules' || name.name === 'dist') continue;
            const p = join(dir, name.name);
            if (name.isDirectory()) walk(p);
            else files.push(p);
        }
    }
    walk(root);
    files.sort();
    const h = createHash('sha256');
    for (const file of files) {
        const rel = file.slice(root.length).split(sep).join('/');
        h.update(rel);
        h.update('\0');
        h.update(readFileSync(file));
        h.update('\0');
    }
    return { digest: h.digest('hex'), fileCount: files.length };
}

function gitOutput(args) {
    const result = spawnSync('git', args, { cwd: REPO_ROOT, encoding: 'utf8' });
    if (result.status !== 0) return null;
    return (result.stdout || '').trim() || null;
}

function toolVersion(command, args) {
    const result = spawnSync(command, args, { encoding: 'utf8' });
    if (result.status !== 0) return null;
    return (result.stdout || result.stderr || '').trim().split('\n')[0] || null;
}

function portablePath(absPath, ...roots) {
    const normalized = normalize(absPath);
    for (const root of roots) {
        if (!root) continue;
        const rootNorm = normalize(root);
        const prefix = rootNorm.endsWith(sep) ? rootNorm : rootNorm + sep;
        if (normalized === rootNorm) return '.';
        if (normalized.toLowerCase().startsWith(prefix.toLowerCase())) {
            return normalized.slice(prefix.length).split(sep).join('/');
        }
    }
    const parts = normalized.split(/[/\\]/).filter(Boolean);
    const idx = parts.findIndex((p) => p === 'valkyrie.v' || p === 'valkyrie.rs' || p === 'dist' || p === 'packages');
    if (idx >= 0) return parts.slice(idx).join('/');
    fail(`refusing non-portable path in provenance: ${absPath}`);
}

function readPackageVersion() {
    const pkgPath = join(PACKAGE_DIR, 'package.json');
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
    return pkg.version;
}

function assertNotFixturePath(artifactDir, allowFixture) {
    const normalized = normalize(artifactDir).toLowerCase();
    for (const frag of FORBIDDEN_FROM_FRAGMENTS) {
        if (!normalized.includes(frag.toLowerCase())) continue;
        if (allowFixture) {
            process.stderr.write(`build assemble: WARNING --allow-fixture: accepting ${frag}\n`);
            return;
        }
        fail(`refusing --from ${artifactDir}: fixture path (${frag})`);
    }
}

function assertSourceProject(sourceProject) {
    if (!existsSync(sourceProject)) fail(`--source-project not found: ${sourceProject}`);
    if (!existsSync(join(sourceProject, 'legion.von'))) {
        fail(`missing legion.von under ${sourceProject}`);
    }
}

function assertWasmArtifacts(wasmPath, contractsText, allowFixture) {
    const size = statSync(wasmPath).size;
    if (size < MIN_WASM_BYTES) fail(`legion.wasm is ${size} bytes (< ${MIN_WASM_BYTES})`);
    if (/placeholder-minimal-wasm/i.test(contractsText)) fail('run-contracts.txt is placeholder-minimal-wasm');
    if (!allowFixture && /smoke/i.test(contractsText)) {
        fail('run-contracts.txt mentions fixture marker; use --allow-fixture for local only');
    }
    if (!/physical_entry:\s*"legion\.mjs"/i.test(contractsText)) {
        fail('run-contracts.txt must declare physical_entry: "legion.mjs"');
    }
}

/** @param {string[]} argv */
export function runAssemble(argv) {
    const args = parseArgs(argv);
    const artifactDir = resolvePath(args.from, '--from <artifact-dir>');
    if (!existsSync(artifactDir)) fail(`artifact dir not found: ${artifactDir}`);
    assertNotFixturePath(artifactDir, args.allowFixture);

    const sourceProject = resolvePath(args.sourceProject, '--source-project <dir>');
    assertSourceProject(sourceProject);
    const sourceClosure = sha256Tree(sourceProject);
    const sourceProjectPortable = portablePath(sourceProject, REPO_ROOT, dirname(REPO_ROOT));

    const wasm = findWasm(artifactDir);
    const contractsText = readFileSync(requireFile(artifactDir, 'run-contracts.txt'), 'utf8');
    assertWasmArtifacts(wasm.path, contractsText, args.allowFixture);

    const version = readPackageVersion();

    const rustCommit = gitOutput(['rev-parse', 'HEAD']);
    if (!rustCommit) fail('git rev-parse HEAD failed');

    if (args.dryRun) {
        process.stdout.write(
            `${JSON.stringify(
                {
                    packageDir: 'projects/packages/vcc-unknown-wasm32',
                    from: portablePath(artifactDir, REPO_ROOT),
                    sourceProject: sourceProjectPortable,
                    sourceClosure,
                    wasmBytes: statSync(wasm.path).size,
                    version,
                    rustCommit,
                },
                null,
                2,
            )}\n`,
        );
        return;
    }

    mkdirSync(PACKAGE_DIR, { recursive: true });
    copyFileSync(wasm.path, join(PACKAGE_DIR, 'legion.wasm'));
    copyFileSync(requireFile(artifactDir, 'legion.mjs'), join(PACKAGE_DIR, 'legion.mjs'));
    copyFileSync(requireFile(artifactDir, 'run-contracts.txt'), join(PACKAGE_DIR, 'run-contracts.txt'));
    const licenseSrc = join(REPO_ROOT, 'LICENSE.md');
    const licenseDest = join(PACKAGE_DIR, 'LICENSE.md');
    if (existsSync(licenseSrc) && !existsSync(licenseDest)) copyFileSync(licenseSrc, licenseDest);

    const digests = {
        'legion.wasm': sha256File(join(PACKAGE_DIR, 'legion.wasm')),
        'legion.mjs': sha256File(join(PACKAGE_DIR, 'legion.mjs')),
        'run-contracts.txt': sha256File(join(PACKAGE_DIR, 'run-contracts.txt')),
    };

    const provenance = {
        package: '@valkyrie-language/vcc-unknown-wasm32',
        version,
        target: 'wasm32-node-unknown-wasm',
        rust_repo: 'valkyrie.rs',
        rust_commit: rustCommit,
        valkyrie_v_commit: args.vCommit ?? null,
        source_project: sourceProjectPortable,
        source_closure: { digest: sourceClosure.digest, file_count: sourceClosure.fileCount },
        toolchain: {
            rustc: toolVersion('rustc', ['--version']),
            cargo: toolVersion('cargo', ['--version']),
            node: toolVersion('node', ['--version']),
        },
        build: {
            from: portablePath(artifactDir, REPO_ROOT),
            locked: true,
            assembled_at: new Date().toISOString(),
            fixture: args.allowFixture === true,
        },
        artifacts: digests,
    };

    writeFileSync(join(PACKAGE_DIR, 'provenance.json'), `${JSON.stringify(provenance, null, 2)}\n`, 'utf8');
    digests['provenance.json'] = sha256File(join(PACKAGE_DIR, 'provenance.json'));
    const sumLines = Object.entries(digests)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([name, hash]) => `${hash}  ${name}`);
    writeFileSync(join(PACKAGE_DIR, 'SHA256SUMS'), `${sumLines.join('\n')}\n`, 'utf8');

    process.stdout.write(`assembled projects/packages/vcc-unknown-wasm32 (${digests['legion.wasm'].slice(0, 12)}…)\n`);
}
