/// `JVM Class` 二进制格式模型。
pub mod class;

/// `COFF` 二进制格式模型。
pub mod coff;

/// Android `DEX` 二进制格式模型。
pub mod dex;
/// 各平台宿主可执行封装（PE / ELF / Mach-O）。
pub mod host_exe;

/// `ELF` 二进制格式模型。
pub mod elf;

/// Apple `Mach-O` 二进制格式模型。
pub mod mach_o;

/// x86-64 指令编码。
pub mod x86_64;

/// `JAR` 二进制格式模型。
pub mod jar;

/// `PE` 二进制格式模型。
pub mod pe;

// 已删除 `wasm` 模块：Wasm 编解码由 `acorn-wasm` 拥有，这里的副本与
// `acorn-wasm` 的 `decode` / `section` / `opcode` 重复。

/// TextVM `.tvm` 正则引擎产物格式。
pub mod tvm;

/// AArch64 指令常量。
pub mod aarch64;
