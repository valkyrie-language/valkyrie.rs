---
name: update-change-logs
description: >-
  valkyrie.rs 版本发布说明闭环：生成 commit 级 reference、手工提炼
  `documentation/maintenance/releases/vX.Y.Z.md`、按需 `gh release edit` 同步 GitHub。
  用户提及 change-logs、release notes、更新 changelog、GitHub Release 时加载。
---

# Update Change Logs（发布说明）

在本仓， **更新 changelog** 指 **reference → 发布稿 →（可选）GitHub Release** 的完整闭环，不是把 reference 直接发布，也不是写「正式发布
npm」等元信息。

```text
① 生成 reference  →  ② 提炼发布稿  →  ③ 同步 GitHub Release（用户明确要求时）
   git-change-logs     releases/vX.Y.Z.md     gh release edit
```

## 前置：安装 `git-change-logs`

与 `git-reword` 相同，来自 [git-tools](https://github.com/oovm/git-tools)：

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-change-logs
```

验证：`git-change-logs --help`。本仓 `pnpm change-logs` 仅为同名快捷方式，**须**已安装并在 `PATH` 中。

## 路径与产物

| 路径                                                     | 用途                                    | 入库                |
|----------------------------------------------------------|-----------------------------------------|---------------------|
| `git-change-logs`（git-tools 全局 CLI）                  | commit 索引生成器                       | 否                  |
| `documentation/maintenance/release-notes.template.md`    | 发布稿模板                              | 是                  |
| `documentation/maintenance/author-github.json`           | 非 noreply 邮箱 → GitHub `id` / `login` | 是                  |
| `documentation/maintenance/releases/vX.Y.Z.reference.md` | 按 commit 分组的**对照稿**              | **否**（gitignore） |
| `documentation/maintenance/releases/vX.Y.Z.md`           | 面向用户的**发布稿**                    | 是                  |

## ① 生成 reference

在仓库根目录（或任意子目录）：

```text
git-change-logs --tags
git-change-logs --version X.Y.Z
git-change-logs --version X.Y.Z --write
git-change-logs --from vA.B.C --to vX.Y.Z
```

（等价：`pnpm change-logs …`，前提同上。）

- `--version X.Y.Z`：范围 = 上一个 `v*` tag .. `vX.Y.Z`（semver 回退兜底）。
- `--write`：写入 `documentation/maintenance/releases/vX.Y.Z.reference.md`（与发布稿同目录）。
- 输出按 gitmoji 分组：Features / Bug Fixes / Breaking / Other；每行 `- <subject> (@user)`。

### `v0.0.0` 特例

首个 tag 无 `--from` 时，reference 覆盖 **到该 tag 为止的全历史**，不能逐条照抄。只取与 tag 锚点 commit
一致、或用户可感知的里程碑；其余合并或跳过。

## ② 提炼发布稿

**必读**：`documentation/maintenance/release-notes.template.md` 与刚生成的 `vX.Y.Z.reference.md`。

**完成标准**：

1. 编辑 `documentation/maintenance/releases/vX.Y.Z.md`（ **不是** `.reference.md`）。
2. 保留模板全部分类：`## ✨ Features`、`## 🐛 Bug Fixes`、`## ⚠️ Breaking Changes`、`## 👥 Contributors`、`## 📝 Other`；无内容写「无」。
3. 正文中文；只写读者安装/使用后 **能感知**的变化。
4. `## 👥 Contributors`：从 reference 复制头像墙；头像优先 `https://avatars.githubusercontent.com/u/<id>?s=100`（
   `author-github.json` 的 `id` 稳定）。

### 提炼规则（硬性）

| reference 内容                        | 发布稿                                     |
|---------------------------------------|--------------------------------------------|
| Release / publish / bump 版本         | **跳过** — 发版本身不是 feature            |
| CI、reword、submodule、文档树整理     | **跳过**（或整节 Other 写「无」）          |
| 用户可感知的 API / CLI / npm 行为变化 | 合并为 1 条通俗中文                        |
| 同主题多条 commit                     | 合并为 1 条，不列 commit 清单              |
| reference 标在 Other 的维护项         | 默认不进发布稿；确属贡献者须知时才放 Other |

**禁止**：

- 把 reference 整段粘贴为 GitHub Release body。
- 写 hash、完整 commit 列表、安装命令（Release 页与 README 已有）。
- 写「通过 GitHub Actions 正式发布」「npm 上首次发布」等空话。

### 标题 emoji

发布稿首行**固定**与模板相同，无例外：

```markdown
# 🚀 `@valkyrie-language` vX.Y.Z
```

🐛 / ✨ / 🔧 只出现在正文 `## …` 小节标题里，**不得**替换首行火箭。

## 贡献者映射

`documentation/maintenance/author-github.json`：

```json
{
  "email@example.com": {
    "id": 12345678,
    "login": "handle"
  }
}
```

- `id`：稳定，用于头像与去重；`login` 可改。
- 兼容旧格式：`"email": "handle"` 字符串。
- noreply 邮箱自动解析：`{id}+login@users.noreply.github.com`。

缺映射且 reference 贡献者墙为空时，用 `git-change-logs lookup` 查 `id` / `login`（写入 `author-github.json`），再重新 `--write`：

```text
git-change-logs lookup --email aster@vers.site
git-change-logs lookup --login oovm
```

输出 JSON：`{ "id": …, "login": "…" }`。noreply 邮箱无需映射。自定义邮箱可设 `GITHUB_TOKEN` 后加 `--fetch`。

## ③ 同步 GitHub Release

**仅在用户明确要求时**执行；不要顺带 `git push` 或改 tag。

```text
gh release edit vX.Y.Z --notes-file documentation/maintenance/releases/vX.Y.Z.md
```

对多个版本逐条执行。推送 git 提交与更新 Release 是独立步骤，不要混为一谈。

## 自检

- [ ] `vX.Y.Z.reference.md` 已生成且未 `git add`
- [ ] `vX.Y.Z.md` 无发布废话、无 commit 清单
- [ ] 各分类齐全，空节为「无」
- [ ] Contributors 头像可显示（`id` 或已映射 `login`）
- [ ] （若要求）`gh release view vX.Y.Z` 与本地发布稿一致

## 相关入口

-
发布模板：[documentation/maintenance/release-notes.template.md](../../../documentation/maintenance/release-notes.template.md)
- 维护索引：[documentation/maintenance/index.md](../../../documentation/maintenance/index.md)
- 提交信息与 `git-reword`：[update-commit-messages](../update-commit-messages/SKILL.md)
- git-tools 文档：[change-logs.md](https://github.com/oovm/git-tools/blob/dev/documentation/change-logs.md)
