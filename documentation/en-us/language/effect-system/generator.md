# Generators

Valkyrie's generators are special functions that can produce a series of values through the `yield` keyword. Generators provide a lazy computation approach, calculating the next value only when needed, making them ideal for handling large amounts of data or infinite sequences.

## Basic Generator Syntax

### Simple Generators

```valkyrie
# Basic generator function
micro count_up(max: i32) {
    let i = 0
    while i < max {
        yield i
        i += 1
    }
}

# Using the generator
let counter = count_up(5)
for value in counter {
    print(value)  # Output: 0, 1, 2, 3, 4
}
```

### Infinite Generators

```valkyrie
# Fibonacci sequence generator
micro fibonacci() {
    let a = 0
    let b = 1
    loop {
        yield a
        let temp = a + b
        a = b
        b = temp
    }
}

# Get first 10 Fibonacci numbers
let fib = fibonacci()
for i in 0..<10 {
    print(fib.next())  # 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
}
```

### Generators with Return Values

```valkyrie
# Generators can have a final return value
micro process_items(items: [utf8]) -> i32 {
    let count = 0
    for item in items {
        if item.is_valid() {
            yield item.process()
            count += 1
        }
    }
    count  # Final return: number of processed items
}

# Usage
let processor = process_items(["item1", "item2", "item3"])
for result in processor {
    print("Processed: {result}")
}
let total_count = processor.return_value()  # Get final return value
```

## Generator State Management

### Implementation Principle: Algebraic Effects

You might wonder why `yield` in a `sequence` closure can be "intercepted" even though `sequence` is an ordinary function. The core mechanism behind this is Valkyrie's **Algebraic Effects**.

1. **Yield is an Effect**: In Valkyrie, `yield` is not a hardcoded keyword bound to function definitions. It's essentially an "effect call," similar to a resumable exception.
2. **Handler Stack**: The Valkyrie VM maintains a handler stack. When `yield` is executed, the VM pauses the current execution flow and searches up the call stack for the nearest matching handler.
3. **Penetration Capability**: Since effects are managed through the VM stack, they have the ability to "penetrate" ordinary functions and closures. Even if `sequence` is an ordinary function and the closure is an ordinary closure, `yield` inside will bubble up until caught by the `handle` block set up inside `sequence`.
4. **Resume Execution**: The handler inside `sequence`, after catching the `yield` value, also obtains a "Continuation". This allows `sequence` to return the value to the iterator and precisely resume the closure's execution when `next()` is called again.

This design makes generators no longer limited to whole-function transformation, but can exist as a local, nestable control flow primitive.

### Sequence Environment

Besides defining an entire `micro` function as a generator, Valkyrie also supports using the `sequence` environment to locally define generators within ordinary functions. This allows you to produce a lazy sequence without changing the nature of the entire function.

#### Local Generators

Using a `sequence` block creates an anonymous generator object:

```valkyrie
micro process_data(data: [i32]) {
    # Define local generator in ordinary function
    let gen = sequence {
        for item in data {
            if item > 0 {
                yield item * 2
            }
        }
    }
    
    # Use local generator
    for val in gen {
        print(val)
    }
}
```

#### Explicit Type Declaration

You can also explicitly specify the element type produced by the `sequence` environment (through generic parameters):

```valkyrie
let gen = sequence⟨utf8⟩ {
    yield "Hello"
    yield "World"
}
```

Note that `sequence` is not a keyword, but a higher-order function that uses algebraic effects to intercept `yield`, so it uses standard generic call syntax `⟨T⟩`.

#### Expression Usage

The `sequence` environment is an expression that can be passed directly as an argument or returned:

```valkyrie
micro get_numbers() {
    return sequence {
        yield 1
        yield 2
        yield 3
    }
}
```

### Generator Lifecycle

```valkyrie
# Generator state enum
unite GeneratorState {
    Created,     # Created but not started
    Running,     # Currently executing
    Suspended,   # Suspended (yield)
    Completed,   # Completed
    Fail { error: any } # Error occurred
}

# Check generator state
micro example_generator() {
    print("Start execution")
    yield "First value"
    print("Continue execution")
    yield "Second value"
    print("Execution complete")
}

let gen = example_generator()
print(gen.state())  # Created

let first = gen.next()
print(gen.state())  # Suspended
print(first)        # "First value"

let second = gen.next()
print(gen.state())  # Suspended
print(second)       # "Second value"

gen.next()          # Complete execution
print(gen.state())  # Completed
```

### Generator Control

```valkyrie
# Manual generator execution control
micro controlled_generator() {
    let mut value = 0
    loop {
        let input = yield value
        if input != null {
            value = input  # Receive external input
        } else {
            value += 1     # Default increment
        }
    }
}

let gen = controlled_generator()
print(gen.next())        # 0
print(gen.send(10))      # 10 (send value to generator)
print(gen.next())        # 11
print(gen.send(100))     # 100
```

## Generator Pipelines

### Pipeline Processing

