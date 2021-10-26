#!/usr/bin/env node
/**
 * Batch reword commit messages (UTF-8 safe, Windows-friendly).
 *
 * Preferred workflow uses a hash-keyed map: only commits listed in the file are
 * rewritten. See AGENTS.md「Git 提交（gitmoji）」.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync, unlinkSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const statePath = join(root, '.git', 'reword-state.json');

const HELP = `reword.mjs — batch reword commit messages (UTF-8 safe)

Preferred map file (only listed commits are rewritten):

  <full-or-prefix-hash>
  ✨ Subject with \`identifiers\` in backticks

  Optional body paragraph.

  ---

Lines starting with # are comments. Blocks are separated by a line containing only ---.

Options:
  --file <path>       Hash map or legacy sequential message file
  --base <ref>        Commit range base (default: origin/dev)
  --export [path]     Write hash-keyed template for <base>..HEAD (default: reword.pending.txt)
  --dry-run           Print planned rewords without running git rebase
  --lint              Lint messages in --file
  --lint-log          Lint commits in <base>..HEAD (style + duplicate subjects)
  --help              Show this help

Examples:
  node scripts/reword.mjs --lint-log --base origin/dev
  node scripts/reword.mjs --export --base 0210bb77
  node scripts/reword.mjs --lint --file reword.pending.txt --base origin/dev
  node scripts/reword.mjs --dry-run --file reword.pending.txt --base origin/dev
  node scripts/reword.mjs --file reword.pending.txt --base origin/dev
`;

const HASH_LINE = /^[0-9a-f]{8,40}$/i;
const EMOJI_START = /^\p{Extended_Pictographic}/u;
const MILESTONE = /\b(phase\s*[-_]?\s*\d|m\d|s\d|a\d|f\d|gate[- ]?\d)\b/i;
const BARE_TS = /\bTS\b(?![a-z])/;

/** @param {string[]} argv */
function parseArgs(argv) {
    const opts = {
        file: '',
        base: 'origin/dev',
        dryRun: false,
        lint: false,
        lintLog: false,
        exportPath: '',
        help: false,
    };
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (arg === '--help' || arg === '-h') {
            opts.help = true;
        } else if (arg === '--file') {
            opts.file = argv[++i] ?? '';
        } else if (arg === '--base') {
            opts.base = argv[++i] ?? '';
        } else if (arg === '--dry-run') {
            opts.dryRun = true;
        } else if (arg === '--lint') {
            opts.lint = true;
        } else if (arg === '--lint-log') {
            opts.lintLog = true;
        } else if (arg === '--export') {
            const next = argv[i + 1];
            if (next && !next.startsWith('-')) {
                opts.exportPath = next;
                i++;
            } else {
                opts.exportPath = 'reword.pending.txt';
            }
        } else {
            console.error(`reword: unknown argument: ${arg}`);
            process.exit(1);
        }
    }
    return opts;
}

/** @param {string[]} args */
function git(...args) {
    const result = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
    if (result.status !== 0) {
        const detail = (result.stderr || result.stdout || '').trim();
        throw new Error(detail || `git ${args.join(' ')} failed`);
    }
    return (result.stdout ?? '').trimEnd();
}

/** @param {string} text */
function splitBlocks(text) {
    return text
        .replace(/\r\n/g, '\n')
        .split(/\n---\n/)
        .map((block) => block.trim())
        .filter(Boolean);
}

/**
 * @param {string} raw
 * @returns {{ subject: string, body: string }}
 */
function splitMessage(raw) {
    const lines = raw.split('\n');
    const subject = (lines[0] ?? '').trim();
    const body = lines.slice(1).join('\n').trim();
    return { subject, body };
}

/** @param {string} raw */
function formatMessage(raw) {
    const { subject, body } = splitMessage(raw);
    return body ? `${subject}\n\n${body}\n` : `${subject}\n`;
}

