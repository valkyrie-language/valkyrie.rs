# Native Backend Architecture

Valkyrie's native instruction set support (x86, x64, arm64, riscv) no longer relies on Cranelift or LLVM, but adopts a more powerful and fully self-controlled **Nyar VM + Project Gaia** architecture.

## Architecture Overview

Valkyrie's native compilation process is driven by the following core components:

### 1. Nyar VM (Core Runtime)
Nyar VM is Valkyrie's primary driver, responsible for coordinating the entire compilation and execution process.
- **AOT Compilation Driver**: Pre-compiles source code or bytecode into efficient target platform artifacts through `nyar-aot`.
- **JIT Execution Mode**: Supports dynamic machine code generation at runtime based on hotspot code.

### 2. Project Chomsky (Optimization Engine)
Replacing traditional LLVM optimization sequences, Chomsky adopts more modern optimization techniques.
- **E-Graph Equality Saturation**: Utilizes E-Graph-based equality saturation technology for extreme optimization.
- **IKun Intermediate Representation**: Unified intent representation, ensuring AOT and JIT share the same optimization logic.

### 3. Project Gaia (Multi-target Emitter)
Gaia is an extremely flexible backend system responsible for generating final executable files or libraries.
- **Multi-format Support**: Directly supports generating ELF, PE, WASM, JVM, CLR, and other formats.
- **Full-target Emission**: Capable of emitting machine code for x86, x64, ARM64, RISC-V, and other hardware architectures.
- **Ultimate Control**: Compared to LLVM, Gaia allows finer control over memory layout and instruction sequences, making it very suitable for OS kernel development.

## Why Choose a Self-controlled Architecture?

1. **Algebraic Effect Support**: Valkyrie's advanced features (such as Effect System) require special handling of stack and continuations at the底层; a self-controlled architecture can provide better adaptation.
2. **Optimization Potential**: E-Graph technology can explore optimization spaces that LLVM cannot reach.
3. **Lightweight and Portability**: Eliminates dependency on the bulky LLVM C++ library, making the entire toolchain more compact.
