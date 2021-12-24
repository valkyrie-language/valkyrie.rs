import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

/** 与 Legion `EXECUTION_MANIFEST_SCHEMA_VERSION` 对齐。 */
export const MIN_EXECUTION_MANIFEST_SCHEMA_VERSION = 3;

export const EXECUTION_MANIFEST_FILE = 'run-contracts.txt';
export const COMPILE_EVIDENCE_FILE = 'compile-evidence.txt';

export type ExecutionInputDigest = {
    role: string;
    path: string;
    hash: string;
};

export type ExecutionArtifactDigest = {
    path: string;
    hash: string;
};

export type CompileEvidenceDigest = {
    semantic_closure_hash: string;
    canonical_surface_hash: string;
    host_provider_binding_hash: string;
};

export type RunContractDigest = {
    physical_entry: string;
    logical_entry: string;
    invocation: string;
};

export type ExecutionManifest = {
    schema_version: number;
    target: string;
    project_name: string;
    inputs: ExecutionInputDigest[];
    artifacts: ExecutionArtifactDigest[];
    compile_evidence: CompileEvidenceDigest | null;
    run_contracts: RunContractDigest[];
};

export type ArtifactCacheFreshness = {
    fresh: boolean;
    reason: string | null;
    manifest: ExecutionManifest | null;
};

function hashFileSha256(filePath: string): string {
    return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

function parseQuotedField(block: string, field: string): string | null {
    const match = block.match(new RegExp(`${field}\\s*:\\s*"((?:\\\\.|[^"\\\\])*)"`));
    if (!match?.[1]) {
        return null;
    }
    return match[1].replace(/\\\\/g, '\\');
}

function parseObjectBlocks(section: string): string[] {
    const blocks: string[] = [];
    let depth = 0;
    let start = -1;
    for (let index = 0; index < section.length; index += 1) {
        const char = section[index];
        if (char === '{') {
            if (depth === 0) {
                start = index;
            }
            depth += 1;
        } else if (char === '}') {
            depth -= 1;
            if (depth === 0 && start >= 0) {
                blocks.push(section.slice(start, index + 1));
                start = -1;
            }
        }
    }
    return blocks;
}

function extractArraySection(source: string, field: string): string | null {
    const marker = new RegExp(`${field}\\s*:\\s*\\[`);
    const match = marker.exec(source);
    if (!match || match.index === undefined) {
        return null;
    }
    const openIndex = source.indexOf('[', match.index);
    let depth = 0;
    for (let index = openIndex; index < source.length; index += 1) {
        const char = source[index];
        if (char === '[') {
            depth += 1;
        } else if (char === ']') {
            depth -= 1;
            if (depth === 0) {
                return source.slice(openIndex + 1, index);
            }
        }
    }
    return null;
}

function extractObjectSection(source: string, field: string): string | null {
    const marker = new RegExp(`${field}\\s*:\\s*\\{`);
    const match = marker.exec(source);
    if (!match || match.index === undefined) {
        return null;
    }
    const openIndex = source.indexOf('{', match.index);
    let depth = 0;
    for (let index = openIndex; index < source.length; index += 1) {
        const char = source[index];
        if (char === '{') {
            depth += 1;
        } else if (char === '}') {
            depth -= 1;
            if (depth === 0) {
                return source.slice(openIndex, index + 1);
            }
        }
    }
    return null;
}

function parseInputDigests(source: string): ExecutionInputDigest[] {
    const section = extractArraySection(source, 'inputs');
    if (!section) {
        return [];
    }
    return parseObjectBlocks(section)
        .map((block) => ({
            role: parseQuotedField(block, 'role') ?? '',
            path: parseQuotedField(block, 'path') ?? '',
            hash: parseQuotedField(block, 'hash') ?? '',
        }))
        .filter((entry) => entry.role.length > 0 && entry.path.length > 0 && entry.hash.length > 0);
}

function parseArtifactDigests(source: string): ExecutionArtifactDigest[] {
    const section = extractArraySection(source, 'artifacts');
    if (!section) {
        return [];
    }
    return parseObjectBlocks(section)
        .map((block) => ({
            path: parseQuotedField(block, 'path') ?? '',
            hash: parseQuotedField(block, 'hash') ?? '',
        }))
        .filter((entry) => entry.path.length > 0 && entry.hash.length > 0);
}

function parseRunContracts(source: string): RunContractDigest[] {
    const section = extractArraySection(source, 'run_contracts');
    if (!section) {
        return [];
    }
    return parseObjectBlocks(section)
        .map((block) => ({
            physical_entry: parseQuotedField(block, 'physical_entry') ?? '',
            logical_entry: parseQuotedField(block, 'logical_entry') ?? 'main',
            invocation: parseQuotedField(block, 'invocation') ?? '',
        }))
        .filter((entry) => entry.physical_entry.length > 0);
}

function parseCompileEvidence(source: string): CompileEvidenceDigest | null {
    const block = extractObjectSection(source, 'compile_evidence');
    if (!block) {
        return null;
    }
    return parseCompileEvidenceRecord(block);
}

/** 解析 `run-contracts.txt` / `run-contract.txt` 中的 execution manifest 子集。 */
export function parseExecutionManifest(source: string): ExecutionManifest | null {
    const schemaMatch = source.match(/schema_version\s*:\s*(\d+)/);
    if (!schemaMatch) {
        return null;
    }
    const target = source.match(/target\s*:\s*"([^"]+)"/)?.[1] ?? '';
    const project_name = source.match(/project_name\s*:\s*"([^"]+)"/)?.[1] ?? '';
    const inputs = parseInputDigests(source);
    const artifacts = parseArtifactDigests(source);
    const run_contracts = parseRunContracts(source);
    const compile_evidence = parseCompileEvidence(source);
    return {
        schema_version: Number.parseInt(schemaMatch[1], 10),
        target,
        project_name,
        inputs,
        artifacts,
        compile_evidence,
        run_contracts,
    };
}

