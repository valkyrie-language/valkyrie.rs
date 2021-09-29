
## Asynchronous Primitives and Type System

### Future: Low-Level Asynchronous Primitive

Valkyrie's asynchronous system is based on `Future` as the low-level primitive. All asynchronous operations ultimately produce `Future` instances:

- `Promise` - Concrete implementation of Future, used for asynchronous task execution, value passing, and composition
- **[Channel](./channel.md)** - Channel for communication between asynchronous tasks, its receiving end is a Stream implementation
- `async { ... }` block - Syntactic sugar for creating Promise instances

```valkyrie
# All of these are Promise instances (implementing the Future interface)
let promise1: Promise⟨string⟩ = async { "hello" }
let promise2: Promise⟨i32⟩ = Promise.resolve(42)
let composed: Promise⟨string⟩ = async { promise1.await + promise2.await.to_string() }
```

## Async Block: async { }

Inside or outside asynchronous functions, you can use `async { ... }` to create an executable asynchronous Promise object. Within the block, you can use `await` to wait for other asynchronous results.

```valkyrie
# Create an asynchronous Promise (won't immediately block current thread)
let promise = async {
    let user = fetch_user(42).await?
    let posts = fetch_posts(user.id).await?
    (user, posts)
}

# Promise can be composed
let composed = async {
    let (u, p) = promise.await?
    render(u, p)
}
```

Characteristics:
- `async { ... }` is an expression that returns a Promise handle, can be stored in variables, passed as parameters, or further composed.
- Promise doesn't automatically block the current thread, how to "run" it is controlled by `.await`, `.block`, and `.awake` described in the next section.

## Execution Control: .await / .block / .awake

To uniformly control the execution and result retrieval of asynchronous Promises, Promise handles provide the following control operations:

- `promise.await`: In asynchronous context, suspends the current coroutine until the Promise completes and returns the result.
- `promise.block`: In synchronous context, blocks the current thread until the Promise completes, returns the result (suitable for CLI, test entry points, etc.).
- `promise.awake`: Schedules the Promise to start asynchronously on the executor, but doesn't wait for the result, returns a lightweight handle or unit.

### Usage Examples

```valkyrie
# In synchronous entry point (blocking wait)
micro main() {
    let promise = async {
        compute_heavy()  # Assume it's a compute-intensive operation
    }
    let result = promise.block?
    print("Result: ${result}")
}
```

```valkyrie
# In asynchronous context (cooperative wait)
micro handle_request(id: i64) -> string {
    let promise = async {
        let data = fetch_by_id(id).await?
        transform(data)
    }
    let out = promise.await?
    out
}
```

```valkyrie
# Schedule but don't care about result (fire-and-forget)
async {
    audit("user_login")
}.awake

let bg_promise = async { refresh_cache() }
bg_promise.awake   # Trigger background refresh and ignore result
```

### Asynchronous Method Call Rules

#### Execution Control Semantics

For method calls that return Future (Promise and other Future instances):

1. **Auto-Execution Rule**:
   - `obj.call_fut()` itself is equivalent to `obj.call_fut.await()`, will automatically execute and wait for result
   - Parentheses can be omitted: `obj.call_fut` is equivalent to `obj.call_fut()`

2. **Explicit Control Semantics**:
   - `obj.call_fut.await` - Explicit wait (equivalent to auto-execution)
   - `obj.call_fut.awake` - fire-and-forget semantics, don't wait for result
   - `obj.call_fut.block` - Blocking wait (use in synchronous context)

3. **Function Binding**:
   - `let f = obj.call_fut` - Won't auto-execute, but binds the future-returning function to f
   - Static methods follow the same rules

4. **Error Handling**:
   - `?` operator is for Result type error propagation, unrelated to await
   - `promise.await` is for waiting for Promise to complete
   - `promise.block` is for blocking wait for Promise to complete
   - If error propagation is needed, use after the entire expression: `promise.await?`

