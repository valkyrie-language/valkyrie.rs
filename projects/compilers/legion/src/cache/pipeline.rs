//! Frontend cache waterfall: staging → tokens → semantics (IR/artifact stored by caller).

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::planner::PlannedSemanticSourceGroup;
use miette::{IntoDiagnostic, NamedSource, Result as MietteResult, miette};
use nyar_language::{CompilerSourceGroup, FrontendBuildOutput, ValkyrieCompiler};
use nyar_workspace::{combined_hash, file_hash};
use vcc_data::text::valkyrie::lexer::{Lexer, Token, decode_tokens, encode_tokens};

use super::{CompilationCache, StageCacheEntry, TokenCacheEntry};

/// Result of the frontend cache waterfall.
#[derive(Debug)]
pub struct CachedFrontendCompile {
    /// Combined preprocessed source (for diagnostics).
    pub combined_source: String,
    /// Frontend bundle ready for backend planning.
    pub build_output: FrontendBuildOutput,
    /// Whether every per-file staging entry was restored.
    pub staging_hit: bool,
    /// Whether tokens for the parse input were restored.
    pub tokens_hit: bool,
}

/// 将 Resolver 的源码组快照交给 Compiler；Legion 不拥有语义导出或 MIR 链接。
pub fn compile_source_snapshot(
    groups: &[PlannedSemanticSourceGroup],
    arch: &str,
    preprocess: impl Fn(&str, &str) -> String,
) -> MietteResult<FrontendBuildOutput> {
    let mut compiler_groups = Vec::with_capacity(groups.len());
    for group in groups {
        let mut source = String::new();
        for path in &group.source_files {
            let content =
                fs::read_to_string(path).into_diagnostic().map_err(|error| error.wrap_err(format!("读取源码失败 {}", path.display())))?;
            let staged = preprocess(content.strip_prefix('\u{FEFF}').unwrap_or(&content), arch);
            source.push_str(&staged);
            source.push('\n');
        }
        compiler_groups.push(CompilerSourceGroup {
            dependency_key: group.dependency_key.clone(),
            name: group.name.clone(),
            source,
            direct_dependencies: group.direct_dependencies.clone(),
        });
    }
    ValkyrieCompiler::default()
        .compile_source_groups(&compiler_groups)
        .map_err(|error| miette!("Compiler semantic snapshot failed: {error}"))
}

/// Load sources, apply only source-derived staging/token caches, return frontend output.
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

    let compiler = ValkyrieCompiler::default();

    let (_tokens, tokens_hit) = load_or_tokenize_combined(cache, source_files, &combined_source)?;
    let build_output = compiler
        .compile_source_to_build_output(&combined_source)
        .map_err(|error| attach_combined_source(error, &combined_source, source_files, &staged_parts))?;

    Ok(CachedFrontendCompile { combined_source, build_output, staging_hit: all_staging_hit, tokens_hit })
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
    fn frontend_waterfall_recompiles_semantics_from_source() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let source_path = write_main(dir.path());
        let sources = vec![source_path];
        let triple = "clr-microsoft-unknown-managed";

        let first = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(!first.tokens_hit);

        let second = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(second.staging_hit);
        assert!(second.tokens_hit);
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

        let _first = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        let second = compile_frontend_with_cache(&cache, &sources, triple, "clr", |s, _| s.to_string()).unwrap();
        assert!(second.staging_hit);
        assert!(second.tokens_hit);
    }
}
