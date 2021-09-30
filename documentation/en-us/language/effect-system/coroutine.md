# Coroutines

Valkyrie provides powerful coroutine support, implementing cooperative multitasking through the `yield` keyword. Coroutines allow functions to pause and resume during execution, making them ideal for handling asynchronous operations and state machines.

For advanced applications of coroutines in async programming (such as `async/await` and runtime scheduling), please refer to:
- **[Asynchronous Effects](./asynchronous.md)**

## Coroutine State Management

### Coroutine Lifecycle

```valkyrie
# Coroutine state enum
union CoroutineState {
    Created,     # Created but not started
    Running,     # Currently executing
    Suspended,   # Suspended (yield)
    Completed,   # Completed
    Fail { error: Any } # Error occurred
}

# Check coroutine state
micro example_coroutine() {
    print("Start execution")
    yield "First value"
    print("Continue execution")
    yield "Second value"
    print("Execution complete")
}

let coro = example_coroutine()
print(coro.state())  # Created

let first = coro.next()
print(coro.state())  # Suspended
print(first)         # "First value"

let second = coro.next()
print(coro.state())  # Suspended
print(second)        # "Second value"

coro.next()          # Complete execution
print(coro.state())  # Completed
```

### Coroutine Control

```valkyrie
# Manual coroutine execution control
micro controlled_coroutine() {
    let state = "idle"
    loop {
        let command = yield state
        match command {
            case "start":
                state = "running"
            case "pause":
                state = "paused"
            case "stop":
                state = "stopped"
                break
            case _:
                state = "unknown_command"
        }
    }
}

let coro = controlled_coroutine()
print(coro.next())           # "idle"
print(coro.send("start"))    # "running"
print(coro.send("pause"))    # "paused"
print(coro.send("stop"))     # "stopped"
```

## Async Coroutines

### Async Operations

```valkyrie
# Async coroutine
micro fetch_data(url: utf8) -> utf8 {
    print("Starting request: {url}")
    let response = http_get(url).await?
    yield "Request sent"  # Can use yield in async functions
    
    if response.status == 200 {
        yield "Request successful"
        response.body
    } else {
        raise "Request failed: {response.status}"
    }
}

# Using async coroutine
micro main() {
    let fetcher = fetch_data("https://api.example.com/data")
    
    # Handle intermediate states
    for status in fetcher {
        print("Status: {status}")
    }
    
    # Get final result
    try {
        let data = fetcher.await?
        print("Data: {data}")
    }
    .catch {
        case _:
            print("Error: {error}")
    }
}
```

### Concurrent Coroutines

```valkyrie
# Execute multiple coroutines concurrently
micro concurrent_processing(items: [utf8]) {
    let promises = items.map { item ->
        let result = process_item(item)
        yield "Processing complete: {item}"
        result
    }
    
    # Wait for all Promises to complete
    let results = Promise::all(promises).await?
    yield "All tasks complete"
    results
}

# Usage
micro run_concurrent() {
    let processor = concurrent_processing(["item1", "item2", "item3"])
    
    for update in processor {
        print(update)
    }
    
    let final_results = processor.await?
    print("Final results: {final_results}")
}
```

## Advanced Coroutine Patterns

### State Machine Coroutine

```valkyrie
# State machine implementation
union State {
    Idle,
    Processing,
    Waiting,
    Complete
}

micro state_machine() {
    let mut state = State::Idle
    let mut data = null
    
    loop {
        state.match {
            case State::Idle: {
                yield "Waiting for input"
                data = yield_receive()  # Wait for external input
                state = State::Processing
            }
            case State::Processing: {
                yield "Processing..."
                let result = process_data(data)
                if result.is_ok() {
                    state = State::Complete
                } else {
                    state = State::Waiting
                }
            }
            case State::Waiting: {
                yield "Waiting for retry"
                sleep(1000)  # Wait 1 second
                state = State::Processing
            }
            case State::Complete: {
                yield "Processing complete"
                break
            }
        }
    }
}
```

### Coroutine Pool

```valkyrie
# Coroutine pool management
class CoroutinePool {
    coroutines: [Coroutine],
    max_size: i32,
    active_count: i32
    
    micro new(max_size: i32) -> Self {
        CoroutinePool {
            coroutines: [],
            max_size: max_size,
            active_count: 0
        }
    }
    
    micro spawn(task: micro() -> Any) -> bool {
        if self.active_count < self.max_size {
            let coro = Coroutine::new(task)
            self.coroutines.push(coro)
            self.active_count += 1
            true
        } else {
            false  # Pool is full
        }
    }
    
    micro run_all() {
        while self.active_count > 0 {
            for coro in self.coroutines {
                if coro.state() == CoroutineState::Suspended {
                    let result = coro.resume()
                    yield "Coroutine progress: {result}"
                    
                    if coro.state() == CoroutineState::Completed {
                        self.active_count -= 1
                    }
                }
            }
        }
        yield "All coroutines complete"
    }
}
```

## Error Handling

### Coroutine Exception Handling

```valkyrie
# Exception handling in coroutines
micro error_prone_generator() {
    try {
        yield "Start processing"
        
        let risky_operation = perform_risky_task()
        yield "Risky operation complete"
        
        if risky_operation.is_error() {
            raise "Operation failed"
        }
        
        yield "Processing successful"
    }
    .catch {
        case _:
            yield "Error occurred: {error}"
            raise error  # Re-throw exception
    }
}

# Using coroutine with error handling
let gen = error_prone_generator()
try {
    for status in gen {
        print(status)
    }
}
.catch {
    case _:
        print("Coroutine exception: {error}")
}
```

## Best Practices

### 1. Coroutine Design Principles

```valkyrie
# Keep coroutines simple and focused
micro good_generator(data: [utf8]) {
    for item in data {
        if item.is_valid() {
            yield item.process()  # Do only one thing
        }
    }
}

# Avoid complex state management in coroutines
# Bad example:
micro bad_generator() {
    let mut complex_state = ComplexState::new()
    # ... Complex state logic
}
```

### 2. Resource Management

```valkyrie
# Ensure proper resource release
micro file_processor(filename: utf8) {
    let file = open_file(filename)
    try {
        while !file.eof() {
            let line = file.read_line()
            yield process_line(line)
        }
    }
    # Use using to ensure file closure
    # using file = open_file(filename) { ... }
}
```

### 3. Performance Considerations

```valkyrie
# Avoid frequent small yields
# Bad example:
micro inefficient_generator(data: [i32]) {
    for item in data {
        yield item  # Yield every element
    }
}

# Good example:
micro efficient_generator(data: [i32]) {
    let mut batch = []
    for item in data {
        batch.push(item)
        if batch.length >= 100 {
            yield batch  # Batch yield
            batch = []
        }
    }
    if !batch.is_empty() {
        yield batch  # Process remaining items
    }
}
```

### 4. Testing Coroutines

```valkyrie
# Coroutine testing strategy
micro test_generator() {
    let gen = count_up(3)
    
    # Test generated values
    @assert_equal(gen.next(), 0)
    @assert_equal(gen.next(), 1)
    @assert_equal(gen.next(), 2)
    @assert_equal(gen.next(), null)
    
    # Test state
    @assert_equal(gen.state(), CoroutineState::Completed)
}

# Async coroutine test
micro test_async_generator() {
    let gen = async_data_processor()
    
    let first_result = gen.next().await?
    assert!(first_result != null)
    
    let final_result = gen.collect_all().await?
    @assert_equal(final_result.length, 5)
}
```
