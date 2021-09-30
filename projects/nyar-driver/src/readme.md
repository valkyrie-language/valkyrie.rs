# nyar-driver

`nyar-driver` 是 bundled backend 的统一驱动门面。

## 职责
- 接收上层已经完成目标 lane 选择的 `LIR` 与前端上下文。
- 按后端家族分发到对应的 bundled compiler。
- 汇总 `ArtifactSet`、入口点与运行契约，返回给 `legion` 这类编排层。

## 分层原则
- `src/lib.rs` 只保留稳定公开接口与共享请求/响应模型。
- `src/families/*` 负责单个后端家族的编译细节，不把不同家族的实现混在同一个文件。
- 新后端家族接入时，优先新增独立 family compiler，而不是继续扩写中央分发逻辑。

## 当前布局
- `families/clr.rs`：`CLR` 的 bundled 编译链适配。
- `families/wasm.rs`：`WASM/WASI` 的 lane lowering、产物生成与运行契约。
- `families/native.rs`：`native` 的 lane lowering 与对象文件输出。
- `families/mod.rs`：family compiler 注册与查找。
