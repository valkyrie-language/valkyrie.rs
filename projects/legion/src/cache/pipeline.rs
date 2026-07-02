//! Frontend cache waterfall: staging → tokens → semantics (IR/artifact stored by caller).

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::planner::PlannedSemanticSourceGroup;
use miette::{IntoDiagnostic, NamedSource, Result as MietteResult, miette};
use nyar_language::{
    FrontendBuildOutput, Identifier, NamePath, ValkyrieCompiler,
    types::hir::{HirDependencySemanticExport, HirModule},
};
use nyar_workspace::{combined_hash, file_hash};
use std_data::text::valkyrie::lexer::{Lexer, Token, decode_tokens, encode_tokens};

use super::{CompilationCache, SemanticCacheEntry, StageCacheEntry, TokenCacheEntry};

// Bump whenever HIR semantic contracts or their validation ordering changes.
// Source hashes alone cannot make an old serialized HIR safe to consume.
const SEMANTIC_CONTRACT_FINGERPRINT: &str = "semantic-contract-2026-08-08-utf8-scalar-and-content-opcodes";

/// Result of the frontend cache waterfall.
#[derive(Debug)]
pub struct CachedFrontendCompile {
    /// Combined preprocessed source (for diagnostics).
    pub combined_source: String,
    /// Frontend bundle ready for backend planning.
    pub build_output: FrontendBuildOutput,
    /// Whether semantics were restored from cache.
    pub semantics_hit: bool,
    /// Whether every per-file staging entry was restored.
    pub staging_hit: bool,
    /// Whether tokens for the parse input were restored.
    pub tokens_hit: bool,
}

