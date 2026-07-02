# `legion bootstrap`

实现 `seed -> v1 -> v2` 的诚实自举编译链，支持 `CLR`、`JVM` 与 `WASM` 目标。

## 工作流程

1. 使用 seed（已有的可运行 `legion`）编译自举目标项目（默认 `legion.tools` 规模项目目录），得到 v1。
2. 运行 v1 验证其可执行，并校验 `--version` / `--help`：
   - CLR：`dotnet exec`
   - JVM：本机 `java -jar`
3. 使用 v1 编译同一份项目源码，得到 v2（JVM seed 同样经 `java -jar` 启动）。
4. 比对 v1 与 v2：WASM 比对运行输出；CLR / JVM 优先比对 `run-contract.txt`。

## 自举目标

- 统一验收项目：`valkyrie.v/projects/legion._/projects/legion.tools`
- Rust 入口：
  - `legion bootstrap --project <legion.tools路径> --target clr`
  - `legion bootstrap --project <legion.tools路径> --target jvm`
- 发行版入口：`node valkyrie.v/scripts/bootstrap-clr.mjs` / `bootstrap-jvm.mjs`
- 能力契约见 `valkyrie.v/projects/legion._/projects/legion.tools/documentation/pages/zh-hans/bootstrap-contract.md`

## 二进制调试

产物反汇编与方法体检查统一走 **`legion spy`**，不要使用临时 mjs / `javap` 旁路：

```text
legion spy jvm <artifact.jar> --list
legion spy jvm <artifact.jar> --func demo/Main --method main
legion spy jvm <artifact.class> --method main
```

## 诚实自举护栏

- v1 必须实际读取源文件内容，输出依赖源文件。
- 禁止预编译模板、自复制、外部编译器代理、差分测试伪装。
- 禁止使用固定输入值代替真实源文件路径。
- 禁止通过 HostBridge、`LEGION_SEED` 或任何外部 bridge 委托 seed 完成 v1/v2 编译。

## `WASM` 目标补充（非 CLR 发布门）

- WASM seed 路径仅保留历史兼容；`micro_compiler` 作弊工程已废弃。
- CLR 源头自举以 `bootstrap-clr.mjs` 与 `bootstrap-smoke-clr.mjs` 为准。
