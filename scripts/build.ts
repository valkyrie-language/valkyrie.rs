import { spawnSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { copyFileSync, mkdirSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

interface Target {
    target: string;
    pkgSuffix: string;
    binName: string;
    tool: 'cargo' | 'zigbuild';
    altBin?: string;
}

const commonTargets: Target[] = [
    { target: 'x86_64-pc-windows-msvc', pkgSuffix: 'win32-x64', binName: '{name}.exe', tool: 'cargo' },
    { target: 'x86_64-unknown-linux-musl', pkgSuffix: 'linux-x64', binName: '{name}', tool: 'zigbuild' },
    { target: 'x86_64-apple-darwin', pkgSuffix: 'darwin-x64', binName: '{name}', tool: 'zigbuild' },
    { target: 'aarch64-apple-darwin', pkgSuffix: 'darwin-arm64', binName: '{name}', tool: 'zigbuild' },
    { target: 'wasm32-wasip1', pkgSuffix: 'wasm32-wasi', binName: '{name}.wasm', tool: 'cargo', altBin: '{alt_name}.wasm' },
];

function build(projectName: string) {
    console.log(`\x1b[36m--- Building ${projectName} for all targets ---\x1b[0m`);

    for (const t of commonTargets) {
        const binName = t.binName.replace('{name}', projectName);
        const altBin = t.altBin?.replace('{alt_name}', projectName.replace(/-/g, '_'));
        const pkgName = `${projectName}-${t.pkgSuffix}`;
        const pkgPath = join(root, 'packages', pkgName);

        console.log(`\x1b[33mBuilding for ${t.target}...\x1b[0m`);

        const args = ['--release', '--target', t.target, '-p', projectName];
        
        if (t.tool === 'zigbuild') {
            args.unshift('zigbuild');
        } else {
            args.unshift('build');
        }

        const result = spawnSync('cargo', args, { stdio: 'inherit', cwd: root, shell: true });
        if (result.status !== 0) process.exit(result.status ?? 1);

        let srcBin = join(root, 'target', t.target, 'release', binName);
        if (altBin && !existsSync(srcBin)) {
            srcBin = join(root, 'target', t.target, 'release', altBin);
        }

        if (!existsSync(srcBin)) {
            console.error(`\x1b[31mCould not find built binary at ${srcBin}\x1b[0m`);
            continue;
        }

        if (!existsSync(pkgPath)) {
            mkdirSync(pkgPath, { recursive: true });
        }

        const destBin = join(pkgPath, binName);
        console.log(`\x1b[32mCopying ${binName} to ${pkgName}...\x1b[0m`);
        copyFileSync(srcBin, destBin);
    }
}

const project = process.argv[2];

if (project === 'lsp' || project === 'valkyrie-lsp') {
    build('valkyrie-lsp');
} else if (project === 'legion') {
    build('legion');
} else if (!project) {
    build('valkyrie-lsp');
    build('legion');
} else {
    console.error(`Unknown project: ${project}. Use 'lsp' or 'legion'.`);
    process.exit(1);
}

console.log(`\x1b[36m--- Build Complete ---\x1b[0m`);
