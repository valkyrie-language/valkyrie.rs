# Asynchronous Effects (Async Effect)

In Valkyrie, asynchronous programming is not just syntactic sugar; it is a concrete application of **Algebraic Effects**. Drawing inspiration from C# `async2` (Runtime-handled Tasks), Valkyrie offloads asynchronous logic from the compiler level to the runtime level.

## Core Concept: Await is Also an Effect

In traditional asynchronous models (like Rust or C# 5.0), `async` functions are rewritten by the compiler into a complex **state machine**.

In Valkyrie:
- **`.await` is an Effect**: When you call `.await`, it essentially triggers (performs) an effect named `await`, carrying a `Future` or `Task` object.
- **The Scheduler is the Handler**: The runtime environment (such as Nyar VM) provides a top-level effect handler. It catches the `await` effect, suspends the current continuation, and hands it to the asynchronous scheduler (Executor) for management.

## Why Do This?

### 1. Eliminate Function Coloring
Since asynchrony is implemented through the effect system, async functions and sync functions are highly unified in their underlying structure. The compiler doesn't need to generate completely different code paths for `async`, making async code's performance and invocation style closer to sync code.

### 2. Runtime-managed
Similar to C#'s `async2` experiment, Valkyrie hands off task suspension and resumption to the runtime for direct handling.
- **Zero-cost Rewriting**: Bytecode remains concise, without massive state machine jumps.
- **Dynamic Optimization**: The runtime can dynamically decide whether to immediately resume the continuation or put it in a wait queue based on current CPU load and I/O status.

## Asynchronous Primitives: Decoupling Compiler from Runtime

Unlike many languages, Valkyrie's core compiler (HIR) does not include special AST structures for `Future`, `Promise`, or `async/await`.

- **Library-defined, not Built-in**: `Future` and `Promise` are ordinary Traits and Classes defined in the standard library.
- **Transparent Asynchrony**: For the compiler, `.await` is just an effect trigger operation, `.block` is just a regular property access.
- **Runtime Scheduling**: This design draws from the core concept of C# `async2`. In `async2`, the runtime is responsible for managing task suspension and resumption (through lightweight continuations), rather than having the compiler generate heavy state machine code for each async function.

### Advantages

1. **Zero-cost Abstraction**: When async code executes synchronously, there's no state machine switching overhead.
2. **Minimal Bytecode**: Nyar VM only needs to handle generic instructions like `Perform`, `CaptureCont`, and `ResumeWith` to support complex async logic.
3. **Stronger Interoperability**: Since async is just a type of effect, you can easily use other algebraic effects (like dependency injection, exception handling) in async code.

## Runtime Mechanism

### Effect Flow Process

1. **Perform**: When executing `future.await`, the VM executes `perform await(future)`.
2. **Suspend**: The VM immediately saves the current function's execution state (registers, stack frames, IP).
3. **Catch**: The effect bubbles up to the nearest async handler (usually `AsyncRuntime`).
4. **Register**: The scheduler registers the `future` with the I/O multiplexer (like epoll/kqueue) or timer.
5. **Resume**: When the `future` completes, the scheduler finds the corresponding continuation and restores the VM's execution state.

## Example: Low-level Perspective

When you write:
```valkyrie
let data = socket.read().await
```

At the low level, it's equivalent to triggering an `await` effect:
```valkyrie
let data = effect.perform("await", socket.read())
```

If running in a sync environment without an async handler, this effect will keep bubbling up until caught by a handler corresponding to `.block`, or cause the program to crash with "unhandled effect". This ensures predictability and explicitness of async behavior.

## Async Syntax

### Async Block: `async { }`

In Valkyrie, you can use `async { ... }` to create an async task. Note that this is not a special keyword syntax, but **standard syntax of function call with trailing closure**:
- `async` is an ordinary function.
- `{ ... }` is the trailing closure passed to that function.
- The function returns a `Promise` instance after execution.

```valkyrie
let p = async {
    let data = fetch_data().await
    process(data)
}
```

### Automatic Execution and Explicit Control

To simplify code, Valkyrie applies the following rules to function calls returning `Future`:

1. **Automatic Await**: In async context, `obj.call_fut()` is automatically treated as `obj.call_fut().await`.
2. **Suffix Control**: You can explicitly use suffixes to change behavior:
   - `.await`: Explicitly suspend and wait for result.
   - `.awake`: Immediately start task and continue execution (Fire and Forget).
   - `.block`: Block on the current thread waiting for result.

### Shortcut Function: `go`

Similarly, `go { }` is also a shortcut function that receives a closure, running the task immediately in `.awake` mode:

```valkyrie
# Use go function to start background task
go {
    logger.info("Task started")
    do_some_work().await
    logger.info("Task finished")
}
```

Its definition is very simple, essentially calling `async` followed by `.awake`:
```valkyrie
micro go(body: () -> T) -> Promise⟨T⟩ {
    async(body).awake
}
```

## Execution Control

To uniformly control async task execution, Valkyrie provides three core run modes. From the effect system perspective, they represent different effect handling strategies:

### 1. Async Await (`.await`)
**Semantics**: Suspend current coroutine until result is ready.
- **Low-level Mechanism**: Triggers `await` effect, captured by top-level async handler and registered with scheduler.
- **Use Case**: Most async programming scenarios.
```valkyrie
let data = fetch_api().await
```

### 2. Sync Block (`.block`)
**Semantics**: Block current physical thread until async task completes.
- **Low-level Mechanism**: This is a special effect handler. After catching the `await` effect, it doesn't return control to the OS thread, but starts a simple polling loop (Spin/Poll) in place until the result is obtained.
- **Use Case**: `main` function entry, unit tests, or boundaries that must interact with sync legacy code.
```valkyrie
micro main() {
    let result = run_async_task().block
}
```

### 3. Async Start (`.awake`)
**Semantics**: Fire and Forget.
- **Low-level Mechanism**: It doesn't trigger the `await` effect, but directly sends a "start" signal to the scheduler. The current function doesn't need to suspend and continues immediately.
- **Use Case**: Logging, telemetry, background cache refresh, and other non-critical path tasks.
```valkyrie
# Use suffix syntax to start background task
refresh_cache().awake
```

## Async Primitives and Type System

### Future: Low-level Contract
`Future` is the carrier of async effects. At the low level, any method implementing the `poll` effect can be considered a `Future`.

### Promise: Standard Implementation
`Promise` is the concrete implementation of `Future`, with zero-overhead interoperability with JavaScript Promises. In Valkyrie, you can manually control Promise resolution:
```valkyrie
let (p, resolver) = Promise.pending⟨string⟩()
resolver.resolve("Done")
```

## Relationship with Coroutines

Async effects are a specialization of coroutines:
- **Coroutines**: Manual control of `yield` and `resume`.
- **Async Effects**: Automatic control of `yield` (await) and `resume` (ready) by runtime scheduler.

---
**Related Sections**:
- [Coroutines](./coroutine.md) - Foundation of async effects
- [Channel](../reactive-programming/channel.md) - Communication and coordination between tasks
- [Future](../reactive-programming/future.md) - Carrier of async operations
