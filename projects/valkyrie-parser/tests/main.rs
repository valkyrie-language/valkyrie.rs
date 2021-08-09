#![allow(unused, dead_code)]

use nyar_error::{third_party::Url, Failure, SourceCache, Success, Validation};
use std::{
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use valkyrie_parser::{
    ClassBlockNode, ClassTermNode, DefineFieldNode, DefineImportNode, DefineMethodNode, DefineNamespaceNode,
    MainExpressionNode, ProgramContext, ProgramNode, StatementNode, ValkyrieParser, ValkyrieRule,
};
use yggdrasil_rt::{OutputResult, YggdrasilError, YggdrasilParser};

mod expression;

mod literal;
// mod statement;

#[test]
fn ready() {
    println!("it works!")
}
//
fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").canonicalize().expect("failed to get manifest dir")
}

// pub fn pretty_print<T: PrettyPrint>(value: &T) {
//     let arena = PrettyProvider::new(80);
//     println!("{}", value.pretty_colorful(&arena));
// }

fn parse_program(input: &str, output: &str) -> std::io::Result<()> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").canonicalize()?;
    let cst = ValkyrieParser::parse_cst(input, ValkyrieRule::Program).unwrap();
    println!("Short Form:\n{}", cst);
    let ast = ProgramNode::from_str(input).unwrap();
    let mut file = File::create(here.join(output))?;
    file.write_all(format!("{:#?}", ast).as_bytes())
}

fn read_io(dir: &str, file: &str) -> std::io::Result<(String, String, PathBuf)> {
    let here = here();
    let input = here.join(dir).join(format!("{}.vk", file)).canonicalize()?;
    let output = here.join(dir).join(format!("{}.ron", file)).canonicalize()?;
    if let Ok(o) = Url::from_file_path(&input) {
        println!("Parsing: {}", o)
    }
    let in_text = std::fs::read_to_string(&input)?;
    let out_text = std::fs::read_to_string(&output)?;
    Ok((in_text, out_text, output))
}

#[test]
fn reexport_all() {
    find_all("debug", true).ok();
    find_all("literal", false).ok();
    find_all("expression", false).ok();
    find_all("statement", false).ok();
    find_all("declaration", false).ok();
}

fn find_all(dir: &str, debug: bool) -> anyhow::Result<()> {
    let mut cache = SourceCache::default();
    let path = here().join(dir).canonicalize()?;
    if !path.is_dir() {
        panic!("{} must a directory", path.display())
    }
    for file in path.read_dir()? {
        let path = file?.path();
        match path.extension() {
            Some(s) if s.eq("vk") => {}
            _ => continue,
        }
        // set file path
        let file = cache.load_local(&path)?;
        if let Ok(o) = Url::from_file_path(&path) {
            unsafe {
                cache.set_source(file, o.as_str().to_string());
            }
            if debug {
                println!("Short Form: {}", o)
            }
        }
        // parse text
        let text = cache.fetch(&file)?.to_string();
        match ValkyrieParser::parse_cst(&text, ValkyrieRule::Program) {
            Ok(o) if debug => println!("{}", o),
            _ => {}
        };
        let out = ProgramContext { file }.parse(&mut cache);
        match out {
            Success { value, diagnostics } => {
                for error in diagnostics {
                    error.as_report().eprint(&cache)?
                }
                let mut out = File::create(path.with_extension("ron"))?;
                for statement in &value.statements {
                    out.write_all(format!("{:#?}\n", statement).as_bytes())?;
                }
            }
            Failure { fatal, diagnostics } => {
                for error in diagnostics {
                    error.as_report().eprint(&cache)?
                }
                fatal.as_report().eprint(&cache)?
            }
        }
    }
    Ok(())
}
