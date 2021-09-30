# 架构优化记录

## 目标

- 以渐进式方式降低编译驱动层与具体后端实现的耦合度。
- 为后续接入更多后端家族时减少中心分发代码修改面。
- 保持现有 `legion build` 行为与产物协议稳定。

## 当前优化清单

### 高优先级

1. `nyar-driver` 对 `CLR / WASM / native` 的编译流程使用集中式硬编码分支，新增后端时必须修改单一中心文件，扩展成本高。
2. `CLR` 家族已有独立 pipeline，而 `WASM / native` 仍在 driver 内直接串接 lane lowering 与 backend，家族职责边界不一致。
3. 上层 `legion` 虽已只依赖 `nyar-driver`，但 driver 内部仍聚合过多 family 细节，长期会重新形成新的“大编排文件”。

### 中优先级

1. `valkyrie-lsp` 仍直接持有独立编译路径，与主构建链复用不足。
2. `nyar` 的 `selection` / `backends` 抽象尚未完全用于 bundled family compiler 的注册与选择。
3. `JVM` 家族尚未接入 `nyar-driver` 的 bundled compiler 注册表。

### 低优先级

1. 多 crate 的 `missing_docs` 警告较多，影响大规模架构调整时的信噪比。
2. 一些 smoke / build 测试仍主要位于 `legion`，驱动层专用集成覆盖较少。

## 本轮实施方案

### 优化点

- 将 `nyar-driver` 的 bundled backend family 分发改成模块化适配器结构。

### 模块职责拆分

- `nyar-driver/src/lib.rs`
  - 仅保留稳定公开接口。
  - 暴露共享请求/响应模型。
  - 将 family 分发委托给注册层。
- `nyar-driver/src/families/mod.rs`
  - 维护 family compiler 注册表。
  - 根据 `TargetBackendFamily` 查找并执行对应编译器。
- `nyar-driver/src/families/clr.rs`
  - 承接 `CLR` bundled pipeline 适配。
- `nyar-driver/src/families/wasm.rs`
  - 承接 `WASM/WASI` lowering、backend 编译与运行契约生成。
- `nyar-driver/src/families/native.rs`
  - 承接 `native` lowering 与对象文件编译。

### 通信机制

- `legion -> nyar-driver`：传入 `DriverCompileRequest`。
- `nyar-driver -> family compiler`：按 `TargetBackendFamily` 做只读注册表查找。
- `family compiler -> backend crate`：调用对应 lane lowering 和 `TargetCodeGenBackend` 实现。

### 数据流

1. 上层产生 `HIR + LIR + target profile`。
2. `nyar-driver` 根据 family 选择适配器。
3. family 适配器完成各自的 lowering / backend 编译。
4. family 适配器回传统一 `DriverCompileReport`。
5. `legion` 只消费统一产物集合和运行契约。

### 预期收益

- 新增 bundled family 时，中心分发文件改动收敛到注册层。
- family 逻辑隔离，降低不同后端之间互相污染的风险。
- 后续把 `JVM` 接入 driver 时可直接复用现有 family compiler 模式。

## 已完成变更

### 代码结构

- 新增 `nyar-driver/src/readme.md`，明确 driver 分层职责。
- 新增 `nyar-driver/src/families/mod.rs`，引入 family compiler 注册层。
- 新增 `nyar-driver/src/families/clr.rs`。
- 新增 `nyar-driver/src/families/wasm.rs`。
- 新增 `nyar-driver/src/families/native.rs`。
- 精简 `nyar-driver/src/lib.rs`，只保留公共模型与统一入口。

### 行为兼容性

- `compile_with_bundled_backends()` 公开接口保持不变。
- `DriverCompileRequest` / `DriverCompileReport` 保持不变。
- `legion build` 的调用方式与产物协议保持不变。

## 验证记录

### 单元测试

命令：

```bash
cargo test -p nyar-driver
```

结果：

- `4` 个测试全部通过。
- 覆盖 family 注册完整性与 `WASM/WASI` 运行契约生成。

### 集成测试

命令：

```bash
cargo test -p legion --test cmds_build
```

结果：

- `6` 个构建集成测试全部通过。
- 覆盖 `CLR`、`node wasm`、`wasi`、`native` 实际构建链路。

### 性能观察

- 本轮调整属于控制面重构，不改后端编码热点路径。
- `legion` 构建集成测试整套执行完成，未出现功能性回退或明显异常耗时放大。
- 后续若继续推进 driver registry 泛化，再补专门的编排层压力基准。

## 回滚边界

- 本轮改动主要限定在 `nyar-driver` crate。
- 若需回滚，仅需恢复 `nyar-driver/src/lib.rs` 与 `src/families/*` 模块结构，不影响各后端 crate 内部表示。

## 下一阶段建议

1. 将 `JVM` 接入 `nyar-driver` family compiler 注册层。
2. 评估 `nyar::selection` 是否可上收为 driver 的统一 family/backend 选择入口。
3. 进一步收敛 `valkyrie-lsp` 与主编译链的重复编译路径。
