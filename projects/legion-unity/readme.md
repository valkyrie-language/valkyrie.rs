# legion-unity

Unity 目标**可选**伴随 CLI。大多数 Valkyrie 用户不需要安装；Unity 游戏项目按需单独构建。

## 安装

```bash
cargo install --path projects/legion-unity
# 或与 legion 同目录 cargo build -p legion-unity
```

## 用法

```bash
legion-unity build
legion-unity export
legion-unity sync --unity-project ./unity
legion-unity status
```

已安装时，`legion` 会将 `legion unity …` 自动转发到 `legion-unity`（`legion unity build` ≡ `legion-unity build`）。

日常开发优先使用 Unity Editor 包菜单（Import MSIL / Rebuild）。
