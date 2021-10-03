//! Node-API（N-API）原生绑定层：把 `legion` / `asgard` 编译器能力导出给 Node 宿主。
//!
//! 用户面对的 `legion` / `asgard` 命令行二进制在 `packages/legion` 与 `packages/asgard`，
//! 由本 crate 与各 `packages/vcc-*` 平台 collect 组装而成。

#![warn(missing_docs)]

pub use asgard;
pub use legion;

/// Node 宿主占位：后续在此挂 `#[napi]` 导出（build / check / spy 等）。
pub fn napi_placeholder() -> &'static str {
    "vcc-napi"
}
