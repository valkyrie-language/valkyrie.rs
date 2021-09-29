# Scope and Automatic Lifecycle Insertion

Valkyrie abandons obscure and cumbersome explicit lifetime annotations, instead achieving fully automatic AIFD lifecycle management through **Lexical Scope**, **Parameter Qualifiers**, and deep **Static Control Flow Analysis**.

## Determination Core: Lexical Scope

In Valkyrie, curly braces `{}` define a rigorous physical boundary and execution timeline. The compiler uses this as a baseline to precisely orchestrate resource birth and death.

### 1. Allocate and Initiate
When a variable is bound within a scope, the compiler immediately inserts code for A and I phases.
- **Value Types (`structure`)**: Typically manifested as stack pointer movement, achieving near-zero-cost allocation.
- **Reference Types (`class`)**: Triggers managed heap memory allocation request and executes construction logic.

### 2. Finalize and Delocate
The compiler automatically injects necessary destruction logic at scope end, or all possible early exit paths (like `return`, `break`, `throw`).

```valkyrie
{
    let a = Point { x: 1.0, y: 3.0 }
        # Compiler automatically inserts p.finalize() and arranges memory reclamation
    }
}here
        return
    }
}
# When scope exits normally, compiler likewise automatically ensures destruction code execution
```

---

## Ownership Management: Qualifiers and Default Policies

Valkyrie introduces qualifiers (`ref`, `mut`, `own`) to precisely define function ownership of parameters. To balance performance and development experience, the compiler tailors intuitive default policies for different types.

### 1. Explicit Qualifier Semantics

- **Reference Mode (`ref` / `mut`)**: Represents **Borrowing**. No ownership transfer occurs. The original scope retains responsibility for object destruction after function call ends.
- **Ownership Mode (`own`)**: Represents **Taking Ownership**. **Move** occurs. Object lifecycle termination responsibility completely transfers from caller to callee.

### 2. Implicit Ownership Policy

| Type Category | Default Qualifier | Core Design Intent |
| :--- | :--- | :--- |
| **Structure** (Value Type) | **`own`** | Ensures value semantic independence, completely eliminating hidden cross-scope side effects. |
| **Class** (Reference Type) | **`ref`** | Provides transparent and natural sharing experience, conforming to managed object programming intuition. |

```valkyrie
micro process(d: Data)   # Implicitly equivalent to (own d: Data)
micro display(p: Person) # Implicitly equivalent to (ref p: Person)
```

### 3. Design Depth: Eliminating "Implicit Shared Mutability" Trap

Valkyrie's default policy aims to fundamentally eliminate ownership ambiguity hazards common in dynamic languages like Python and JavaScript.

#### Scenario A: Side Effects from Shared Mutable State
In traditional managed languages, simple assignments often introduce dangerous implicit sharing:
```python
# Python example: Implicit sharing causes data to be unexpectedly modified
a = [1, 2]
b = a
a.append(3) 
print(b) # Outputs [1, 2, 3] —— b's content changed without developer awareness
```

**Valkyrie's Defense Mechanism**:
1. **For Class (Reference Type)**: Defaults to **`ref` (read-only reference)**, strictly following **Read-Write Mutual Exclusion Principle**.
    ```valkyrie
    let mut a = [1, 2]
    let b = a        # b automatically gets ref [] permission
    a.push(3)        # Compile error! Cannot modify 'a' because 'b' is borrowing it in read-only mode
    ```
2. **For Structure (Value Type)**: Defaults to **`own` (Ownership Move)**.
    ```valkyrie
    let a = Point { x: 1, y: 2 }
    let b = a        # a's ownership has completely moved to b, a is invalid in current scope
    print(a.x)       # Compile error! 'a' has been marked invalid, cannot access again
    ```

#### Scenario B: Mutable Default Parameter State Persistence
Another classic trap is function default parameter state persistence across calls:
```python
# Python example: Default parameter becomes global state
def add_to(item, list=[]):
    list.append(item)
    return list

print(add_to(1)) # [1]
print(add_to(2)) # [1, 2] —— Default list is shared between calls!
```

**Valkyrie's Defense Mechanism**: Adheres to **Default Immutability** and **Explicit Permission Authorization**.
1. **Parameters Default to Read-Only**: Whether `class` or `structure`, parameters are read-only by default when passed.
    ```valkyrie
    micro add_to(item: i32, list: [i32] = []) {
        list.push(item) # Compile error! 'list' has read-only permission by default
    }
    ```
