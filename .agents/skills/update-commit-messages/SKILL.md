---
name: update-commit-messages
description: >-
  valkyrie.rs 提交信息与历史改写：gitmoji 规范、批量 `git-reword` 用法、
  `reword.pending.json` 工作流、安装 git-tools。用户提及 reword、改 commit message、
  提交规范、git-fix-message 时加载。
---

# Update Commit Messages（提交信息）

本仓 **禁止** 添加或恢复 `scripts/reword.mjs`。批量改历史 message 一律用全局 **
`git-reword`**（[git-tools](https://github.com/oovm/git-tools)）。

发布说明（release notes）另见 [`update-change-logs`](../update-change-logs/SKILL.md)。

## 提交规范（gitmoji）

### 格式

- **subject 与 body 用英文**。注释、readme、skill 正文仍用中文。
- subject **必须以一个真实 gitmoji 字符开头**（如 ✨ 🐛 📝 ♻️ ⬆️ 🔧 🧪）。禁止 `?`、Conventional Commit 前缀、描述词冒充 emoji、多个
  emoji。
- subject **末尾禁止句号** `.`。body 句子正常用句号。
- 全文禁止 `;` 与 `；`（subject 与 body，改用句号或分行）。
- **标识符必须反引号**：slug、crate、路径、模块、字段、函数名，如 `` `legion.von` ``、`` `scripts/change-logs.mjs` ``、
  `` `nyar-vm.rs` ``。
- **禁止含糊缩写**：写全称（约定俗成的 `BFS`/`DFS` 可保留；指 TypeScript 时写 `TypeScript`，勿写 bare `TS`）。
- **版本号不进 subject**（如 `0.0.3`）；改用「patched releases」等表述，或写行为而不写号。
- **禁止内部计划/里程碑代号**：`Phase 1`、`M0`、`Gate-N` 等一律不进 commit message。

### 常见 gitmoji

| Emoji | 用途                |
|-------|---------------------|
| ✨    | 新能力              |
| 🐛    | 缺陷修复            |
| 📝    | 文档 / release 文稿 |
| ♻️    | 重构                |
| 🔧    | 配置、脚本、维护    |
| ⬆️    | 依赖或包版本 bump   |
| 🧪    | 测试                |
| 👷    | CI                  |
| 💥    | Breaking change     |

### 示例

```text
🐛 Fix VCC wasm capability assembly for npm publish

📝 Add `scripts/change-logs.mjs` and maintainer release notes under `documentation/maintenance`

🔧 Point workspace at `projects/compilers` and `nyar-vm.rs` git deps
```

### UTF-8 落盘（Windows）

**禁止**用 PowerShell here-string / 控制台默认编码写带 emoji 的 `git commit -m`（易变成 `?`）。

**推荐**：

- 用编辑器或 Node 写 UTF-8 的 `reword.pending.json` / 临时 message 文件，再 `git commit -F`。
- 批量历史改写用 **`git-reword`**，不要手写 `git rebase -i`。

## 安装 `git-reword`

命令找不到时，从 **git-tools** 安装（Rust 工具链需已就绪）：

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-reword
```

本机开发 git-tools 时：

```bash
cargo install --path <git-tools-root> --force
```

验证：

```bash
where git-reword    # Windows
which git-reword    # Unix
git-reword --help
```

仍找不到：确认 `~/.cargo/bin`（或 `%USERPROFILE%\.cargo\bin`）在 `PATH` 中。

上游仓库：<https://github.com/oovm/git-tools>（`git-reword` 子命令）。

## 批量改写工作流

1. **工作区干净** — `git status` 无未提交改动（submodule 脏指针若仅本地探测，先确认是否阻塞改写）。
2. **选定 base** — 独占基线 commit；改写范围为 `(base..ref]`。
    - 从 `ABC` **之后**改起：`--base ABC`（不改写 `ABC` 本身）。
    - **包含** `ABC`：`--base ABC^`。
3. **导出模板**（可选，也可手写 JSON）：

```bash
cd <repo-root>
git-reword export --base <exclusive-base> --ref dev --path reword.pending.json
```

4. **编辑 map** — 只保留需要改的 commit（`reword.pending.json` 已在 `.gitignore`）：

```json
{
  "version": 1,
  "entries": [
    {
      "hash": "14a101902ffabf7315fa8719826f6fccd843837e",
      "message": "📝 Add `scripts/change-logs.mjs` and maintainer release notes under `documentation/maintenance`\n"
    }
  ]
}
```

扁平 `{ "abc12345": "message" }` 也可。

5. **Dry-run**：

```bash
git-reword rewrite --base <exclusive-base> --ref dev --path reword.pending.json --dry-run
```

6. **执行**：

```bash
git-reword rewrite --base <exclusive-base> --ref dev --path reword.pending.json
```

7. **核对** — `git log --oneline <exclusive-base>..dev`

8. **推送** — **仅用户明确要求时**；历史已改写需 `git push --force-with-lease origin dev`。禁止对 `master`
   force-push，除非用户明确授权。

## 常见错误对照

| 问题                 | 反例                     | 正例                                                   |
|----------------------|--------------------------|--------------------------------------------------------|
| 无 gitmoji           | `add change log`         | `📝 Add \`change-logs\` script`                        |
| Conventional Commits | `fix: wasm assembly`     | `🐛 Fix wasm capability assembly`                      |
| 标识符无反引号       | `Fix legion.von parsing` | `` Fix `legion.von` parsing ``                         |
| PowerShell 乱码      | `? Add script`           | 用 UTF-8 JSON + `git-reword`                           |
| subject 带版本       | `` Bump to `0.0.3` ``    | `` Bump `@valkyrie-language/*` npm package versions `` |
| 分号                 | `Fix a; update b`        | 两句或分行                                             |

## 禁止

- 在本仓新增 `scripts/reword.mjs` 或 `reword.pending.txt` 流程。
- 能用 `git-reword` 时仍用 `git rebase -i` 批量改 message。
- 未经用户确认 force-push 已推送过的 commit。
- 在 `master` 上改写历史（除非用户明确要求）。

## 相关

- 发布说明闭环：[update-change-logs](../update-change-logs/SKILL.md)
- 维护索引：[documentation/maintenance/index.md](../../../documentation/maintenance/index.md)