/** @param {string} raw @param {string} [label] */
function lintMessage(raw, label = 'message') {
    const errors = [];
    const { subject, body } = splitMessage(raw);
    const full = body ? `${subject}\n${body}` : subject;

    if (!subject) {
        errors.push(`${label}: subject is empty`);
        return errors;
    }
    if (!EMOJI_START.test(subject)) {
        errors.push(`${label}: subject must start with a real gitmoji character`);
    }
    if (subject.endsWith('.')) {
        errors.push(`${label}: subject must not end with a period`);
    }
    if (/[;；]/.test(full)) {
        errors.push(`${label}: must not contain semicolons (use periods or new lines)`);
    }
    if (MILESTONE.test(full)) {
        errors.push(`${label}: must not contain internal milestone codes (Phase, Gate-N, etc.)`);
    }
    if (BARE_TS.test(full)) {
        errors.push(`${label}: write TypeScript in full instead of bare TS`);
    }
    if (/\b0\.\d+\.\d+\b/.test(full)) {
        errors.push(`${label}: avoid version numbers in commit messages`);
    }
    return errors;
}

/** @param {string} base */
function listCommits(base) {
    const out = git('log', '--reverse', '--format=%H %s', `${base}..HEAD`);
    if (!out) {
        return [];
    }
    return out.split('\n').map((line) => {
        const space = line.indexOf(' ');
        return { hash: line.slice(0, space), subject: line.slice(space + 1) };
    });
}

/**
 * @param {string} prefix
 * @param {{ hash: string, subject: string }[]} commits
 */
function resolveHashPrefix(prefix, commits) {
    const needle = prefix.toLowerCase();
    const matches = commits.filter(({ hash }) => hash.toLowerCase().startsWith(needle));
    if (matches.length === 0) {
        throw new Error(`hash prefix not found in range: ${prefix}`);
    }
    if (matches.length > 1) {
        throw new Error(`ambiguous hash prefix in range: ${prefix}`);
    }
    return matches[0].hash;
}

/**
 * @param {string} text
 * @returns {'hash' | 'sequential'}
 */
function detectFileMode(text) {
    for (const block of splitBlocks(text)) {
        for (const line of block.split('\n')) {
            const trimmed = line.trim();
            if (!trimmed || trimmed.startsWith('#')) {
                continue;
            }
            if (HASH_LINE.test(trimmed)) {
                return 'hash';
            }
            if (EMOJI_START.test(trimmed)) {
                return 'sequential';
            }
            throw new Error(`reword: cannot detect file mode from line: ${trimmed}`);
        }
    }
    throw new Error('reword: message file is empty');
}

/**
 * @param {string} text
 * @param {{ hash: string, subject: string }[]} commits
 */
function parseHashMapFile(text, commits) {
    /** @type {Map<string, string>} */
    const byHash = new Map();
    const blocks = splitBlocks(text);
    if (blocks.length === 0) {
        throw new Error('reword: no commit blocks in file');
    }

    for (const [index, block] of blocks.entries()) {
        const lines = block.split('\n');
        let hashLine = '';
        let start = 0;
        for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed || trimmed.startsWith('#')) {
                start += line.length + 1;
                continue;
            }
            hashLine = trimmed;
            start += line.length + 1;
            break;
        }
        if (!HASH_LINE.test(hashLine)) {
            throw new Error(`block ${index + 1}: expected commit hash, got ${hashLine || '(empty)'}`);
        }
        const message = block.slice(start).trim();
        if (!message) {
            throw new Error(`block ${index + 1}: missing message for ${hashLine}`);
        }
        const fullHash = resolveHashPrefix(hashLine, commits);
        if (byHash.has(fullHash)) {
            throw new Error(`duplicate hash entry in file: ${hashLine}`);
        }
        byHash.set(fullHash, message);
    }
    return byHash;
}

/** @param {string} text */
function parseSequentialFile(text) {
    const messages = splitBlocks(text);
    if (messages.length === 0) {
        throw new Error('reword: no commit blocks in file');
    }
    return messages;
}