function parseCompileEvidenceRecord(source: string): CompileEvidenceDigest | null {
    const semantic_closure_hash = parseQuotedField(source, 'semantic_closure_hash');
    const canonical_surface_hash = parseQuotedField(source, 'canonical_surface_hash');
    const host_provider_binding_hash = parseQuotedField(source, 'host_provider_binding_hash');
    if (!semantic_closure_hash || !canonical_surface_hash || !host_provider_binding_hash) {
        return null;
    }
    return {
        semantic_closure_hash,
        canonical_surface_hash,
        host_provider_binding_hash,
    };
}

function readCompileEvidenceSidecar(artifactDir: string): CompileEvidenceDigest | null {
    const sidecarPath = join(artifactDir, COMPILE_EVIDENCE_FILE);
    if (!existsSync(sidecarPath)) {
        return null;
    }
    const source = readFileSync(sidecarPath, 'utf8');
    return parseCompileEvidence(source) ?? parseCompileEvidenceRecord(source);
}

/** 校验 manifest 与产物目录仍匹配当前源码闭包与 artifact 摘要。 */
export function verifyExecutionManifestFresh(manifest: ExecutionManifest, artifactDir: string, expectedTarget?: string): string | null {
    if (manifest.schema_version < MIN_EXECUTION_MANIFEST_SCHEMA_VERSION) {
        return `execution manifest schema ${manifest.schema_version} < ${MIN_EXECUTION_MANIFEST_SCHEMA_VERSION}`;
    }
    if (!manifest.compile_evidence) {
        return 'execution manifest missing compile_evidence';
    }
    if (expectedTarget && manifest.target !== expectedTarget) {
        return `execution manifest target ${manifest.target} != ${expectedTarget}`;
    }
    if (manifest.inputs.length === 0) {
        return 'execution manifest has no inputs';
    }
    if (manifest.artifacts.length === 0) {
        return 'execution manifest has no artifacts';
    }
    if (manifest.run_contracts.length === 0) {
        return 'execution manifest has no run_contracts';
    }

    for (const input of manifest.inputs) {
        if (!existsSync(input.path)) {
            return `execution manifest input missing: ${input.path}`;
        }
        if (hashFileSha256(input.path) !== input.hash) {
            return `execution manifest input stale: ${input.path}`;
        }
    }

    for (const artifact of manifest.artifacts) {
        const artifactPath = join(artifactDir, artifact.path);
        if (!existsSync(artifactPath)) {
            return `execution manifest artifact missing: ${artifact.path}`;
        }
        if (hashFileSha256(artifactPath) !== artifact.hash) {
            return `execution manifest artifact stale: ${artifact.path}`;
        }
    }

    const sidecar = readCompileEvidenceSidecar(artifactDir);
    if (!sidecar) {
        return `missing ${COMPILE_EVIDENCE_FILE}`;
    }
    if (
        sidecar.semantic_closure_hash !== manifest.compile_evidence.semantic_closure_hash ||
        sidecar.canonical_surface_hash !== manifest.compile_evidence.canonical_surface_hash ||
        sidecar.host_provider_binding_hash !== manifest.compile_evidence.host_provider_binding_hash
    ) {
        return 'compile-evidence sidecar does not match execution manifest';
    }

    return null;
}

export function readExecutionManifest(artifactDir: string): ExecutionManifest | null {
    for (const fileName of [EXECUTION_MANIFEST_FILE, 'run-contract.txt']) {
        const manifestPath = join(artifactDir, fileName);
        if (!existsSync(manifestPath)) {
            continue;
        }
        const manifest = parseExecutionManifest(readFileSync(manifestPath, 'utf8'));
        if (manifest) {
            return manifest;
        }
    }
    return null;
}

/** 判断缓存产物目录是否仍可信。 */
export function isArtifactCacheFresh(artifactDir: string, expectedTarget?: string): ArtifactCacheFreshness {
    const manifest = readExecutionManifest(artifactDir);
    if (!manifest) {
        return { fresh: false, reason: 'missing execution manifest', manifest: null };
    }
    const reason = verifyExecutionManifestFresh(manifest, artifactDir, expectedTarget);
    return { fresh: reason === null, reason, manifest };
}
