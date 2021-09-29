# Backend Maintenance Guide

This document introduces the compilation process and design considerations for various backends supported by the Valkyrie compiler.

## Overview

Valkyrie now adopts a unified, **Nyar VM**-centric compilation optimization pipeline. The core logic has evolved from traditional LIR linear transformation to E-Graph-based equality saturation optimization:

`Source -> AST -> HIR -> UIR (Chomsky) -> Optimized UIR (Nyar VM) -> Target`

This architecture allows us to delegate complex language feature optimization tasks to specialized optimization engines, while the frontend only needs to focus on semantic lowering.

## Backend Implementations

- [Nyar VM](nyar-vm.md): **Core Backend**.
    - **AOT Mode**: Emits static binaries through `NyarAot` and `Gaia`.
    - **JIT Mode**: Implements just-in-time compilation and hotspot optimization through `NyarJit`.
- [WASM Backend](wasi.md): Targets WebAssembly (WASI), utilizing Nyar VM's UIR emission adapter.
- [Native Backend](native.md): Now fully handled by the Nyar VM / Gaia architecture, providing native instruction set support.
- [JVM/CLR Backend](jvm.md): Traditional backends targeting stack-based virtual machines, typically skipping the LIR stage and generating instructions directly from CFG/UIR.

## Design Decisions

### 1. Unified Optimization Entry Point

Since 2026, all core optimization tasks (including inlining, escape analysis, dead code elimination, etc.) are completed by the Chomsky engine driven by Nyar VM. This avoids the maintenance cost of implementing optimization passes repeatedly across different backends.

### 2. Skipping Register Allocation for Stack Machines

For stack machine backends like JVM or CLR:
- **Rationale**: LIR is designed for register machines. Mapping SSA to LIR involves register allocation, which is unnecessary for stack machines.
- **Strategy**: Generating stack-based instructions directly from CFG or optimized UIR can more naturally utilize the operand stack.