/**
 * @param {string} path
 * @param {{ hash: string, subject: string }[]} commits
 */
function loadRewordPlan(path, commits) {
    const abs = resolve(root, path);
    if (!existsSync(abs)) {
        throw new Error(`message file not found: ${path}`);
    }
    const text = readFileSync(abs, 'utf8');
    const mode = detectFileMode(text);

    if (mode === 'hash') {
        const byHash = parseHashMapFile(text, commits);
        return { mode, byHash, messages: [] };
    }

    console.warn('reword: sequential message files are deprecated. Prefer `node scripts/reword.mjs --export`.');
    const messages = parseSequentialFile(text);
    if (messages.length !== commits.length) {
        throw new Error(
            `sequential message count (${messages.length}) does not match commit count (${commits.length}) in range`,
        );
    }
    return { mode, byHash: new Map(), messages };
}

/** @param {string} base @param {string} [path] */
function exportTemplate(base, path = 'reword.pending.txt') {
    const commits = listCommits(base);
    if (commits.length === 0) {
        throw new Error(`no commits in range ${base}..HEAD`);
    }

    const chunks = [
        `# Hash-keyed reword map for ${base}..HEAD (${commits.length} commit(s)).`,
        '# Delete blocks you do not want to change before running reword.',
        `# Generated: node scripts/reword.mjs --export --base ${base}`,
        '',
    ];

    for (const { hash } of commits) {
        const body = git('log', '-1', '--format=%B', hash).trimEnd();
        chunks.push(hash, body, '', '---', '');
    }

    const abs = resolve(root, path);
    writeFileSync(abs, `${chunks.join('\n').replace(/\n+$/, '')}\n`, 'utf8');
    console.log(`export: wrote ${commits.length} block(s) to ${path}`);
}

/**
 * @param {string} base
 * @param {{ mode: 'hash' | 'sequential', byHash: Map<string, string>, messages: string[] }} plan
 * @param {boolean} dryRun
 */
function planReword(base, plan, dryRun) {
    const commits = listCommits(base);
    if (commits.length === 0) {
        throw new Error(`no commits in range ${base}..HEAD`);
    }

    /** @type {{ hash: string, from: string, to: string, message: string }[]} */
    const changes = [];

    if (plan.mode === 'hash') {
        for (const { hash, subject } of commits) {
            const message = plan.byHash.get(hash);
            if (!message) {
                continue;
            }
            const current = git('log', '-1', '--format=%B', hash).trimEnd();
            const nextSubject = splitMessage(message).subject;
            if (nextSubject === subject && current === message.trim()) {
                if (dryRun) {
                    console.log(`${hash.slice(0, 8)}  ${subject}`);
                    console.log('       ->  (unchanged)\n');
                }
                continue;
            }
            changes.push({ hash, from: subject, to: nextSubject, message });
        }
        if (changes.length === 0) {
            if (dryRun) {
                console.log('dry-run: hash map matched, no message changes needed');
                return { commits, changes };
            }
            throw new Error('reword: no commits matched the hash map or all messages are unchanged');
        }
    } else {
        for (let i = 0; i < commits.length; i++) {
            const { hash, subject } = commits[i];
            const message = plan.messages[i];
            const nextSubject = splitMessage(message).subject;
            if (nextSubject !== subject || git('log', '-1', '--format=%B', hash).trimEnd() !== message.trim()) {
                changes.push({ hash, from: subject, to: nextSubject, message });
            }
        }
    }

    console.log(`reword plan (${plan.mode}): ${changes.length} commit(s) to rewrite after ${base}\n`);
    for (const { hash, from, to } of changes) {
        console.log(`${hash.slice(0, 8)}  ${from}`);
        console.log(`       ->  ${to}\n`);
    }

    for (const { hash, message } of changes) {
        const errors = lintMessage(message, `${hash.slice(0, 8)}`);
        for (const err of errors) {
            console.error(`lint: ${err}`);
        }
        if (errors.length > 0) {
            process.exit(1);
        }
    }

    if (dryRun) {
        console.log('dry-run: no rebase performed');
        return { commits, changes };
    }

    return { commits, changes };
}

