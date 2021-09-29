# Valkyrie Execution Models

## 1. Overview

Valkyrie is designed to support multiple execution environments, from high-performance production environments to highly interactive development environments. The core execution logic is driven by **Nyar VM**.

## 2. Modern Execution Architecture

Valkyrie adopts a **Unified Intent Backend Architecture**. All code is first lowered to Chomsky UIR, then distributed by Nyar VM based on execution mode:

```mermaid
graph TD
    A[Chomsky UIR] -->|Nyar VM| B{Execution Mode};
    B -->|JIT| C[Dynamically generate machine code and execute];
    B -->|AOT| D[Generate static binary files];
    B -->|Interpreter| E[Intent-based interpretation (for debugging)];
```

### 2.1 JIT Mode (Just-In-Time)
- **Scenario**: Development debugging, high-performance script execution.
- **Mechanism**: Nyar VM analyzes UIR hotspots in real-time, invokes the `Gaia JIT` engine to emit UIR intents directly to memory and execute them.
- **Advantages**: 
    - Combines the flexibility of dynamic languages with the high performance of native code.
    - Supports Hot Reloading.

### 2.2 AOT Mode (Ahead-Of-Time)
- **Scenario**: Production environment deployment, WASM module distribution.
- **Mechanism**: Uses `NyarAot` to statically scan the entire UIR intent tree, applies deep global optimization, then emits target platform machine code or bytecode (such as WASI, x86_64) through `Gaia`.
- **Advantages**: 
    - Zero startup overhead.
    - Extreme binary size optimization.

## 3. Nyar VM Core Features

Regardless of the execution mode, Valkyrie shares runtime capabilities provided by Nyar VM:

- **E-Graph-based Global Optimization**: All AOT and JIT optimizations are driven by the built-in Chomsky engine.
- **Native Algebraic Effect Support**: Nyar VM implements efficient effect handlers and continuation capture at the底层.
- **RAII and GC Integrated Memory Model**: Provides native RAII (Resource Acquisition Is Initialization) support for managed languages. Through NyarVM's compilation pipeline control, all managed objects trigger their corresponding finalization logic when reclaimed by GC (mapped to Rust's `Drop` semantics at the底层), ensuring deterministic release of non-memory resources such as files and network connections.
- **Unified Cost Model**: Developers only need to define one backend cost model to benefit from both AOT and JIT optimization.

## 4. Legacy LIR/SSA Models

Early Valkyrie planned to use multi-layer linear lowering (SSA -> LIR), but has now fully transitioned to the **Chomsky UIR + Nyar VM** architecture to achieve deeper global optimization.

- **Why Deprecated**: 
    - Traditional SSA/LIR optimization passes have fixed order, making it difficult to discover cross-phase equivalence optimization opportunities.
    - Maintaining multiple backend emission logic (WASM, Native, VM) is too costly.
    - Nyar VM's equality saturation technology provides a higher optimization ceiling.
