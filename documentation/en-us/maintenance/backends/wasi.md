# WASM Backend Maintenance Guide (WASI Preview 2)

The WASM backend is responsible for compiling Valkyrie to WebAssembly Component format, following the WASI Preview 2 standard.

## 1. Compilation Pipeline

`Source -> AST -> HIR -> UIR (Chomsky) -> Optimized UIR -> WASM Component`

The current implementation is transitioning from `CFG`-driven to `UIR`-driven. Optimized `UIR` intent trees are emitted as `WebAssembly Component`.

## 2. Design Considerations

### Stack Machine Architecture and Control Flow
- **Control Flow Mapping**: Utilize `UIR`'s structured intents or `CFG`'s linearization results, using `$dispatch` loops and `br_table` to map unstructured control flow to WASM structured control flow.
- **Local Variables**: Bindings in UIR or Locals in Cfg map to WASM's `local`.

### Types and Operations
- **Type Mapping**: `i32`, `i64`, `f32`, `f64`, `utf8`, `bool`, `unit`, etc.
  - Pointers, references, arrays, classes, etc. are uniformly mapped to `i32` (wasm32) or `i64` (wasm64) in WASM linear memory.
- **Arithmetic Operations**: Automatically select instructions based on operand types (e.g., `i64.add`, `f64.add`).
- **Constant Pool**: String constants are collected at compile time and stored in WASM `DataSection`.

### Memory Layout and Alignment
- **Linear Memory**: Built using `wasm-encoder`.
- **Structure Layout**: 
  - Fields are arranged in defined order.
  - **Memory Alignment**: Each field is aligned according to its type's natural alignment requirements.
- **Union Type Layout**: 
  - Uses `Tag + Payload` pattern.
  - Tag is `i32`, located at offset 0.
  - Payload follows immediately after.
  - **Variant Fields**: Supports variants with fields, mapped through component model's `variant` type.
- **Memory Management**:
  - **Heap Allocation**: Core module implements `cabi_realloc` function, following Canonical ABI standard.
  - **Allocation Algorithm**: Currently uses simple Bump Allocation (pointer bumping) algorithm, suitable for short-running or small scripts.
  - **Heap Pointer**: Uses WASM global variable to track current heap top, initial position immediately follows `DataSection` (constant pool).

### Component Model (Component Model / WASI P2)
- **Multi-module Architecture**: 
  - `MockMemory`: Responsible for exporting linear memory, serving as the single source of truth within the component.
  - `Main`: Core logic module, imports memory exported by `MockMemory`, and exports `cabi_realloc` for memory management.
- **Interface Docking**:
  - **wasi:cli/stdout**: Implemented interface import, supports obtaining standard output handle through `get-stdout`.
  - **wasi:io/streams**: Implemented `write` interface docking, supports writing byte sequences to output stream.
  - **Canonical ABI**: 
    - Uses `canon lower` to lower component-level functions (like `write`) to functions callable by core modules.
    - The lowering process is associated with memory provided by `MockMemory` to support `list<u8>` (utf8) type passing.
- **Instantiation and Linking**: Uses `ComponentInstanceSection` and `ComponentAliasSection` to complete module instantiation and linking within the component. Currently supports multi-level alias mapping, ensuring core modules can correctly identify and call lowered WASI functions.

## 3. Valkyrie Feature Handling

WASM backend is configured through `WasmConfig`:

- **Variant**: Supports `wasm32` and `wasm64`.
  - `wasm32`: Uses 32-bit address space, currently the mainstream choice.
  - `wasm64`: Uses 64-bit address space, suitable for scenarios requiring large memory support.
- **Effect Lowering**:
  - `experimental_stack_switch`: Boolean. If `true`, attempts to use WASM native `stack-switching` proposal (Option B); if `false`, falls back to more compatible CPS transformation (Option C).

### Traits and Polymorphism
- **Implementation Scheme**: Adopts classic VTable (Virtual Method Table) scheme.
- **Memory Layout**: Object header contains an offset pointing to VTable in linear memory. VTable stores function indices.
- **Calling Convention**: Uses `call_indirect` instruction to dynamically call functions based on indices in VTable.

