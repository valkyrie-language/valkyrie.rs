# legion

Valkyrie 工作区 CLI：清单解析、依赖解析、构建与目标选择。

## 职责

- `legion` 命令行入口
- 工作区与 `legion.von` / `legions.von` 解析
- 构建计划与目标编排

编译器语义与后端降低在 `vcc-data` 与 nyar-vm git 依赖中实现；本 crate 负责装配与调度。

## 构建

```bash
cargo build -p legion --release
cargo test -p legion
```

npm 装配见 [`projects/packages/legion`](../../packages/legion/README.md)。