### Promise Advanced Usage

#### 1. Wrapping Callback Functions

Promise can be used to wrap traditional callback-style APIs, converting them to async/await mode:

```valkyrie
# Wrap callback-style API
micro wrap_callback_api(url: string) -> Promise⟨string⟩ {
    Promise(micro(resolve, reject) {
        # Call traditional callback-style API
        http_request_with_callback(url, micro(result) {
            if result.is_success() {
                resolve(result.data)
            } else {
                reject(result.error)
            }
        })
    })
}

# Use wrapped Promise
micro fetch_data() {
    try {
        let data = wrap_callback_api("https://api.example.com").await?
        print("Fetched data: ${ data }")
    }
    .catch {
        case _:
            print("Request failed: ${ error }")
    }
}
```

#### 2. Promise Cancellation

Promise supports cancellation operations, a feature not available in the basic Future interface:

```valkyrie
# Create cancellable Promise
let (promise, token) = Promise.cancellable {
    let mut count = 0
    loop {
        if $3() {
            $2("Operation cancelled")
            break
        }
        
        count += 1
        sleep(1000ms)  # Auto await
        
        if count >= 10 {
            $1("Operation completed")
            break
        }
    }
}

# Cancel operation elsewhere
sleep(5000ms) {
    token.cancel()
    print("Cancellation requested")
}

# Wait for result or cancellation
try {
    let result = promise.await?
    print("Result: ${ result }")
}
.catch {
    case _:
        print("Operation cancelled or failed: ${ error }")
}
```

**Note**: Future as a low-level primitive doesn't provide cancel functionality, only concrete implementations like Promise support cancellation.

### Unification of Future System

Since Promise is a concrete implementation of Future, all asynchronous operations provide unified execution control interfaces through Promise:

```valkyrie
# All asynchronous operations return Promise
let promise1 = async { compute() }
let promise2 = Promise.resolve(42)

# Unified execution control
promise1.await    # Wait for Promise to complete
promise2.await    # Wait for Promise to complete
promise1.awake    # fire-and-forget Promise
promise2.awake    # fire-and-forget Promise
```

Promise as the sole implementation of Future provides complete asynchronous functionality, including advanced features like cancellation.

### Relationship with Existing await Syntax

- Inside async functions, Promise method calls usually auto-await, no need to manually write .await
- In synchronous functions, if you need to wait for Promise result, use `.block`; if not waiting, use `.awake`
- `awake` semantics are "fire then ignore", suitable for non-critical paths, retryable, or discardable tasks
- All Promise instances follow the same execution semantics

## Asynchronous Stream: Stream

### Stream Concept

When coroutines and generators combine with asynchronous operations, a special `Stream` type is needed to handle asynchronous iteration. Stream is an asynchronous version of an iterator, capable of handling asynchronously produced value sequences.

```valkyrie
# Stream trait definition
trait Stream⟨T⟩ {
    micro next(mut self) -> T?
    micro collect(self) -> [T]
    micro for_each⟨F⟩(self, f: F) where F: micro(T) -> unit
}
```

### Coroutine to Stream

Coroutines can be converted to Streams, providing asynchronous iteration capabilities:

```valkyrie
# Coroutine to Stream
micro fetch_pages(base_url: string) -> Stream⟨string⟩ {
    let mut page = 1
    loop {
        let url = "${ base_url }?page=${ page }"
        let response = http_get(url).await?
        
        if response.is_empty() {
            break
        }
        
        yield response  # Asynchronously produce value
        page += 1
    }
}

# Use Stream
micro process_all_pages() {
    let page_stream = fetch_pages("https://api.example.com/data")
    
    # Asynchronous iteration
    for page in page_stream {
        try {
            process_page(page).await?
        }
        .catch {
            case NetworkError(e):
                print("Network error, skipping: ${ e }")
                continue
            case _:
                break  # Stop processing on other errors
        }
    }
}
```