### Algebraic Effects
One of Valkyrie's core features is algebraic effects, which is challenging to implement in WASM. With browser support for new proposals, the current plan is as follows:
- **Option A (Asyncify)**: Utilize Binaryen's `asyncify` tool to save and restore call stack in user space. (No longer the preferred option)
- **Option B (Stack Switching)**: Utilize WASM native `stack-switching` proposal. This is the optimal path for implementing resumable effects. Can be enabled with `experimental_stack_switch = true`.
- **Option C (CPS Transformation)**: Convert code with effects to Continuation Passing Style at compile stage. This is the default option (`experimental_stack_switch = false`).

**Current Status and Evaluation**: 
- `Raise`: 
    - **Non-resumable Path**: Given WASM `exception-handling` proposal has been implemented in browsers, we will prioritize using `throw` instruction to implement `Raise`. This makes effects behave as standard exceptions when not resuming.
    - **Resumable Path**: Depends on `stack-switching` or `CPS` transformation.
- `PushHandler` / `PopHandler`: Need to be implemented combining `try-catch` or `try_table` instructions.

### Memory Management
- **Linear Memory Model**: Currently implements a minimal Bump Allocator (`cabi_realloc`). Due to lack of `free`, only suitable for short-term tasks.
- **WASM GC Model**: Given WASM GC proposal has been implemented, GC object-based type expression support has been introduced.
    - **Current Progress**: 
        - Structures and arrays now support mapping to WASM's `structure` and `array` types.
        - Object allocation based on `struct.new` / `array.new_fixed` has been implemented.
        - Field and index access based on `struct.get` / `struct.set` and `array.get` / `array.set` has been implemented.
    - **Advantages**: Eliminates memory leaks, enhances security, and simplifies object lifecycle management in AE continuations.
- **Recommendation**: Parallel retention of linear memory model (for底层 FFI) and GC model (for Valkyrie native types). Can be enabled through `experimental_gc` configuration item.

### Aggregate Types (Structure/Array)
- Basic allocation has been implemented, but `emit_load` / `emit_store` does not yet support by-value copying of aggregate types (Memcpy).
- Array literals have lowering loss issues in `AST -> HIR` stage.
- **String (utf8)**: Will interface with WASI's component model string representation, directly encoding relevant type definitions in WASM binary.


### Component Model and Toolchain
- **Direct Construction**: We do not depend on external tools like `wit-component`. Component wrapping, type declarations (such as WIT-corresponding parts) required by WASI Preview 2 are all implemented by directly writing to WASM binary (Component Section).
- **Breaking Points**: `AST -> HIR` and `HIR -> CFG` stages have missing handling for certain complex expressions (such as arrays, closures).
- **Validation**: Continuously track fix progress through `wasi_test.rs`. Currently `arithmetic` passes, `structure/array/control_flow` still limited by above missing features.

## 4. Missing Features and Roadmap

### Core Features
- [ ] **Algebraic Effects (AE)**:
    - [ ] Implement `Raise`'s non-resumable path (map to WASM `throw` instruction).
    - [ ] Implement `PushHandler` / `PopHandler` (map to WASM `try-catch` or `try_table`).
    - [ ] Research and implement continuation resumption logic under `stack-switching` proposal.
- [ ] **Traits and Polymorphism**:
    - [ ] Design and implement VTable layout in linear memory.
    - [ ] Implement dynamic dispatch based on `call_indirect`.
- [ ] **Enum**:
    - [ ] Implement `Tag + Payload` linear memory layout.
    - [ ] Support variant expression in GC mode (possibly mapped to WASM `structure` subclasses or unions).

### Optimization and Enhancement
- [ ] **GC Mode Completion**:
    - [ ] Support GC array dynamic length allocation (`array.new`).
    - [ ] Support GC strings (`utf8.new_utf8`, etc.).
    - [ ] Implement bridge layer between GC objects and linear memory FFI.
- [ ] **Backend Architecture**:
    - [ ] Enhance `relooper` logic to support more complex control flow (such as labeled `break`).
    - [ ] Implement `memcpy` optimization for by-value copying of aggregate types in linear memory.

### Tools and Validation
- [ ] **Test Coverage**:
    - [ ] Fix missing handling for arrays and structure constructors in `AST -> HIR` stage.
    - [ ] Add more unit tests for GC mode and AE mechanism.
