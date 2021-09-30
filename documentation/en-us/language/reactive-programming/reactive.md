# Reactive Programming Overview

Valkyrie's reactive programming system is built on several core primitives that work together to handle asynchronous data flows and state management.

## Core Primitives

### 1. Future & Promise

`Future` is a low-level trait representing a value that will be available later. `Promise` is its standard implementation.

```valkyrie
# Create a Promise
let promise = async {
    let result = compute_heavy_task().await
    result * 2
}

# Wait for result
let value = promise.await
```

### 2. Signal

`Signal` represents current state. It always has a value and notifies subscribers when that value changes.

```valkyrie
# Create a signal with initial value
let count = Signal.new(0)

# Read current value
print(count.get())  # 0

# Update value
count.set(5)

# Computed signal
let doubled = count.derive { $ * 2 }
```

### 3. Observable

`Observable` represents a sequence of values over time, ideal for events and streams.

```valkyrie
# Create from events
let clicks = Observable.from_event(button, "click")

# Transform and filter
let processed = clicks
    .filter { $.target.matches(".important") }
    .map { $.target.id }
    .debounce(300ms)
```

### 4. Stream

`Stream` is an asynchronous iterator, bridging the gap between synchronous iteration and async data flows.

```valkyrie
# Create a stream
let stream = Stream.from_generator {
    for i in 1..10 {
        yield i
    }
}

# Consume asynchronously
for item in stream {
    print(item)
}
```

### 5. Channel

`Channel` enables communication between concurrent tasks.

```valkyrie
let (tx, rx) = Channel.new⟨string⟩()

# Producer
go {
    tx.send("Hello").await
    tx.close()
}

# Consumer
for message in rx {
    print(message)
}
```

## Choosing the Right Primitive

| Primitive | Use Case | Characteristics |
|-----------|----------|-----------------|
| **Future/Promise** | Single async operation | One value, one-time |
| **Signal** | UI state, configuration | Always has value, synchronous read |
| **Observable** | Events, user actions | Multiple values, lazy evaluation |
| **Stream** | Async iteration | Pull-based, can be paused |
| **Channel** | Task communication | Multi-producer, multi-consumer |

## Execution Control

Valkyrie provides unified execution control for async operations:

```valkyrie
# Async wait (non-blocking)
let result = async_operation().await

# Sync blocking
let result = async_operation().block

# Fire and forget
async_operation().awake
```

## Error Handling

All primitives support structured error handling:

```valkyrie
try {
    let result = risky_operation().await?
    process(result)
}
.catch {
    case NetworkError(e):
        retry()
    case ValidationError(e):
        show_error(e)
}
```

## Integration with UI

Valkyrie's reactive primitives integrate seamlessly with the UI system:

```valkyrie
widget Counter {
    count: Signal⟨i32⟩ = Signal.new(0)
    
    Column {
        Text("Count: {count.get()}")
        Button("Increment") {
            on_click { count.update { $ + 1 } }
        }
    }
}
```

## Chapter Contents

- **[Future](./future.md)**: Low-level async primitive
- **[Promise](./promise.md)**: Standard Future implementation
- **[Signal](./signal.md)**: State management primitive
- **[Observable](./observable.md)**: Event stream primitive
- **[Stream](./stream.md)**: Async iterator
- **[Channel](./channel.md)**: Concurrent task communication
