# Valkyrie 编译器架构

本文固定生产编译协议与事实所有权。接口合同不证明某个 target 已完成；能力须由对应修订的正式构建与真实执行证明。

## 生产入口与所有者

语言拥有的 IR 或解释器专门化 residual → 语言与目标适配器 → 目标私有计划与 LIR → 验证与编码 → ArtifactStore。

Nyar 提供可复用变换、分析、优化、部分求值及残余化工具。Valkyrie MIR、复杂控制流、效果和实例验证归 Valkyrie；历史 `Canonical MIR` 不能成为其他语言的公共前提。共享算法协议不等于统一语言操作枚举。

Python、Lua 和动态 TypeScript 由语言包拥有解释器 `I_L`。LegendVM 提供 guest 解释与专门化支撑；第一投影 `S(I_L, P) = R_L,P` 保留未知运行输入及必要动态 runtime，不能以编译期执行删除可观察效果。这是职责合同，不宣称这些语言或第二、第三投影已经实现。

Wasm、CLR、JVM、OVM、native 各自拥有计划、包装及 LIR。普通函数与合成包装共用 LIR 编码。OVM 产出 `.odin` 由 OdinVM 加载、验证和执行，并由其管理 VM 对象、执行帧、GC 与可选 JIT；其他目标不经 `.odin`。native 的 `.node` 和 `.pyd`/扩展 `.so` 分别使用 Node-API 和 CPython bridge 加 native 链接，明确注册、值、错误、资源和线程合同。


Legion 只编排 WorkspaceResolver、Compiler、ArtifactStore；不直接构造 AST/HIR/MIR 或语义选择结果。Compiler 不自行读取工程文件。

Oak 生产语法树、恢复状态和 syntax diagnostics。Valkyrie frontend 生产名称、类型、泛型替换、trait/imply evidence、调用与 package Semantic MIR；实际编译工具链在 `nyar-vm.rs`，本仓维护夹具与符合性入口，不拥有编译器实现。Nyar 工具不能反向生产 Valkyrie 语义。Acorn 负责外部格式编解码。

Nyar 可以托管多个语言 frontend。frontend adapter 按需把自己的语义 IR 或其他明确 contract 交给选定的 Nyar 模块；Nyar 不要求所有语言共享同一套 AST/HIR/语义 IR，也不建立为了兼容所有 frontend 的 Universal IR。某个 frontend 的私有类型规则不能升级为平台公共语义。target 优化可以选择 specialization、witness 消除或内联，但语言层不以单态化作为泛型语义。

## 声明绑定与语义需求

inventory 是完整来源集合；声明索引提供签名、导出、类型与可见性；语义需求决定物化哪些 body。索引、绑定、类型与实例/evidence 需求协作到固定点，不能先全库 body lowering 再裁剪。

每个物化 body 都须有入口 export 经声明/实例/evidence 到该 body 的合法需求链。短名恰好唯一不证明可见，文件被选中不证明同文件所有 body 可达。无计划、显式空需求和非空需求未物化分别处理。

## 调用、类型与表示

AST operator 在 HIR 展开为普通 callable。HIR/MIR/LIR 不建立 unary/binary/operator 语义类别；用户函数、std、trait/imply 和展开后的 operator 共用实例解析，Semantic MIR 用 Invoke(callee, arguments)。

ItemInstanceId 关联声明、完整 substitution、签名及 evidence。字段/变体引用所属完整名义实例。同声明不同实参不能混用，Self、Cast 目标及关联类型等全部类型位共用绑定事实。

linker 选择 target-conditioned adaptor/import；只有接入相应模块时才生成 RepresentationPlan 和 TargetExecutionPlan。RepresentationPlan 是稳定语义 ID keyed 的稀疏侧表，不复制 CFG。语言与目标适配器产生私有计划和 LIR，backend 只编码已验证的目标 LIR，不能按 callee、field、类型名或 carrier 补语义。

## 恢复、诊断与工具

Oak 同时返回可用树、recovery/missing/error 与 diagnostics，保留后续有效声明。恢复不能缺 token 补普通值，parser 不检查 place/type，Compiler 按可达语义决定准入。

各阶段保留 stage/code/owner/source/span，Compiler 汇总诊断。Lint 不另设语言合法性；工程 check/LSP 共享工程快照。源码 formatter 由 Oak CST/token-gap 生成 edits，Nyar PP document 仅打印模型，不替代源码格式化。

## 验收

修复前写清行为、缺事实、唯一生产者、消费接口和失败方式。测试成对覆盖可达/不可达、同名不同 owner/namespace、未声明依赖、跨包调用、实例与 evidence。缺身份必须结构化失败。

分别记录机制删除、合同成立和真实交付证据。局部测试、错误文本变化、枚举删除和字符串扫描均不替代 artifact/export/invoke 及题意测试。删除须迁移消费者、删除依赖与实现并验证旧生产入口不可达。