```valkyrie
# Generator pipeline processing
micro pipeline_stage1(input: Iterator⟨i32⟩) {
    for value in input {
        yield value * 2  # Stage 1: multiply by 2
    }
}

micro pipeline_stage2(input: Iterator⟨i32⟩) {
    for value in input {
        if value % 4 == 0 {
            yield value  # Stage 2: filter multiples of 4
        }
    }
}

micro pipeline_stage3(input: Iterator⟨i32⟩) {
    for value in input {
        yield "Result: {value}"  # Stage 3: format
    }
}

# Build pipeline
let numbers = [1, 2, 3, 4, 5, 6, 7, 8]
let stage1 = pipeline_stage1(numbers.iter())
let stage2 = pipeline_stage2(stage1)
let stage3 = pipeline_stage3(stage2)

for result in stage3 {
    print(result)  # "Result: 4", "Result: 8", "Result: 12", "Result: 16"
}
```

### Combining Generators

```valkyrie
# Combine multiple generators
micro combine_generators(gen1: Generator⟨i32⟩, gen2: Generator⟨i32⟩) {
    # Alternately produce values from both generators
    loop {
        let val1 = gen1.next()
        let val2 = gen2.next()
        
        if val1.is_some() {
            yield val1.unwrap()
        }
        if val2.is_some() {
            yield val2.unwrap()
        }
        
        if val1.is_none() && val2.is_none() {
            break
        }
    }
}

let gen1 = count_up(3)  # 0, 1, 2
let gen2 = count_up(2)  # 0, 1
let combined = combine_generators(gen1, gen2)

for value in combined {
    print(value)  # 0, 0, 1, 1, 2
}
```

## Advanced Generator Patterns

### Lazy Computation

```valkyrie
# Lazy prime number computation
micro prime_generator() {
    let mut candidates = 2..
    let mut primes = []
    
    for candidate in candidates {
        let is_prime = primes.all { candidate % $ != 0 }
        if is_prime {
            primes.push(candidate)
            yield candidate
        }
    }
}

# Get first 10 primes
let primes = prime_generator()
for i in 0..<10 {
    print(primes.next())  # 2, 3, 5, 7, 11, 13, 17, 19, 23, 29
}
```

### File Processing Generator

```valkyrie
# Read file line by line
micro read_lines(filename: utf8) {
    let file = open_file(filename)
    try {
        while !file.eof() {
            let line = file.read_line()
            if !line.is_empty() {
                yield line.trim()
            }
        }
    } finally {
        file.close()
    }
}

# Usage
for line in read_lines("data.txt") {
    print("Line: {line}")
}
```

### Data Transformation Generator

```valkyrie
# Data transformation pipeline
micro transform_data(data: Iterator<utf8>) {
    for item in data {
        # Parse JSON
        let parsed = json_parse(item)
        if parsed.is_ok() {
            let obj = parsed.unwrap()
            
            # Validate data
            if obj.has_field("id") && obj.has_field("name") {
                # Transform format
                let transformed = {
                    id: obj.id,
                    name: obj.name.to_uppercase(),
                    timestamp: current_time()
                }
                yield transformed
            }
        }
    }
}
```

## Error Handling

### Generator Exception Handling

```valkyrie
# Exception handling in generators
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

# Using generator with error handling
let gen = error_prone_generator()
try {
    for status in gen {
        print(status)
    }
}
.catch {
    case _:
        print("Generator exception: {error}")
}
```

## Best Practices

### 1. Generator Design Principles

```valkyrie
# Keep generators simple and focused
micro good_generator(data: [utf8]) {
    for item in data {
        if item.is_valid() {
            yield item.process()  # Do only one thing
        }
    }
}

# Avoid complex state management in generators
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
    using file = open_file(filename) {
        while !file.eof() {
            let line = file.read_line()
            yield process_line(line)
        }
    }  # File automatically closes
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

### 4. Testing Generators

```valkyrie
# Generator testing strategy
micro test_generator() {
    let gen = count_up(3)
    
    # Test generated values
    @assert_equal(gen.next(), Some(0))
    @assert_equal(gen.next(), Some(1))
    @assert_equal(gen.next(), Some(2))
    @assert_equal(gen.next(), None)
    
    # Test state
    @assert_equal(gen.state(), GeneratorState::Completed)
}

# Generator integration test
micro test_pipeline() {
    let input = [1, 2, 3, 4]
    let pipeline = pipeline_stage1(input.iter())
    let results = pipeline.collect()
    
    @assert_equal(results, [2, 4, 6, 8])
}
```

### 5. Return Value Restrictions

```valkyrie
# Generator return values cannot be anonymous classes
# Error example:
micro bad_generator() -> class { x: i32 } {  # Compile error
    yield 1
    class { x: 42 }  # Anonymous class as return value causes type inference difficulties
}

# Correct example:
class Result {
    x: i32
}

micro good_generator() -> Result {
    yield 1
    Result { x: 42 }  # Use named type
}

# Or use type alias
type GeneratorResult = class { x: i32 }

micro another_good_generator() -> GeneratorResult {
    yield 1
    GeneratorResult { x: 42 }
}
```