/// Compiles workspace source groups in dependency order. This is intentionally
/// separate from the legacy text-concatenating cache path: a consumer receives
/// only direct dependency exports, never their source text or transitive HIR.
///
/// After the final consumer group compiles, reachable Valkyrie dependency MIR
/// bodies are linked into the consumer MIR so Stage1 emit SMIR003 can see them
/// in the executable registry (not as host stubs).
pub fn compile_semantic_source_groups(
    groups: &[PlannedSemanticSourceGroup],
    arch: &str,
    preprocess: impl Fn(&str, &str) -> String,
) -> MietteResult<FrontendBuildOutput> {
    let compiler = ValkyrieCompiler::default();
    let mut exports = std::collections::BTreeMap::<String, HirDependencySemanticExport>::new();
    let mut dependency_mirs = Vec::new();
    let mut final_output = None;
    eprintln!(
        "[seed-debug] semantic-groups count={} names={}",
        groups.len(),
        groups.iter().map(|group| group.name.as_str()).collect::<Vec<_>>().join(",")
    );
    for group in groups {
        eprintln!(
            "[seed-debug] semantic-group-begin name={} deps={} files={}",
            group.name,
            group.direct_dependencies.join(","),
            group.source_files.len()
        );
        let mut source = String::new();
        let mut source_files = Vec::new();
        let mut staged_parts = Vec::new();
        for path in &group.source_files {
            let content =
                fs::read_to_string(path).into_diagnostic().map_err(|error| error.wrap_err(format!("读取源码失败 {}", path.display())))?;
            let staged = preprocess(content.strip_prefix('\u{FEFF}').unwrap_or(&content), arch);
            source_files.push(path.clone());
            staged_parts.push(staged.clone());
            source.push_str(&staged);
            source.push('\n');
        }
        let dependency_exports = group
            .direct_dependencies
            .iter()
            .map(|name| {
                exports.get(name).cloned().ok_or_else(|| {
                    miette!(
                        "semantic dependency export `{name}` is unavailable for `{}` (have: {})",
                        group.name,
                        exports.keys().cloned().collect::<Vec<_>>().join(",")
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let output = compiler
            .compile_source_to_build_output_with_semantic_exports(&source, &dependency_exports)
            .map_err(|error| attach_combined_source(error, &source, &source_files, &staged_parts))?;
        let hir = output.hir_module();
        let export = HirDependencySemanticExport {
            module: NamePath::new(vec![Identifier::new(&group.name)]),
            functions: hir.functions.clone(),
            structs: hir.structs.clone(),
            enums: hir.enums.clone(),
            traits: hir.traits.clone(),
            type_aliases: hir.type_aliases.clone(),
            impls: hir.impls.clone(),
        };
        exports.insert(group.name.clone(), export.clone());
        // auto_link / dependencies use directory basename (`core`) while
        // manifest.name may be `valkyrie-core`. Mirror planner's projects_by_name alias.
        if let Some(basename) = group.manifest_dir.file_name().and_then(|name| name.to_str()) {
            if basename != group.name {
                exports.entry(basename.to_string()).or_insert(export);
            }
        }
        // Keep prior groups' MIR for Stage1 link; the final consumer replaces
        // `final_output` and receives linked bodies below.
        if let Some(previous) = final_output.replace(output) {
            dependency_mirs.push(previous.semantic_mir().clone());
        }
    }
    let mut final_output = final_output.ok_or_else(|| miette!("semantic source group plan is empty"))?;
    if !dependency_mirs.is_empty() {
        eprintln!(
            "[seed-debug] dependency-mir-link-start deps={} consumer_functions={}",
            dependency_mirs.len(),
            final_output.semantic_mir().functions.len()
        );
        final_output.link_dependency_mir_modules(&dependency_mirs);
    }
    Ok(final_output)
}

/// Load sources, apply staging/token/semantics caches, return frontend output.
pub fn compile_frontend_with_cache(
    cache: &CompilationCache,
    source_files: &[PathBuf],
    canonical_triple: &str,
    arch: &str,
    preprocess: impl Fn(&str, &str) -> String,
) -> MietteResult<CachedFrontendCompile> {
    if source_files.is_empty() {
        return Err(miette!("没有找到任何源码文件"));
    }

    let mut all_staging_hit = true;
    let mut staged_parts = Vec::with_capacity(source_files.len());
    for (index, source_path) in source_files.iter().enumerate() {
        let (staged, hit) = stage_source_file(cache, source_path, canonical_triple, arch, &preprocess)?;
        all_staging_hit &= hit;
        staged_parts.push(staged);
        if index == 0 || (index + 1) % 25 == 0 || index + 1 == source_files.len() {
            eprintln!("[seed-debug] frontend-stage-progress {}/{} hit={}", index + 1, source_files.len(), hit);
        }
    }

    let mut combined_source = String::new();
    let mut debug_map = String::new();
    let mut offset = 0usize;
    for (source_path, staged) in source_files.iter().zip(staged_parts.iter()) {
        let end = offset + staged.len();
        debug_map.push_str(&format!("{offset}-{end}: {}\n", source_path.display()));
        combined_source.push_str(staged);
        combined_source.push('\n');
        offset = combined_source.len();
    }
    let _ = fs::write("target/source-offsets.txt", &debug_map);
    let _ = fs::write("target/preprocessed-source.v", &combined_source);

    let primary_path = path_key(&source_files[0]);
    let ast_hash = combined_hash(&[&combined_source, SEMANTIC_CONTRACT_FINGERPRINT]);
    eprintln!("[seed-debug] frontend-staging-done files={} bytes={} all_hit={}", source_files.len(), combined_source.len(), all_staging_hit);
    let compiler = ValkyrieCompiler::default();

    if let Some(entry) = cache.try_get_semantics(&primary_path, canonical_triple, &ast_hash) {
        if let Ok(hir) = serde_json::from_slice::<HirModule>(&entry.semantic_data) {
            compiler
                .validate_hir_semantic_contract(&hir)
                .map_err(|error| attach_combined_source(error, &combined_source, source_files, &staged_parts))?;
            let build_output = FrontendBuildOutput::from_hir_module(hir);
            eprintln!("[seed-debug] frontend-semantics-cache-hit");
            return Ok(CachedFrontendCompile {
                combined_source,
                build_output,
                semantics_hit: true,
                staging_hit: all_staging_hit,
                tokens_hit: false,
            });
        }
    }

    let (_tokens, tokens_hit) = load_or_tokenize_combined(cache, source_files, &combined_source)?;
    eprintln!("[seed-debug] frontend-tokenize-done hit={tokens_hit}");
    let build_output = compiler
        .compile_source_to_build_output(&combined_source)
        .map_err(|error| attach_combined_source(error, &combined_source, source_files, &staged_parts))?;
    eprintln!("[seed-debug] frontend-semantic-compile-done");

    if let Ok(semantic_data) = serde_json::to_vec(build_output.hir_module()) {
        let _ = cache.put_semantics(
            &primary_path,
            canonical_triple,
            &ast_hash,
            &SemanticCacheEntry { semantic_data, ast_hash: ast_hash.clone(), canonical_triple: canonical_triple.to_string() },
        );
    }
    Ok(CachedFrontendCompile { combined_source, build_output, semantics_hit: false, staging_hit: all_staging_hit, tokens_hit })
}

fn stage_source_file(
    cache: &CompilationCache,
    source_path: &Path,
    canonical_triple: &str,
    arch: &str,
    preprocess: &impl Fn(&str, &str) -> String,
) -> MietteResult<(String, bool)> {
    let path = path_key(source_path);
    let content_hash = file_hash(source_path).map_err(|e| miette!("{e}"))?;
    if let Some(entry) = cache.try_get_staging(&path, canonical_triple, &content_hash) {
        if let Ok(text) = String::from_utf8(entry.staged_token_data) {
            return Ok((text, true));
        }
    }

    let content =
        fs::read_to_string(source_path).into_diagnostic().map_err(|error| error.wrap_err(format!("读取源码失败 {}", source_path.display())))?;
    let trimmed = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    let staged = preprocess(trimmed, arch);
    let _ = cache.put_staging(
        &path,
        canonical_triple,
        &content_hash,
        &StageCacheEntry {
            staged_token_data: staged.as_bytes().to_vec(),
            content_hash: content_hash.clone(),
            canonical_triple: canonical_triple.to_string(),
        },
    );
    Ok((staged, false))
}

/// Token cache for the combined staged text used by the parser.
fn load_or_tokenize_combined(cache: &CompilationCache, source_files: &[PathBuf], combined_source: &str) -> MietteResult<(Vec<Token>, bool)> {
    let token_path = if source_files.len() == 1 { path_key(&source_files[0]) } else { format!("__combined__/{}", path_key(&source_files[0])) };
    // Hash the staged combined text so arch/preprocess changes invalidate tokens.
    let content_hash = combined_hash(&[combined_source]);
    if let Some(entry) = cache.try_get_tokens(&token_path, &content_hash) {
        if let Some(tokens) = decode_tokens(&entry.token_data) {
            return Ok((tokens, true));
        }
    }

    // Populate per-file token entries from each original source (plan: per source_file).
    for source_path in source_files {
        let path = path_key(source_path);
        let file_content_hash = file_hash(source_path).map_err(|e| miette!("{e}"))?;
        if cache.try_get_tokens(&path, &file_content_hash).is_none() {
            let content = fs::read_to_string(source_path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("读取源码失败 {}", source_path.display())))?;
            let trimmed = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
            if let Ok(tokens) = Lexer::tokenize(trimmed) {
                let _ = cache.put_tokens(
                    &path,
                    &file_content_hash,
                    &TokenCacheEntry { token_data: encode_tokens(&tokens), content_hash: file_content_hash.clone() },
                );
            }
        }
    }

    let tokens = Lexer::tokenize(combined_source).map_err(|error| attach_source(error, combined_source))?;
    let _ = cache.put_tokens(
        &token_path,
        &content_hash,
        &TokenCacheEntry { token_data: encode_tokens(&tokens), content_hash: content_hash.clone() },
    );
    Ok((tokens, false))
}

fn path_key(path: &Path) -> String {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().replace('\\', "/")
}

fn attach_source(error: impl Into<miette::Report>, source: &str) -> miette::Report {
    error.into().with_source_code(NamedSource::new("combined-source.v", source.to_string()))
}

/// Preserve the original source path for diagnostics produced after the seed
/// concatenates a package's staged source files into one compilation unit.
fn attach_combined_source(error: impl Into<miette::Report>, source: &str, source_files: &[PathBuf], staged_parts: &[String]) -> miette::Report {
    let report = error.into();
    let message = report.to_string();
    let diagnostic = format!("{report:?}");
    eprintln!("[seed-debug] frontend-compile-error-debug {diagnostic}");
    let location = combined_source_location(&diagnostic, source_files, staged_parts)
        .map(|location| format!("\n\n[combined source location] {location}"))
        .unwrap_or_default();
    miette!("{message}{location}").with_source_code(NamedSource::new("combined-source.v", source.to_string()))
}

fn combined_source_location(message: &str, source_files: &[PathBuf], staged_parts: &[String]) -> Option<String> {
    let span_marker = "span: ";
    let after_marker = message.split_once(span_marker)?.1;
    let offset_text: String = after_marker.chars().take_while(char::is_ascii_digit).collect();
    let offset = offset_text.parse::<usize>().ok()?;
    let mut start = 0usize;
    for (path, staged) in source_files.iter().zip(staged_parts) {
        let end = start + staged.len();
        if offset <= end {
            return Some(format!("{} byte {}", path.display(), offset.saturating_sub(start)));
        }
        start = end + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std_data::text::valkyrie::AstParser;
    use tempfile::tempdir;

    fn write_main(dir: &Path) -> PathBuf {
        let path = dir.join("main.v");
        fs::write(
            &path,
            r#"[main]
micro main(): i64 {
    return 0;
}
"#,
        )
        .unwrap();
        path
    }

    #[test]
    fn frontend_waterfall_hits_semantics_on_second_compile() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let source_path = write_main(dir.path());
        let sources = vec![source_path];
        let triple = "clr-microsoft-unknown-managed";

        let first = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(!first.semantics_hit);
        assert!(!first.tokens_hit);

        let second = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(second.semantics_hit);
        assert!(second.staging_hit);
        assert_eq!(second.build_output.hir_function_count(), first.build_output.hir_function_count());
    }

    #[test]
    fn staging_cache_is_triple_specific() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let path = write_main(dir.path());
        let key = path_key(&path);
        let content_hash = file_hash(&path).unwrap();
        let clr = "clr-microsoft-unknown-managed";
        let jvm = "jvm-oracle-unknown-managed";
        cache
            .put_staging(
                &key,
                clr,
                &content_hash,
                &StageCacheEntry { staged_token_data: b"clr_body".to_vec(), content_hash: content_hash.clone(), canonical_triple: clr.into() },
            )
            .unwrap();
        assert!(cache.try_get_staging(&key, jvm, &content_hash).is_none());
        assert_eq!(String::from_utf8(cache.try_get_staging(&key, clr, &content_hash).unwrap().staged_token_data).unwrap(), "clr_body");
    }

    #[test]
    fn tokens_hit_when_semantics_missing() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let source_path = write_main(dir.path());
        let sources = vec![source_path];
        let triple = "clr-microsoft-unknown-managed";

        let first = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(!first.semantics_hit);

        // Drop only semantics by overwriting with garbage that fails to deserialize.
        let primary = path_key(&sources[0]);
        let ast_hash = combined_hash(&[&first.combined_source, SEMANTIC_CONTRACT_FINGERPRINT]);
        cache
            .put_semantics(
                &primary,
                triple,
                &ast_hash,
                &SemanticCacheEntry { semantic_data: b"not-json".to_vec(), ast_hash: ast_hash.clone(), canonical_triple: triple.into() },
            )
            .unwrap();

        let second = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(!second.semantics_hit);
        assert!(second.staging_hit);
        assert!(second.tokens_hit);
    }

    #[test]
    fn cached_hir_must_pass_the_current_semantic_contract() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let source_path = write_main(dir.path());
        let sources = vec![source_path];
        let triple = "clr-microsoft-unknown-managed";
        let source = fs::read_to_string(&sources[0]).unwrap();
        let ast_hash = combined_hash(&[&format!("{source}\n"), SEMANTIC_CONTRACT_FINGERPRINT]);
        let stale_hir = ValkyrieCompiler::default()
            .lower_root(&AstParser::parse_root("micro main() -> i64 { return absent_call() }").unwrap())
            .expect("raw HIR construction for stale-cache regression");
        let primary = path_key(&sources[0]);
        cache
            .put_semantics(
                &primary,
                triple,
                &ast_hash,
                &SemanticCacheEntry { semantic_data: serde_json::to_vec(&stale_hir).unwrap(), ast_hash, canonical_triple: triple.into() },
            )
            .unwrap();

        let error = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string())
            .expect_err("a stale HIR with an unresolved call must not cross the cache boundary");
        assert!(error.to_string().contains("SMIR003"), "unexpected cache-contract error: {error}");
    }
}
