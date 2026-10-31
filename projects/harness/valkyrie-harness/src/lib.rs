#![warn(missing_docs)]

//! 语料 harness：黑盒经 [`@nyar/valkyrie`](https://www.npmjs.com/package/@nyar/valkyrie) 的
//! `legion` / `asgard` 命令；白盒经 git 依赖的 `rusty-valkyrie` 程序化 API。
//!
//! 本 crate 不重复实现编译器；测试见 `tests/`。

pub mod asgard_cli;
pub mod fixtures;
pub mod legion_cli;
