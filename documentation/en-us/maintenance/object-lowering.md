# Valkyrie Compiler: Lowering Guide

## 1. Lowering Philosophy: From Semantics to Intents

The core architecture of the Valkyrie compiler has evolved from traditional "progressive lowering" to **Intent Mapping**. The core goal is to lower high-level language features into a universal intermediate representation (UIR/IKun) that **ProjectChomsky** can understand.

```mermaid
graph TD
    subgraph Frontend (valkyrie-compiler)
        A[Source Code] -->|Parse| B(<b>Oaks AST</b>);
        B -->|Semantic Analysis| C(<b>HIR</b><br><i>Types, Scopes, Traits</i>);
    end

    subgraph Mid-end (Lowering & Optimization)
        C -->|<b>UIR Lowering</b>| D(<b>Chomsky UIR</b><br><i>Intent Graph, IKun Tree</i>);
        D -->|<b>Equality Saturation</b>| E(<b>Optimized UIR</b><br><i>Nyar VM / Chomsky</i>);
    end

    subgraph Backends (Nyar VM / Gaia)
        E -->|AOT Emission| F[<b>Native Binary</b>];
        E -->|JIT Execution| G[<b>Memory Execution</b>];
        E -->|WASI Export| H[<b>WASM Module</b>];
    end
```

## 2. Modern Lowering Process

To fully leverage Nyar VM's optimization capabilities, Valkyrie adopts a unified lowering path:

### 2.1 Advantages of Lowering to UIR

| Feature | Traditional (SSA/LIR) | Modern (Chomsky UIR) |
| :--- | :--- | :--- |
| **Control Flow** | Explicit jumps/basic blocks | Declarative intents (If/Loop Intents) |
| **Optimization Timing** | Fixed-order passes | Cost model-based global equality saturation |
| **Backend Adaptation** | Need to write code emission for each backend | Unified drive by Gaia, backends only need to define cost models |

## 3. Feature Lowering Example: Pattern Matching

Pattern matching is a core feature of Valkyrie. We will trace how it lowers to Chomsky intents.

### 3.1 AST -> HIR
- **Pattern Parsing**: Identify nested patterns and guards.
- **Type Binding**: Assign types to each pattern component.

### 3.2 HIR -> UIR (Lowering)
- **Decision Intents**: Convert `match` into a series of nested `Select` intents.
- **Data Flow Mapping**: Map variable bindings in patterns to `Define` or `Bind` nodes in UIR.
- **Exhaustiveness Checking**: Still completed in the HIR stage, ensuring the generated UIR tree is logically complete.

### 3.3 UIR Optimization (Chomsky)
- **Branch Folding**: If the matcher is a constant, Chomsky will directly eliminate unnecessary branches through equivalence rewriting.
- **Equivalence Merging**: If multiple branches have the same execution intent, they will be merged in the E-Graph.

## 4. Feature Lowering Example: Control Flow & Effects

Valkyrie's high-level control flow (such as exception handling, async, and algebraic effects) is implemented through **Nyar VM**'s native continuation support.

### 4.1 HIR -> UIR
- **Effect Lowering**: Map `try/catch` to UIR's `EffectScope` and `Handle` intents.
- **Continuation Capture**: Materialize `raise` and `resume` as UIR's `Continuation` calls.

### 4.2 Nyar VM Execution/Generation
- **AOT Mode**: Nyar VM maps control flow to efficient state machine implementations for the target platform (such as CPS transformation or lightweight threads).
- **JIT Mode**: Directly leverage Nyar VM's native coroutines and continuation handlers, reducing context switching overhead.
