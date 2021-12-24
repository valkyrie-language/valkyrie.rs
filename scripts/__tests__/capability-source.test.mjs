import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmdirSync, unlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { capabilitySourceProject } from '../lib/capability-source.mjs';

const buildScript = fileURLToPath(new URL('../build.mjs', import.meta.url));

function withSourceRoot(body) {
    const root = mkdtempSync(join(tmpdir(), 'capability-source-'));
    const directories = [root];
    const files = [];
    const manifest = (parts) => {
        let directory = root;
        for (const part of parts) {
            directory = join(directory, part);
            if (!directories.includes(directory)) {
                mkdirSync(directory);
                directories.push(directory);
            }
        }
        const path = join(directory, 'legion.von');
        writeFileSync(path, '{}');
        files.push(path);
        return directory;
    };
    try {
        body(root, manifest);
    } finally {
        for (const path of files) unlinkSync(path);
        for (const directory of directories.reverse()) rmdirSync(directory);
    }
}

test('capability requires explicit source input before starting compilation', () => {
    assert.throws(() => capabilitySourceProject(), /必须显式提供/);
    const result = spawnSync(process.execPath, [buildScript, 'capability'], { encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /必须显式提供/);
    assert.doesNotMatch(result.stdout, /cargo test|build capability/);
});

test('capability selects only the requested canonical project', () => {
    withSourceRoot((root, manifest) => {
        const project = manifest(['projects', 'legion._', 'projects', 'legion.tools']);
        manifest(['projects', 'legion.tools']);
        assert.equal(capabilitySourceProject(root), project);
    });
});

test('old project layout cannot replace missing canonical source', () => {
    withSourceRoot((root, manifest) => {
        manifest(['projects', 'legion.tools']);
        assert.throws(() => capabilitySourceProject(root), /拒绝旧布局/);
        const result = spawnSync(process.execPath, [buildScript, 'capability', '--valkyrie-v', root], {
            cwd: dirname(buildScript),
            encoding: 'utf8',
        });
        assert.equal(result.status, 1);
        assert.match(result.stderr, /拒绝旧布局/);
        assert.doesNotMatch(result.stdout, /cargo test|build capability/);
    });
});

test('a directory named legion.von is not a source manifest', () => {
    withSourceRoot((root, manifest) => {
        const project = manifest(['projects', 'legion._', 'projects', 'legion.tools']);
        unlinkSync(join(project, 'legion.von'));
        mkdirSync(join(project, 'legion.von'));
        try {
            assert.throws(() => capabilitySourceProject(root), /缺少 legion.von/);
        } finally {
            rmdirSync(join(project, 'legion.von'));
            writeFileSync(join(project, 'legion.von'), '{}');
        }
    });
});
