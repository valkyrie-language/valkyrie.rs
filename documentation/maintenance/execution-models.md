# Valkyrie 执行模型

执行从本次正式编译的 artifact 开始。运行时、host wrapper、编译器与语言规范各自有明确边界；运行时不能回调工程编译器补事实。

## 编译与执行

```text
manifests/source → Compiler → Canonical / frozen plans
  → artifact + export/import + execution manifest
  → target runtime/glue → invoke → result
```

Nyar target 消费验证后的 Nyar 字节码；Wasm target 消费 Wasm 与签名驱动 glue。其他 target 的支持范围以实际构建、导出和执行证据判断。运行时不能按函数短名重建调用、类型或 ABI。

linker 绑定 adaptor/import，表示规划绑定 carrier/layout；backend 和 runtime 只能消费与验证。缺导入、签名、布局或载荷必须结构化失败，不能默认零值、默认签名或复制旧产物。

## 证据

版本输出、CLI 帮助、smoke 和单个 fixture 只证明各自路径。题意结果需要本次 artifact digest、非空 export、真实 invoke、非零且全部通过的 metadata.tests。

机制删除、合同测试、真实交付分别记录。本地 patch 或部分入口通过不能证明准确 pin 的全部入口通过。参见 [编译器架构](compiler-architecture.md)。
