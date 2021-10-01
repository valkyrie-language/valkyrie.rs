#![doc = include_str!("readme.md")]

use std::{
    fmt::{Display, Formatter},
    io::{Cursor, Read, Write},
};

use serde::{Deserialize, Serialize};
use zip::{result::ZipError, write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::class::{JvmClassError, JvmClassFile};

const MANIFEST_PATH: &str = "META-INF/MANIFEST.MF";

/// `JAR` 读写错误。
#[derive(Debug)]
pub enum JvmJarError {
    /// `ZIP` 读写失败。
    Zip(ZipError),
    /// 底层 `IO` 失败。
    Io(std::io::Error),
    /// `ClassFile` 编解码失败。
    Class(JvmClassError),
}

impl Display for JvmJarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zip(error) => write!(f, "JAR ZIP 读写失败：{error}"),
            Self::Io(error) => write!(f, "JAR IO 失败：{error}"),
            Self::Class(error) => write!(f, "JAR 中的 class 文件无效：{error}"),
        }
    }
}

impl std::error::Error for JvmJarError {}

impl From<ZipError> for JvmJarError {
    fn from(value: ZipError) -> Self {
        Self::Zip(value)
    }
}

impl From<std::io::Error> for JvmJarError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JvmClassError> for JvmJarError {
    fn from(value: JvmClassError) -> Self {
        Self::Class(value)
    }
}

/// `JAR` 归档入口。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct JvmJarEntry {
    /// 归档内路径。
    pub path: String,
    /// 入口内容。
    pub bytes: Vec<u8>,
}

/// `JAR` 产物模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct JvmJarPackage {
    /// 归档文件名。
    pub file_name: String,
    /// `MANIFEST.MF` 主类。
    pub main_class: Option<String>,
    /// 归档入口列表。
    pub entries: Vec<JvmJarEntry>,
}

impl JvmJarPackage {
    /// 创建一个新的 `JAR` 产物模型。
    pub fn new(file_name: impl Into<String>) -> Self {
        Self { file_name: file_name.into(), main_class: None, entries: Vec::new() }
    }

    /// 追加一个归档入口。
    pub fn push_entry(&mut self, path: impl Into<String>, bytes: Vec<u8>) {
        self.entries.push(JvmJarEntry { path: path.into(), bytes });
    }

    /// 将 `ClassFile` 编码后放入 `JAR`。
    pub fn push_class(&mut self, class_file: &JvmClassFile) -> Result<(), JvmJarError> {
        let path = format!("{}.class", class_file.internal_name.trim_end_matches(".class"));
        self.push_entry(path, class_file.to_bytes()?);
        Ok(())
    }

    /// 从归档中读取指定类。
    pub fn read_class(&self, internal_name: &str) -> Result<Option<JvmClassFile>, JvmJarError> {
        let target_path = format!("{}.class", internal_name.trim_end_matches(".class"));
        let Some(entry) = self.entries.iter().find(|entry| entry.path == target_path)
        else {
            return Ok(None);
        };
        Ok(Some(JvmClassFile::from_bytes(&entry.bytes)?))
    }

    /// 将 `JAR` 模型编码为二进制归档。
    pub fn to_bytes(&self) -> Result<Vec<u8>, JvmJarError> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            if let Some(main_class) = &self.main_class {
                writer.start_file(MANIFEST_PATH, options)?;
                writer.write_all(build_manifest(main_class).as_bytes())?;
            }

            for entry in &self.entries {
                if entry.path.eq_ignore_ascii_case(MANIFEST_PATH) && self.main_class.is_some() {
                    continue;
                }
                writer.start_file(&entry.path, options)?;
                writer.write_all(&entry.bytes)?;
            }
            writer.finish()?;
        }
        Ok(cursor.into_inner())
    }

    /// 从二进制归档解码出 `JAR` 模型。
    pub fn from_bytes(file_name: impl Into<String>, bytes: &[u8]) -> Result<Self, JvmJarError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        let mut package = Self::new(file_name);
        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;
            if file.is_dir() {
                continue;
            }

            let mut entry_bytes = Vec::new();
            file.read_to_end(&mut entry_bytes)?;
            if file.name().eq_ignore_ascii_case(MANIFEST_PATH) {
                package.main_class = parse_manifest_main_class(&entry_bytes);
            }
            else {
                package.push_entry(file.name(), entry_bytes);
            }
        }
        Ok(package)
    }
}

fn build_manifest(main_class: &str) -> String {
    format!("Manifest-Version: 1.0\r\nMain-Class: {main_class}\r\n\r\n")
}

fn parse_manifest_main_class(bytes: &[u8]) -> Option<String> {
    let manifest = String::from_utf8_lossy(bytes);
    manifest.lines().find_map(|line| line.strip_prefix("Main-Class:").map(|value| value.trim().to_string()))
}
