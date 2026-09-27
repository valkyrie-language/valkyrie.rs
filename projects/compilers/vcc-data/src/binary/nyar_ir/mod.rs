//! Nyar `.nyar` 外码格式合同。
//!
//! 实现位于 [`nyar_bytecode`]。本模块仅作兼容再导出，供仍通过 `vcc_data::binary::nyar_ir`
//! 路径引用的装配层代码过渡使用。新代码应直接依赖 `nyar-bytecode`。

pub use nyar_bytecode::*;
