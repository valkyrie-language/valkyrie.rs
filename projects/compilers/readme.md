# Compilers

`projects/compilers/` 下的 Rust workspace 成员只保留语料 fixture：

| Crate | 职责 |
| --- | --- |
| `valkyrie-fixtures` | Valkyrie lexer/parser/builder/printer fixture 与 golden |
| `von-fixtures` | VON 文本格式 fixture 与序列化 golden |
| `voc-fixtures` | VOC（AWSL 组件）fixture |

编译实现、CLI 与 host 绑定位于 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs)，不在本仓重复实现。
二进制与文本格式编解码由 Acorn / Oak 拥有（[acorn.rs](https://github.com/yggdrasil-language/acorn.rs) /
[oak.rs](https://github.com/yggdrasil-language/oak.rs) git 依赖）。
