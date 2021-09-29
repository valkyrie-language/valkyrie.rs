# Promise

`Promise` is the standard implementation of `Future`. It not only represents a future value but also provides the ability to manually control when that value completes.

## Basic Usage

Typically, you can automatically create a `Promise` through an `async` block:

```valkyrie
let p: Promise⟨i32⟩ = async {
    42
}
```

### Explicit Creation and Completion

In some low-level scenarios or when interacting with external code, you may need to manually control a `Promise`:

```valkyrie
# Create a pending Promise and its resolver
let (p, resolver) = Promise.pending⟨string⟩()

# Manually complete it at a later time
resolver.resolve("Success!")

# Or make it fail
# resolver.reject(Error("Failed"))
```

## Execution Control

`Promise` provides three main execution modes, accessed through the `.run` controller (often omittable):

### 1. Asynchronous Await (.await)
Suspends in async functions without blocking the thread.
```valkyrie
let data = fetch_data().await
```

### 2. Synchronous Blocking (.block)
Blocks the current thread in synchronous environments until the result returns.
```valkyrie
let data = fetch_data().block
```

### 3. Asynchronous Start (.awake)
Starts the task but doesn't wait for its result (Fire and Forget).
```valkyrie
fetch_data().awake
```

## Static Methods

- `Promise.resolve(val)`: Creates an already successful Promise.
- `Promise.reject(err)`: Creates an already failed Promise.
- `Promise.all([p1, p2])`: Waits for all Promises to complete, fails entirely if any fails.
- `Promise.any([p1, p2])`: Returns the result as soon as one Promise succeeds.
- `Promise.allSettled([p1, p2])`: Waits for all Promises to complete (regardless of success or failure).

## Relationship with JavaScript Promise

Valkyrie's `Promise` maps directly to native `Promise` objects when compiled to JavaScript, ensuring zero-overhead interoperability.

---
**Related Sections**:
- [Future](./future.md) - Asynchronous low-level primitive
- [Execution Control](./index.md#execution-controlrunawait--runblock--runawake--awake) - Detailed execution mode explanation
