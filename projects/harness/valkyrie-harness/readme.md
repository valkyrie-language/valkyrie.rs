# valkyrie-harness

语料 harness：**黑盒**经 [`@nyar/valkyrie`](https://www.npmjs.com/package/@nyar/valkyrie) 的 `legion` /
`asgard` 命令；**白盒**经 git 依赖的 `rusty-valkyrie` 程序化 API。

## 职责

| 路径 | 方式 |
| --- | --- |
| `tests/fixtures_plan.rs` | 黑盒：`legion plan` 子进程（`@nyar/valkyrie`） |
| `tests/fixtures_plan_whitebox.rs` | 白盒：`rusty_valkyrie::legion::plan_project` |
| `tests/oak/` | Oak 解析回归（`oak-valkyrie`） |

## 运行

```bash
pnpm install    # 仓库根目录；pnpm workspace 装配本包依赖
cargo test -p valkyrie-harness
```

## 依赖

- **pnpm**：本目录 [`package.json`](package.json) 声明 `@nyar/valkyrie`；见根 [`pnpm-workspace.yaml`](../../../pnpm-workspace.yaml)
- **Cargo**：`rusty-valkyrie` git `rev`（白盒）；本机兄弟仓见 [`.cargo/config.example.toml`](../../../.cargo/config.example.toml)

本机兄弟仓 `nyar-vm.rs` 开发时，可将 [`package.local.json.example`](package.local.json.example) 的 `link:` 依赖合并进 `package.json` 后重新 `pnpm install`。

可选 `VALKYRIE_LEGION` 覆盖黑盒 `legion` 入口。
