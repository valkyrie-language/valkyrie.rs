# Valkyrie Compiler Intermediate Representation Architecture

The Valkyrie compiler adopts a multi-layer intermediate representation (IR) architecture, providing efficient compilation and optimization infrastructure for the language. Through four layers of progressive transformation, Valkyrie can convert high-level language features into optimized target code, supporting various execution environments such as WebAssembly, JavaScript, and native code.

## Compiler Architecture Overview

### Design Goals

The Valkyrie compiler design focuses on:
- **Expressiveness Transformation**: Safely lowering complex functional and algebraic effect features
- **Multi-level Optimization**: Providing multi-layer optimization from high-level semantics to low-level instructions
- **Cross-platform Support**: Unified backend architecture supporting multiple target platforms
- **Development Toolchain**: Providing precise metadata support for IDEs and debuggers

### Compilation Pipeline

Valkyrie's compilation process is modeled as a progressive lowering from source code to machine code:

#### 1. Frontend (Oaks)
- **Input**: Source code
- **Output**: AST (Abstract Syntax Tree) -> HIR (High-level IR)
- **Responsibility**: Syntax parsing, name resolution, type checking, Trait specialization, pattern matching desugaring.
- **Implementation**: [oak-valkyrie](file:///e:/普遍优化/oaks/examples/oak-valkyrie)

#### 2. Intermediate Representation Lowering (Chomsky UIR)
- **Input**: HIR
- **Output**: UIR (Universal Intermediate Representation / IKun)
- **Responsibility**: Converts high-level semantic graphs into E-Graph-based universal intermediate representation, preparing for global optimization.
- **Implementation**: [valkyrie-compiler](file:///e:/普遍优化/valkyrie.rs/projects/valkyrie-compiler)

#### 3. Optimizer (Nyar VM / Chomsky)
- **Input**: UIR
- **Output**: Optimized UIR (IKun Tree)
- **Responsibility**: Whether in AOT or JIT mode, all core optimization tasks (constant folding, dead code elimination, loop optimization, etc.) are completed by the **Chomsky optimization engine** driven by **Nyar VM**.
- **Feature**: Based on E-Graph equality saturation technology, capable of discovering deep optimization opportunities that traditional compilers cannot reach.
- **Implementation**: [ProjectChomsky](file:///e:/普遍优化/ProjectChomsky)

#### 4. Backend Generation (Nyar VM / Gaia)
- **Input**: Optimized UIR
- **Output**: Target machine code (AOT) or memory executable code (JIT)
- **Responsibility**: Register allocation, instruction selection, stack frame management, and final code emission.
- **Implementation**: [nyar-vm](file:///e:/普遍优化/nyar-vm) and [project-gaia](file:///e:/普遍优化/project-gaia)

## Responsibilities of Each IR Layer

### [AST - Abstract Syntax Tree](file:///e:/普遍优化/oaks/examples/oak-valkyrie)

The AST layer serves as the compiler's entry point, receiving syntax trees from the parser and providing unified language feature abstractions.

### [HIR - High-level Intermediate Representation](file:///e:/普遍优化/valkyrie-compiler/projects/valkyrie-compiler)

The HIR layer is the core of semantic analysis, responsible for converting AST into intermediate representation with complete type and semantic information.

### [UIR/IKun - Universal Intermediate Representation](file:///e:/普遍优化/ProjectChomsky)

The UIR layer is the core of optimization, converting high-level semantics into E-Graph-based universal intermediate representation.

### [Nyar IR - Bytecode Intermediate Representation](file:///e:/普遍优化/nyar-vm/documentation/en-us/maintenance/nyar-ir.md)

The Nyar IR layer is the core of code generation, providing efficient bytecode abstractions for Nyar VM. See [nyar-ir](file:///e:/普遍优化/nyar-vm/documentation/en-us/maintenance/nyar-ir.md) for details.

## Core Value of the Compiler

Through this multi-layer architecture, Valkyrie ensures that code can achieve near-native execution performance while maintaining high-level abstractions. Developers can leverage cutting-edge features like algebraic effects without worrying about runtime performance costs, as the compiler optimizes them into efficient control flow through deep static analysis during the transformation process.
