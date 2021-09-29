# AIFD Lifecycle Model

In Valkyrie, an object's complete lifecycle is rigorously divided into four independent yet connected phases. This fine-grained decomposition not only ensures memory safety but also gives developers the ability to flexibly combine memory management strategies in different scenarios.

## Model Definition

1. **A (Allocate)**: **Memory Allocation**. Request raw memory space from a specified allocator that meets the object's alignment and size requirements.
2. **I (Initiate)**: **State Initialization**. "In-place construction" on allocated raw memory, establishing the object's initial logical state.
3. **F (Finalize)**: **Logic Finalization**. Responsible for cleaning up non-memory resources held by the object (like closing file descriptors, disconnecting network connections, releasing mutex locks, or decrementing external reference counts).
4. **D (Delocate)**: **Memory Deallocation**. Return no-longer-used physical memory space to the original allocator, making it available for subsequent operations.

This separation of responsibilities design (especially the separation of F and D) is key to Valkyrie's ability to support both GC and deterministic destruction.

---

## Core Interfaces: Initiate and Finalize

Developers can intervene in object lifecycles by implementing specific Traits.

### 1. State Initialization: `Initiate`

`Initiate` defines how to transform raw memory into valid object state.

```valkyrie
trait Initiate⟨Args⟩ {
    # Safety note: Caller must ensure ptr points to memory successfully allocated in A phase.
    unsafe micro initiate(ptr: ◆Self, args: Args)
}
```

### 2. Logic Cleanup: `Finalize`

`Finalize` focuses on resource cleanup, **strictly prohibiting** physical memory deallocation.

```valkyrie
trait Finalize {
    # Allows execution of final resource release work before object is physically destroyed.
    micro finalize(mut self)
}
```

---

## Declarative Syntax and Automation

To improve development experience, Valkyrie allows directly declaring `initiate` and `finalize` methods in type definitions, and the compiler automatically decomposes them into standard Trait implementations.

### Automatically Generated Implementation

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,

    # Maps to Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
    }

    # Maps to Finalize
    finalize(mut self) {
        intrinsic::close_file(self.handle)
    }
}
```

### Pseudo-overloading Mechanism

Although Valkyrie core syntax doesn't support traditional function overloading, through `Initiate⟨Args⟩` generic design, classes can have multiple "constructors".

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,
    is_temp: bool,

    # Maps to Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = false
    }

    # Maps to Initiate⟨Path, bool⟩
    initiate(mut self, path: Path, is_temp: bool) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = is_temp
    }

    # Maps to Initiate⟨◆u8⟩
    initiate(mut self, handle: ◆u8) {
        self.path = Path::empty()
        self.handle = handle
        self.is_temp = false
    }

    finalize(mut self) {
        intrinsic::close_file(self.handle)
        if self.is_temp {
            intrinsic::delete_file(self.path)
        }
    }
}
```

**Black Magic Principle**: When executing `FileBuffer(...)`, the compiler isn't looking for a same-named function, but rather a Trait instantiation satisfying `Self: Initiate⟨T⟩` constraint. This achieves both flexibility and high performance of static dispatch.

---

## Type Determination and Protocol Consistency

Since lifecycles are abstracted as Traits, all metaprogramming and type determination remain unified:

- **Trait Determination**: `obj is Finalize` can be used to dynamically check if an object needs cleanup logic.
- **Default Behavior**: If `initiate` is not explicitly defined, the compiler generates a default zero-initialization constructor; if `finalize` is omitted, the object is considered "Trivial" and needs no additional operations during destruction.
- **Consistency Protocol**: Keywords like `is` and `as` always determine through Trait protocols, ensuring generic constraints (e.g., `T: Finalize`) perfectly compatible with both manually implemented structures and auto-generated classes.

---

## Next Steps

You've mastered the basics of the AIFD lifecycle model. Next, we'll explore how the compiler uses **[Scope and Auto-insertion](scope.md)** techniques to automatically and precisely orchestrate these lifecycle phases during code execution.
