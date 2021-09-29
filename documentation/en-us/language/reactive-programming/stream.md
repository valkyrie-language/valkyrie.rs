# Stream

`Stream` is an asynchronous iterator in Valkyrie. It bridges the gap between synchronous iteration and asynchronous data flows, allowing you to process sequences of values that arrive over time.

## Core Concept

A `Stream` is like an asynchronous version of an `Iterator`. Instead of returning values immediately, it may need to wait for values to become available.

```valkyrie
trait Stream⟨T⟩ {
    # Attempt to get the next value
    # Returns None when the stream is exhausted
    micro next(mut self) -> Future⟨Option⟨T⟩⟩
}
```

## Creating Streams

### From Generators

The most common way to create a stream is using a generator:

```valkyrie
micro count_stream(n: i32) -> Stream⟨i32⟩ {
    for i in 1..=n {
        yield i
    }
}
```

### From Channels

The receiving end of a channel is a stream:

```valkyrie
let (tx, rx) = Channel.new⟨string⟩()

# rx is a Stream<string>
for message in rx {
    print(message)
}
```

### From Arrays

```valkyrie
let stream = Stream.from([1, 2, 3, 4, 5])
```

## Consuming Streams

### Async For Loop

```valkyrie
let stream = count_stream(5)

for value in stream {
    print(value)
}
```

### Manual Iteration

```valkyrie
let stream = count_stream(5)

loop {
    match stream.next().await {
        case Some(value):
            print(value)
        case None:
            break
    }
}
```

### Collecting Results

```valkyrie
let stream = count_stream(5)
let values: [i32] = stream.collect().await
```

## Stream Combinators

Streams support functional-style transformations:

```valkyrie
let result = count_stream(10)
    .filter { $ % 2 == 0 }      # Keep even numbers
    .map { $ * 2 }               # Double each
    .take(3)                     # Take first 3
    .collect().await             # [4, 8, 12]
```

### Common Combinators

| Combinator | Description |
|------------|-------------|
| `.map(f)` | Transform each value |
| `.filter(p)` | Keep values matching predicate |
| `.take(n)` | Take first n values |
| `.skip(n)` | Skip first n values |
| `.fold(init, f)` | Reduce to single value |
| `.for_each(f)` | Execute function for each value |
| `.buffer(n)` | Buffer up to n values |
| `.throttle(dur)` | Limit rate of values |
| `.debounce(dur)` | Wait for pause before emitting |

## Error Handling

Streams can produce errors:

```valkyrie
micro risky_stream() -> Stream⟨Result⟨i32, Error⟩⟩ {
    for i in 1..=10 {
        if i == 5 {
            yield Fail(Error("Failed at 5"))
        } else {
            yield Fine(i)
        }
    }
}

# Handle errors in stream
for result in risky_stream() {
    match result {
        case Fine(value):
            print(value)
        case Fail(error):
            print("Error: ${error}")
    }
}
```

## Backpressure

Streams naturally handle backpressure - consumers process at their own pace:

```valkyrie
micro producer(tx: Sender⟨i32⟩) {
    for i in 1..=100 {
        # This will wait if buffer is full
        tx.send(i).await
    }
    tx.close()
}

micro consumer(rx: Receiver⟨i32⟩) {
    # Process at our own pace
    for value in rx {
        process(value).await  # May be slow
        # Producer automatically slows down
    }
}
```

## Stream vs Observable

| Feature | Stream | Observable |
|---------|--------|------------|
| **Pull/Push** | Pull-based | Push-based |
| **Control** | Consumer controls pace | Producer controls pace |
| **Cancellation** | Natural (stop pulling) | Requires subscription management |
| **Use Case** | File I/O, network streams | UI events, real-time updates |

## Integration with Async

Streams work seamlessly with Valkyrie's async system:

```valkyrie
micro process_file(path: string) -> Stream⟨string⟩ {
    let file = File.open(path).await?
    
    loop {
        let line = file.read_line().await?
        match line {
            case Some(text):
                yield text
            case None:
                break
        }
    }
}

# Use the stream
let lines = process_file("data.txt")
for line in lines {
    print(line)
}
```

---
**Related Sections**:
- [Channel](./channel.md) - Multi-producer, multi-consumer communication
- [Future](./future.md) - Single async value
- [Observable](./observable.md) - Push-based event streams
