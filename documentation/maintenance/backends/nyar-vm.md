# Nyar VM 执行边界

Nyar runtime 位于 nyar-vm.rs 的 runtimes 目录，消费 Nyar bytecode，不依赖语言 AST 或工程装配实现。

## 正式输入

```text
Compiler → Canonical + RepresentationPlan → BackendPrivatePlan
  → nyar-bytecode → verify → nyar-vm → export/invoke
```

nyar-bytecode 拥有格式和 opcode；nyar-vm 拥有解释执行；nyar-gc 拥有堆和回收；nyar-jit 是可选加速。JIT 不支持能力须返回 Unsupported，不得假成功。

导入按验证后的索引和签名执行，runtime 不按宿主函数名分派语言语义，也不回调 Compiler 修复调用。

## 验收

加载前验证版本、section、跳转、栈、局部初始化、import 和 layout；运行从本次非空 artifact 调用真实 export。解释器、GC 或 JIT 的局部测试不能替代全量题意结果。完整管线见 [编译器架构](../compiler-architecture.md)。
