# Future

`Future` is the most fundamental trait in Valkyrie's asynchronous programming model. It represents a value that will become available at some point in the future.

## Core Concept

`Future` describes a computation that has not yet completed. It doesn't represent the computation itself, but rather a reference to the computation's result.

### Future Trait Definition

At the underlying level, `Future` is similar to the following definition:

```valkyrie
trait Future⟨T⟩ {
    # Attempt to poll the Future's state
    # If completed, return Fine(T)
    # If not completed, return Pending
    micro poll(self, cx: Context) -> Poll⟨T⟩
}
```

## Auto-Await Semantics

In Valkyrie, in most cases you don't need to manually call `poll`. The language provides powerful auto-await semantics:

1. **Suffix Await**: `my_future.await` is the standard way to explicitly suspend the current coroutine and wait for the result.
2. **Implicit Await**: In asynchronous contexts (like `async { }`), directly calling a function that returns `Future` automatically applies `.await` semantics.

## Combinators

`Future` provides rich combinators to handle complex asynchronous logic:

- `fut.map(f)`: When the Future completes, pass its result to function `f`.
- `fut.then(f)`: When the Future completes, pass its result to function `f` that returns another Future (chained calls).
- `Future.join(a, b)`: Wait for two Futures to complete simultaneously, returning their tuple results.
- `Future.race(a, b)`: Wait for either of two Futures to complete, returning the fastest completed result.

## Relationship with Coroutines

Valkyrie's `Future` is deeply integrated with algebraic effects. When a `Future` needs to wait, it performs a special effect that is captured by the executor, suspending the current task until data is ready.

---
**Related Sections**:
- [Promise](./promise.md) - Standard implementation of Future
- [Async Block](./index.md#async-blockasync) - Convenient syntax for creating Futures