### Future Iterator vs Iterator Future

#### Future Iterator (Recommended Pattern)

Each iteration returns a Future, suitable for handling independent asynchronous operations:

```valkyrie
# Future Iterator: Iterator⟨Promise⟨T⟩⟩
class FutureIterator⟨T⟩ {
    micro next(mut self) -> Promise⟨T⟩?
}

# Usage example
micro process_urls(urls: [string]) -> FutureIterator⟨string⟩ {
    urls.into_iter().map {
        http_get($).await?
    }
}

# Concurrent processing
micro handle_concurrent() {
    let futures = process_urls(["url1", "url2", "url3"])
    let results = Promise.all(futures.collect()).await?
    
    for result in results {
        print("Result: ${ result }")
    }
}
```

#### Iterator Future (Special Scenario)

The entire iteration process is asynchronous, suitable for ordered dependency scenarios:

```valkyrie
# Iterator Future: Promise⟨Iterator⟨T⟩⟩
class IteratorFuture⟨T⟩ {
    micro resolve(self) -> Iterator⟨T⟩
}

# Usage example: Need authentication before getting iterator
micro authenticated_data() -> IteratorFuture⟨UserData⟩ {
    let token = authenticate().await?
    let data_iter = fetch_user_data(token).await?
    IteratorFuture(data_iter)
}
```

### Stream Error Handling Strategies

#### 1. Error Propagation (Fail Fast)

```valkyrie
# Stop immediately on error
micro strict_processing() {
    let stream = fetch_pages("https://api.example.com")
    
    for page in stream {
        let processed = process_page(page).await?  # Error will propagate immediately
        save_result(processed).await?
    }
}
```

#### 2. Error Skipping (Continue on Error)

```valkyrie
# Skip error items, continue processing
micro resilient_processing() {
    let stream = fetch_pages("https://api.example.com")
    
    for page_result in stream {
        try {
            let page = page_result?  # Unwrap Result
            let processed = process_page(page).await?
            save_result(processed).await?
        }
        .catch {
            case ProcessingError(e):
                log_error("Processing failed, skipping: ${ e }")
                continue
            case _:
                break  # Stop on serious errors
        }
    }
}
```

#### 3. Error Collection (Collect Errors)

```valkyrie
# Collect all errors and successful results
micro collect_all_results() {
    let stream = fetch_pages("https://api.example.com")
    let mut results = []
    let mut errors = []
    
    for page_result in stream {
        match page_result {
            case Fine { value: page }:
                try {
                    let processed = process_page(page).await?
                    results.push(processed)
                }
                .catch {
                    case e:
                        errors.push(e)
                }
            case Fail { error: e }:
                errors.push(e)
        }
    }
    
    (results, errors)
}
```

### Stream Composition Operations

```valkyrie
# Stream functional operations
micro stream_operations() {
    let stream = fetch_pages("https://api.example.com")
    
    let processed_stream = stream
        .filter { !$is_empty() }  # Filter empty pages
        .map { parse_json($).await? }  # Parse JSON
        .take(10)  # Only take first 10
        .buffer(3)  # Buffer 3 concurrent requests
    
    let results = processed_stream.collect().await?
    print("Processing complete: ${ results.length } results")
}
```

### Backpressure Control

```valkyrie
# Control Stream production speed
class BackpressureStream⟨T⟩ {
    private buffer_size: usize
    private current_buffer: [T]
    
    micro next_batch(mut self, batch_size: usize) -> [T] {
        # Implement backpressure control logic
        while self.current_buffer.length < batch_size {
            if let item? = self.source.next().await {
                self.current_buffer.push(item)
            } else {
                break
            }
        }
        
        self.current_buffer.drain(..batch_size.min(self.current_buffer.length))
    }
}
```

Through Stream abstraction, coroutines and generators can elegantly handle asynchronous iteration scenarios, providing flexible error handling strategies and efficient resource management.
