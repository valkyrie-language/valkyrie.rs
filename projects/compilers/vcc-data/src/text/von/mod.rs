//! VON 文本格式 facade — 重导出 [`oak_von`]（权威 parser / serde 不在 `vcc-data`）。

pub type VonError = oak_core::OakError;
pub use oak_von::language::value::{from_ast, VonArray, VonEnum, VonField, VonObject, VonTuple, VonValue};
pub use oak_von::printer::{PrintOptions, PrintStyle};
pub use oak_von::{to_string, to_string_indented};

/// 解析 VON 文本为 [`VonValue`]（兼容旧 `VonParser::parse` 调用点）。
pub struct VonParser;

impl VonParser {
    /// 解析 VON 文本为 [`VonValue`]。
    pub fn parse(source: &str) -> Result<VonValue, VonError> {
        parse_value(source)
    }
}

/// 解析 VON 文本为 [`VonValue`]。
pub fn parse_value(source: &str) -> Result<VonValue, VonError> {
    let trimmed = source.trim();
    let ast = oak_von::parse(trimmed).map_err(VonError::custom_error)?;
    Ok(from_ast(&ast))
}

/// 将 VON 文本反序列化为 Rust 类型。
pub fn from_str<T>(source: &str) -> Result<T, VonError>
where
    T: serde::de::DeserializeOwned,
{
    oak_von::from_str(source.trim())
}

/// 将 [`VonValue`] 反序列化为 Rust 类型。
pub fn from_value<T>(value: VonValue) -> Result<T, VonError>
where
    T: serde::de::DeserializeOwned,
{
    let text = oak_von::printer::print_value(&value, PrintStyle::Compact, &PrintOptions::default());
    oak_von::from_str(&text)
}

/// 将 Rust 类型序列化为 [`VonValue`]。
pub fn to_value<T>(value: &T) -> Result<VonValue, VonError>
where
    T: serde::Serialize,
{
    parse_value(&to_string(value)?)
}