function assertCleanWorktree() {
    const status = git('status', '--porcelain');
    if (status) {
        throw new Error('working tree is not clean. Commit or stash changes before reword.');
    }
}

/**
 * @param {string} base
 * @param {{ mode: 'hash' | 'sequential', byHash: Map<string, string>, messages: string[] }} plan
 * @param {{ commits: { hash: string, subject: string }[], changes: { hash: string, message: string }[] }} planned
 */
function runReword(base, plan, planned) {
    assertCleanWorktree();

    const state = {
        mode: plan.mode,
        byHash: Object.fromEntries(
            plan.mode === 'hash'
                ? planned.changes.map(({ hash, message }) => [hash, message])
                : plan.byHash.entries(),
        ),
        orderedHashes: planned.commits.map(({ hash }) => hash),
        messages: plan.mode === 'sequential' ? plan.messages : [],
    };
    writeFileSync(statePath, JSON.stringify(state, null, 2), 'utf8');

    const editorCopy = join(root, '.git', 'reword-editor.mjs');
    writeFileSync(editorCopy, readFileSync(join(root, 'scripts', 'reword.mjs'), 'utf8'), 'utf8');

    const node = process.execPath;
    const env = {
        ...process.env,
        GIT_SEQUENCE_EDITOR: `"${node}" "${editorCopy}" --sequence-editor`,
        GIT_EDITOR: `"${node}" "${editorCopy}" --commit-editor`,
    };

    const result = spawnSync('git', ['rebase', '-i', base], {
        cwd: root,
        env,
        stdio: 'inherit',
        shell: true,
    });

    if (existsSync(statePath)) {
        unlinkSync(statePath);
    }
    if (existsSync(editorCopy)) {
        unlinkSync(editorCopy);
    }

    if (result.status !== 0) {
        process.exit(result.status === null ? 1 : result.status);
    }
}

/** @param {string} todoPath */
function sequenceEditor(todoPath) {
    if (!existsSync(statePath)) {
        throw new Error('reword state file missing (.git/reword-state.json)');
    }
    const state = JSON.parse(readFileSync(statePath, 'utf8'));
    const targets =
        state.mode === 'hash'
            ? new Set(Object.keys(state.byHash))
            : new Set(
                  state.orderedHashes.filter((_, index) => {
                      const current = git('log', '-1', '--format=%B', state.orderedHashes[index]).trimEnd();
                      return current !== state.messages[index].trim();
                  }),
              );

    const lines = readFileSync(todoPath, 'utf8').split(/\r?\n/);
    const next = lines
        .map((line) => {
            if (!line.startsWith('pick ')) {
                return line;
            }
            const hash = line.slice(5).trim().split(/\s+/)[0];
            const fullHash = state.orderedHashes.find((item) => item.startsWith(hash));
            if (fullHash && targets.has(fullHash)) {
                return `reword ${line.slice(5)}`;
            }
            return line;
        })
        .join('\n');
    writeFileSync(todoPath, `${next}\n`, 'utf8');
}

/** @param {string} editPath */
function commitEditor(editPath) {
    if (!existsSync(statePath)) {
        throw new Error('reword state file missing (.git/reword-state.json)');
    }
    const state = JSON.parse(readFileSync(statePath, 'utf8'));
    const head = git('rev-parse', 'HEAD');

    let message = '';
    if (state.mode === 'hash') {
        message = state.byHash[head] ?? '';
    } else {
        const index = state.orderedHashes.indexOf(head);
        if (index === -1) {
            throw new Error(`commit ${head.slice(0, 8)} not found in reword plan`);
        }
        message = state.messages[index] ?? '';
    }

    if (!message) {
        return;
    }

    writeFileSync(editPath, formatMessage(message), 'utf8');
}