2. **`mut` Cannot Have Default Values**: If a function needs to modify passed state, it must explicitly declare `mut`, and the compiler strictly prohibits setting default values for such parameters.
    ```valkyrie
    # Recommended style: Callers can immediately identify potential side effects
    micro update(mut data: Data) { ... }

    # Compile error! Parameters with 'mut' marker cannot have default values, must be explicitly passed by caller
    micro wrong(mut list: [i32] = []) { ... }
    ```

**Design Philosophy Summary**: The "owner" of mutable state must always be clear. Valkyrie mandates: **Any operation involving state modification must be authorized by the caller through explicit permissions, ensuring the entire ownership chain is transparent, traceable, and complete.**

---

## Core Decision Guide

- **Only need read-only access?** Use default `ref` (reference types) or explicit `ref` (value types).
- **Need in-place modification?** Explicitly declare `mut` qualifier.
- **Need to take over object or move into container?** Use default behavior for value types, use `own` for reference types.
- **Quick Memory**: Value types (`structure`) move by default, reference types (`class`) share by default.

---

## Deep Static Analysis: Beyond Lexical Boundaries

When execution flow crosses simple lexical boundaries, the compiler ensures lifecycle management rigor through high-order static analysis.

### 1. Control Flow Awareness (CFG) and Linear Type Tracking

The Valkyrie compiler ensures every object with ownership must be "consumed" exactly once on any possible execution path by building a **Control Flow Graph**.

#### Deterministic Unwinding
In complex logic involving `?` error propagation, early `return`, or `throw`, the compiler real-time calculates the "Live Set" at current execution point and automatically inserts destruction code strictly in reverse order of initialization.

```valkyrie
micro process_file() -> Result {
    let f1 = File::open("a.txt")? # If fails, nothing happens
    let f2 = File::open("b.txt")? # If fails, automatically inserts f1.finalize()
    let res = compute(f1, f2)?    # If fails, automatically inserts f2.finalize(), f1.finalize()
    Success
}
```

#### Branch Merging and Ownership Compensation Mechanism
In `if` or `match` branches, if different paths result in inconsistent ownership states, the compiler automatically "compensates" before merge points to align states:

```valkyrie
let x = Data::new()
if condition {
    consume(x) # Path A: x's ownership moved to consume function
} else {
    # Path B: x is still alive
    # Compiler automatically inserts x.finalize() here
    # Ensures after merge point, x is invalid on all paths
}
# Merge point: x state is consistent (both invalid), subsequent access strictly prohibited
```

### 2. Escape Analysis

Escape analysis not only determines destruction location but also directly guides backend physical allocation strategy.

#### Escape Path Determination
If an object has any of these characteristics, it's marked as "Escaped" and its lifecycle extends beyond current scope:
1. **Persistent Storage**: Stored in global variables, singletons, or long-lived containers.
2. **Async Capture**: Captured by `spawn` or `async` closures, entering parallel background execution flow.
3. **Ownership Return**: Returned to caller as function return value via `own`.
4. **Type Erasure**: Converted to `any` dynamic type or passed to external language via FFI interface.

#### Physical Optimization Feedback
- **Stack Promotion**: If a `class` object is determined to not escape, the compiler promotes it from heap allocation to stack allocation. This not only avoids GC pressure but also brings huge performance improvement through CPU cache locality.
- **Scalar Replacement (SROA)**: If `structure` doesn't escape and is only locally accessed, the compiler decomposes it into independent scalar variables and maps to registers, completely eliminating memory access.

### 3. Non-local Jump: Effect System and Scope Management

Valkyrie's algebraic effects support flexible execution flow suspension and resumption, placing extremely high demands on lifecycle management.

#### Suspension
When code executes a `perform` operation, current execution flow and variable environment are suspended. At this point, variables in scope **do not trigger** `finalize` because they may be resumed by a Handler in the future.

#### Truncation Cleanup
If a Handler decides to terminate current execution flow (no `resume`), the compiler identifies all relevant suspended scopes and forcibly executes all live variable cleanup logic in stack unwind order, ensuring no resource leaks.

#### Scope Snapshots
For effects requiring multiple resumptions (like non-deterministic computation), the compiler transparently generates snapshots of scope state and restores AIFD model intermediate state on each `resume`.

---

## Next Steps

Now you understand how the compiler automatically manages lifecycles through scopes. Next, we'll explore **[Reference Types (Class)](class.md)**, the most widely used type in Valkyrie, and how it provides zero-burden development experience under the AIFD model.
