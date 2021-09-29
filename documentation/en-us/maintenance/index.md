# Valkyrie Virtual Machine Maintenance Guide

This guide is intended for **internal maintainers and core development team** of the Valkyrie Virtual Machine project, introducing project architecture, module responsibilities, internal maintenance processes, and system-level design decisions.

> **Target Audience**: Project maintainers, core development team members, system architects
> **Content Focus**: Internal architecture, maintenance processes, system design, code organization

## Project Architecture Overview

Valkyrie adopts a Rust Monorepo (Workspace) architecture, where each component is encapsulated in an independent crate, providing clear dependency relationships, independent testing environments, and efficient parallel compilation.

- **[Compiler Architecture](./compiler-architecture.md)**: Deep dive into Valkyrie's multi-layer IR architecture.
- **[Project Architecture](./project-architecture.md)**: Detailed directory structure and module descriptions.

```
valkyrie/
├── Cargo.toml         # Workspace root configuration
└── projects/
    ├── valkyrie-types/    # Unified intermediate representation type definitions (HIR, UIR/IKun)
    ├── valkyrie-compiler/ # Modern compiler framework based on Chomsky
    ├── nyar-vm/           # Nyar VM core and execution engine
    ├── valkyrie-error/    # Diagnostic system based on miette
    └── legion/            # Command-line tool (Valkyrie toolchain entry point)
```

External dependencies:
- `oak-valkyrie`: New frontend implementation (Lexer, Parser, AST), located at `../oaks`
- `ProjectChomsky`: Compiler backend optimization framework, located at `../ProjectChomsky`
- `project-gaia`: Multi-target instruction emitter, located at `../project-gaia`

## Core Design Philosophy

Valkyrie's architecture is built on five design pillars:

### 1. Modern Compilation Pipeline

Valkyrie's core design has evolved into a modern architecture centered on **Nyar VM**, completely decoupling optimization from backend generation:

#### Stage 1: Frontend (Oaks)
- **Responsibility**: Syntax parsing and high-level semantic processing.
- **Current Implementation**: Uses `oak-valkyrie` as the unified frontend.
- **Key Processing**:
  - **Symbol Resolution**: Builds cross-module symbol references.
  - **Type Checking & Inference**: Ensures language-level type safety.
  - **Pattern Matching Desugaring**: Converts complex `match` structures into decision trees.

#### Stage 2: Lowering (Chomsky UIR)
- **Responsibility**: Converts high-level semantics (HIR) into a universal, optimizable intermediate representation (UIR/IKun).
- **Key Processing**: Maps language-specific semantic primitives to universal UIR intents.

#### Stage 3: Optimization (Nyar VM / Chomsky)
- **Responsibility**: Executes global optimization, supporting both AOT and JIT.
- **Core Technology**: **E-Graph Equality Saturation**.
- **Advantage**: Unified optimization logic, no need to write optimization passes repeatedly for different backends.

#### Stage 4: Backend Emission (Nyar VM / Gaia)
- **Responsibility**: Emits code for specific targets (WASM, Native, JIT memory space).
- **Key Processing**: Register allocation, instruction selection.
  - **Stack Frame Optimization**: Reduces unnecessary push/pop operations.
  - **Instruction Scheduling**: Optimizes execution pipeline to reduce latency.

### 2. Uncompromising Developer Experience

- **Diagnostics as Dialogue**: Uses `miette` framework to provide IDE-level diagnostic experience
- **Uninterrupted Flow**: Achieves sub-second response through efficient compilation pipeline
- **Intuitive and Powerful Language**: Provides high-level abstractions such as algebraic effects, powerful pattern matching

### 3. Unity and Duality of Abstractions

Based on the duality of data and control:
- `match` expression: Decomposition and pattern matching on data
- `catch` expression: Capture and pattern matching on control flow (algebraic effects)

### 4. Duality of Execution Models

- **Dynamic Interpretation/JIT Execution**: Designed for development, debugging, and interactive environments, with built-in complete runtime (Nyar VM)
- **Static AOT Compilation**: Designed for production deployment, emits lightweight, efficient native binaries or WebAssembly modules through Gaia

### 5. Zero-Cost Abstraction

High-level abstractions should be as efficient as hand-written optimal low-level code after compilation.

### 6. Deterministic Resource Management

Valkyrie achieves deep integration of RAII and garbage collection through **Nyar VM**. Since we control the entire compilation process from source code to UIR for managed languages, we can provide the following features for managed languages:
- **Finalizer**: Managed objects automatically trigger their finalization logic at the end of their lifecycle (mapped to Rust's `Drop` trait or unified `Finalizer` trait at the底层).
- **Resource Safety**: Even in a GC environment, managed languages can safely and timely manage non-memory resources (such as FFI objects, file handles, etc.) like C++/Rust.

## Core Module Details

### oak-valkyrie: Compiler Frontend Implementation
**Responsibility**: Provides Lexer, Parser, and AST definitions, parsing source text into abstract syntax trees.

### valkyrie-types: Intermediate Representation Type Definitions
**Responsibility**: Centrally manages intermediate representation type definitions for HIR, UIR (IKun), and other stages.

### nyar-vm: Virtual Machine and Compiler Core
**Responsibility**: Implements lowering from HIR to UIR, and drives Chomsky optimization and Gaia backend generation.

### valkyrie-error: Unified Error Handling
**Responsibility**: Provides centralized error definitions and diagnostic information output.

## Maintenance Process

### Code Review Standards
1. **Architectural Consistency**: Ensure new code aligns with the five design pillars
2. **Error Handling**: Use the unified `valkyrie-error` system
3. **Performance Considerations**: Avoid unnecessary allocations and copies

### Debugging Guide
1. **Compiler Errors**: Check `valkyrie-error` diagnostic output
2. **Code Generation Issues**: Use `--dump-{ast,hir,cfg,ssa,lir}` options to dump intermediate layer output

---

## Design and Implementation Topics

- [Project Architecture Design](project-architecture.md)
- [Execution Models (Interpretation and Compilation)](execution-models.md)
- [Object Lowering](object-lowering.md)
- [Package Management and Symbol Resolution](package-management.md)
- [Miette-based Error Handling](error-handling.md)
- [Performance Optimization Strategies](optimization-strategies.md)
- [Backend Implementation and Considerations](backends/index.md)

---
This maintenance guide will continue to be updated as the project evolves.