/** @param {{ hash: string, subject: string }[]} commits */
function lintDuplicateSubjects(commits) {
    /** @type {Map<string, { hash: string, subject: string }[]>} */
    const bySubject = new Map();
    for (const commit of commits) {
        if (!bySubject.has(commit.subject)) {
            bySubject.set(commit.subject, []);
        }
        bySubject.get(commit.subject)?.push(commit);
    }

    let failed = 0;
    for (const [subject, group] of bySubject) {
        if (group.length <= 1) {
            continue;
        }
        failed++;
        console.error(`\nduplicate subject (${group.length}x): ${subject}`);
        for (const { hash } of group) {
            console.error(`  - ${hash.slice(0, 8)}`);
        }
    }

    for (let i = 1; i < commits.length; i++) {
        if (commits[i].subject !== commits[i - 1].subject) {
            continue;
        }
        failed++;
        console.error(
            `\nconsecutive duplicate subject:\n  ${commits[i - 1].hash.slice(0, 8)} + ${commits[i].hash.slice(0, 8)}  ${commits[i].subject}`,
        );
    }

    return failed;
}

function lintLog(base) {
    const commits = listCommits(base);
    if (commits.length === 0) {
        console.log(`no commits in ${base}..HEAD`);
        return;
    }

    let styleFailed = 0;
    for (const { hash, subject } of commits) {
        const raw = git('log', '-1', '--format=%B', hash);
        const errors = lintMessage(raw.trimEnd(), hash.slice(0, 8));
        if (errors.length === 0) {
            continue;
        }
        styleFailed++;
        console.error(`\n${hash.slice(0, 8)}  ${subject}`);
        for (const err of errors) {
            console.error(`  - ${err}`);
        }
    }

    const duplicateFailed = lintDuplicateSubjects(commits);
    const failed = styleFailed + duplicateFailed;
    if (failed > 0) {
        console.error(`\nlint-log: ${failed} issue(s) in ${commits.length} commit(s)`);
        process.exit(1);
    }
    console.log(`lint-log: ${commits.length} commit(s) OK`);
}

function lintFile(path, base) {
    const commits = listCommits(base);
    const plan = loadRewordPlan(path, commits);
    let failed = 0;

    const entries =
        plan.mode === 'hash'
            ? [...plan.byHash.entries()].map(([hash, message]) => ({ label: hash.slice(0, 8), message }))
            : plan.messages.map((message, index) => ({ label: `block ${index + 1}`, message }));

    for (const { label, message } of entries) {
        const errors = lintMessage(message, label);
        for (const err of errors) {
            console.error(`lint: ${err}`);
        }
        if (errors.length > 0) {
            failed++;
        }
    }

    if (failed > 0) {
        process.exit(1);
    }
    console.log(`lint: ${entries.length} block(s) OK (${plan.mode} mode)`);
}

function main() {
    const argv = process.argv.slice(2);
    if (argv[0] === '--sequence-editor' && argv[1]) {
        sequenceEditor(argv[1]);
        return;
    }
    if (argv[0] === '--commit-editor' && argv[1]) {
        commitEditor(argv[1]);
        return;
    }

    const opts = parseArgs(argv);
    if (opts.help) {
        console.log(HELP);
        return;
    }

    if (opts.lintLog) {
        lintLog(opts.base);
        return;
    }

    if (opts.exportPath) {
        exportTemplate(opts.base, opts.exportPath);
        return;
    }

    if (!opts.file) {
        console.error('reword: --file is required unless using --lint-log or --export');
        console.error(HELP);
        process.exit(1);
    }

    const commits = listCommits(opts.base);
    const plan = loadRewordPlan(opts.file, commits);

    if (opts.lint) {
        lintFile(opts.file, opts.base);
        return;
    }

    const planned = planReword(opts.base, plan, opts.dryRun);
    if (opts.dryRun || !planned) {
        return;
    }

    runReword(opts.base, plan, planned);
}

main();
