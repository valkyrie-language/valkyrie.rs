# Native 后端架构

Valkyrie 的原生指令集支持（x86, x64, arm64, riscv）不再依赖 Cranelift 或 LLVM，而是采用了更强大且完全自主掌控的 **Nyar VM + Project Gaia** 架构。

## 架构概览

Valkyrie 的原生编译流程由以下核心组件驱动：

### 1. Nyar VM (核心运行时)
Nyar VM 是 Valkyrie 的主要驱动力，负责协调整个编译和执行流程。
- **AOT 编译驱动**：通过 `nyar-aot` 将源码或字节码预先编译为高效的目标平台构件。
- **JIT 执行模式**：支持在运行时根据热点代码动态生成机器码。

### 2. Project Chomsky (优化引擎)
替代了传统的 LLVM 优化序列，Chomsky 采用了更现代的优化技术。
- **E-Graph 等价饱和**：利用基于 E-Graph 的等价饱和技术进行极致优化。
- **IKun 中间表示**：统一的意图（Intents）表示，确保 AOT 和 JIT 共享相同的优化逻辑。

### 3. Project Gaia (多目标发射器)
Gaia 是一个极其灵活的后端系统，负责生成最终的可执行文件或库。
- **多格式支持**：直接支持生成 ELF、PE、WASM、JVM、CLR 等多种格式。
- **全目标发射**：具备为 x86, x64, ARM64, RISC-V 等多种硬件架构发射机器码的能力。
- **极致掌控**：相比 LLVM，Gaia 允许对内存布局和指令序列进行更精细的控制，非常适合 OS 内核开发。

## 为什么选择自主架构？

1. **代数效应支持**：Valkyrie 的高级特性（如 Effect System）需要底层对栈和延续（Continuation）有特殊处理，自主架构能提供更好的适配。
2. **优化潜力**：E-Graph 技术能探索 LLVM 难以触及的优化空间。
3. **轻量化与可移植性**：摆脱了对笨重的 LLVM C++ 库的依赖，整个工具链更加紧凑。
