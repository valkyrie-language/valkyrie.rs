# Channel

`Channel` is a core primitive in Valkyrie for communication between concurrent tasks. It is typically used with `go { }` blocks to implement the CSP (Communicating Sequential Processes) programming model.

## From go { } to Concurrent Collaboration

When we use `go { }` to start a background task, that task detaches from the current execution flow. To safely pass data between different tasks, we introduce `Channel`.

```valkyrie
let (tx, rx) = Channel::new⟨i32⟩()

# Start producer task
go {
    for i in 1..5 {
        tx.send(i).await
    }
    tx.close()
}

# Consume data in main flow
# rx itself is an asynchronous stream (Stream)
for item in rx {
    print("Received: ${item}")
}
```

## Core Features

### 1. Producer-Consumer Model
`Channel::new()` returns a pair of handles:
- **Sender (tx)**: Used to send data.
- **Receiver (rx)**: Used to receive data. `Receiver` implements the `Stream` interface, so it can be used in `for` loops like an iterator.

### 2. Asynchronous Suspension
- **`tx.send(val).await`**: If the channel buffer is full, the send operation triggers asynchronous suspension until space is available.
- **`rx.receive().await`**: If the channel is empty, the receive operation suspends until new data arrives.

### 3. Many-to-Many Communication
Valkyrie's `Channel` supports:
- **MPMC (Multi-Producer, Multi-Consumer)**: Multiple `go` tasks can share the same `Sender` or `Receiver`.

## Channel Topology Models

Based on the number of producers and consumers, Valkyrie provides various channel models to optimize performance:

### 1. SPSC (Single-Producer, Single-Consumer)
The simplest model, one sender corresponding to one receiver. Suitable for simple pipeline tasks.
- **Characteristics**: Extremely high performance, lock-free or low-lock implementation.

### 2. MPSC (Multi-Producer, Single-Consumer)
The most common model, multiple background tasks aggregate results to a central processor.
- **Example**: Log collection system, multiple `go` tasks send messages to the same `Logger`.
```valkyrie
let (tx, rx) = Channel::mpsc()
go { tx.send("Task A done") }
go { tx.send("Task B done") }
```

### 3. MPMC (Multi-Producer, Multi-Consumer)
The most general model, multiple tasks sending, multiple tasks competing to process.
- **Scenario**: Worker Pool.
- **Characteristics**: Automatic load balancing, whoever is idle processes.

## Channel Types

### 1. Unbuffered Channel (Rendezvous)
Channels created by default are usually unbuffered. Sender and receiver must "synchronously" meet for data to be transferred.
```valkyrie
let (tx, rx) = Channel::new()
```

### 2. Buffered Channel
You can specify a buffer size, senders won't suspend when the buffer isn't full.
```valkyrie
let (tx, rx) = Channel::buffered(10)
```

## Relationship with Stream

The receiving end of a `Channel` is a dynamic implementation of `Stream`. This means you can use all `Stream` combinators on `rx`:

```valkyrie
let doubled_stream = rx.map { $ * 2 }
                       .filter { $ > 10 }

doubled_stream.for_each { print($) }.await
```

## Design Choice: Channel vs Async/Await

When writing concurrent programs, you might struggle with whether to use `async/await` directly or introduce `Channel`. Here are suggested selection criteria:

### When to Use Async/Await (Future)?
- **Request-Response Model**: When you call a function and expect **one** clear result at some point in the future.
- **Simple Dependency Chains**: Task A must complete before Task B, and A's output is B's input.
- **Concurrent Aggregation**: Use tools like `Future::join_all` to simultaneously wait for multiple task results and aggregate them.
- **Semantics**: It's more like a "time-consuming ordinary function call".

### When to Use Channel?
- **Data Flow and Pipelines**: When data is **continuously produced** and needs to flow through multiple processing steps (like parse -> filter -> store).
- **Producer-Consumer Decoupling**: When data production speed doesn't match processing speed, need buffers to cushion pressure (Backpressure).
- **Many-to-Many Collaboration**: Multiple tasks jointly processing a task pool, or multiple tasks reporting status to a central task.
- **Semantics**: It's more like a "communication line between different components".

---
**Related Sections**:
- [Asynchronous Effects](../effect-system/asynchronous.md) - Understand the underlying principles of `go { }`
- [Stream](./stream.md) - How to handle continuous data sequences
