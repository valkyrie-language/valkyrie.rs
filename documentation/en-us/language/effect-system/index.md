# Effect System

Valkyrie's effect system provides powerful side effect management and control flow mechanisms, including exception handling, coroutines, generators, reactive programming, aspect-oriented programming, and dependency injection.

## Effect System Components

- **[Exception Handling](./error-handler.md)** - Flexible error handling and exception propagation mechanisms
- **[Coroutines](./coroutine.md)** - Foundation for cooperative multitasking
- **[Asynchronous Effects](./asynchronous.md)** - Algebraic effect-based asynchronous programming model
- **[Generators](./generator.md)** - Lazy computation and value sequence generation
- **[Reactive Programming](./reactive.md)** - Data flow and change propagation programming paradigm
- **[Aspect-Oriented Programming](./aop.md)** - Separation and management of cross-cutting concerns
- **[Dependency Injection](./ioc.md)** - Inversion of control and dependency management

---

# Exception Handling System

In Valkyrie, any object can be thrown and caught as an exception. This provides a flexible error handling mechanism that allows programs to handle various exceptional situations in a structured manner.

## Basic Exception Handling

### Throwing Exceptions

```valkyrie
# Any object can be thrown as an exception
raise "Something went wrong"
raise 404
raise { code: 500, message: "Internal Server Error" }

# Throw custom objects
class NetworkError {
    message: string
    code: i32
}

raise NetworkError {
    message: "Connection timeout",
    code: -1
}
```

### Catching Exceptions

```valkyrie
# Basic exception catching
try {
    risky_operation()
}
.catch {
    case Fail(error): print("Caught error: ${error}")
}

# Type-specific exception catching
try {
    network_request()
}
.catch {
    case Fail(error: NetworkError):
            print("Network error: ${error.message}")
            retry_connection()
    case Fail(error: string): print("String error: ${error}")
    else: print("Unknown error")
}
```

## Exception Types and Patterns

### String Exceptions

```valkyrie
# Simple string exceptions
micro validate_age(age: i32) {
    if age < 0 {
        raise "Age cannot be negative"
    }
    if age > 150 {
        raise "Age seems unrealistic"
    }
}

try {
    validate_age(-5)
}
.catch {
    case message: string: print("Validation error: ${message}")
}
```

### Numeric Exceptions

```valkyrie
# Using numbers as error codes
micro http_request(url: string) {
    if !is_valid_url(url) {
        raise 400  # Bad Request
    }
    if !is_authorized() {
        raise 401  # Unauthorized
    }
    if !resource_exists(url) {
        raise 404  # Not Found
    }
    # Normal processing
}

try {
    http_request("invalid-url")
}
.catch {
    case code: i32 {
        match code {
        400 => print("Bad request")
        401 => print("Unauthorized")
        404 => print("Not found")
        _ => print("HTTP error: ${code}")
    }
}
```

### Object Exceptions

```valkyrie
# Structured exception objects
class DatabaseError {
    query: string
    error_code: i32
    message: string
    timestamp: DateTime
}

class ValidationError {
    field: string
    value: string
    constraint: string
}

micro save_user(user: User) {
    # Validate user data
    if user.email.is_empty() {
        raise ValidationError {
            field: "email",
            value: user.email,
            constraint: "Email cannot be empty"
        }
    }
    
    # Database operation
    try {
        database.insert(user)
    }
    .catch {
        case db_error: raise DatabaseError {
            query: "INSERT INTO users...",
            error_code: db_error.code,
            message: db_error.message,
            timestamp: DateTime.now()
        }
    }
}

# Handle different types of exceptions
try {
    save_user(invalid_user)
}
.catch {
    case ValidationError: print("Validation failed for field '${error.field}': ${error.constraint}")
        case DatabaseError:
        print("Database error at ${error.timestamp}: ${error.message}")
        log_error(error)
}
```

## Exception Propagation

### Automatic Propagation

```valkyrie
# Exceptions automatically propagate upward
micro level3() {
    raise "Error from level 3"
}

micro level2() {
    level3()  # Exception propagates here
}

micro level1() {
    level2()  # Exception continues to propagate
}

try {
    level1()
}
.catch {
    case _: print("Caught at top level: ${error}")
}
```

### Exception Transformation

```valkyrie
# Catch and transform exceptions
micro parse_config(content: string) -> Config {
    try {
        json.parse(content)
    }
    .catch {
        case parse_error: raise ConfigError {
            message: "Failed to parse configuration",
            cause: parse_error,
            content_preview: content.substring(0, 100)
        }
    }
}

# Exception chain
class ConfigError {
    message: string
    cause: Any
    content_preview: string
}
```

## Resource Management

### Automatic Cleanup

```valkyrie
# Use 'using' to ensure resource cleanup
micro process_file(filename: string) {
    using file = File.open(filename) {
        let content = file.read_all()
        process_content(content)
    }  # file automatically closes
}

# Use defer for delayed execution
micro database_transaction() {
    let transaction = db.begin_transaction()
    defer transaction.rollback()  # Default rollback
    
    try {
        # Execute database operations
        db.insert(data1)
        db.update(data2)
        db.delete(data3)
        
        transaction.commit()
        defer.cancel()  # Cancel rollback
    }
    .catch {
        case error {
            # Automatic rollback on exception
            raise error
        }
    }
}
```

### Resource Wrapping

```valkyrie
# Wrapper for automatic resource management
class ManagedResource⟨T⟩ {
    resource: T
    cleanup: () -> ()
    
    micro new(resource: T, cleanup: () -> ()) -> Self {
        Self { resource, cleanup }
    }
    
    micro use⟨R⟩(block: (T) -> R) -> R {
        let result = block(self.resource)
        self.cleanup()
        result
    }
}

# Usage example
let managed_file = ManagedResource.new(
    File.open("data.txt"),
    { $close() }
)

managed_file.use {
    let content = $read_all()
    process_content(content)
}  # File automatically closes
```

## Exception Handling Patterns

### Retry Pattern

```valkyrie
micro retry⟨T⟩(max_attempts: i32, operation: () -> T) -> T {
    let mut attempts = 0
    loop {
        try {
            return operation()
        }
        .catch {
            case error {
                attempts += 1
            if attempts >= max_attempts {
                raise RetryExhausted {
                    attempts: attempts,
                    last_error: error
                }
            }
            sleep(Duration.seconds(attempts))  # Exponential backoff
        }
    }
}

# Using retry
try {
    let result = retry(3, { =>
        unreliable_network_call()
    })
    print("Success: ${result}")
}
.catch {
    case RetryExhausted: print("Failed after ${error.attempts} attempts: ${error.last_error}")
}
```

### Circuit Breaker Pattern

```valkyrie
class CircuitBreaker {
    failure_count: i32
    failure_threshold: i32
    state: CircuitState
    last_failure_time: DateTime
    
    micro call⟨T⟩(operation: () -> T) -> T {
        match self.state {
            CircuitState.Closed => {
                try {
                    let result = operation()
                    self.reset()
                    result
                }
                .catch {
                    case error {
                        self.record_failure()
                        raise error
                    }
                }
            }
            CircuitState.Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState.HalfOpen
                    self.call(operation)
                } else {
                    raise CircuitBreakerOpen {
                        message: "Circuit breaker is open"
                    }
                }
            }
            CircuitState.HalfOpen => {
                try {
                    let result = operation()
                    self.reset()
                    result
                }
                .catch {
                    case error {
                        self.state = CircuitState.Open
                        raise error
                    }
                }
            }
        }
    }
}
```

### Exception Aggregation

```valkyrie
# Collect multiple exceptions
class AggregateException {
    exceptions: [Any]
    
    micro add(exception: Any) {
        self.exceptions.push(exception)
    }
    
    micro has_errors() -> bool {
        self.exceptions.length > 0
    }
}

micro process_batch(items: [Item]) {
    let errors = AggregateException { exceptions: [] }
    
    for item in items {
        try {
            process_item(item)
        }
        .catch {
            case _: errors.add(error)
        }
    }
    
    if errors.has_errors() {
        raise errors
    }
}
```

## Best Practices

### 1. Exception Type Design

```valkyrie
# Use meaningful exception types
class UserNotFoundError {
    user_id: string
    search_criteria: Map⟨string, string⟩
}

class PermissionDeniedError {
    user: string
    resource: string
    required_permission: string
}

# Instead of generic strings
# raise "User not found"  # Not recommended
```

### 2. Exception Information

```valkyrie
# Provide sufficient context information
class FileProcessingError {
    filename: string
    line_number: i32
    column: i32
    error_type: string
    suggestion: string
}

micro parse_csv(filename: string) {
    try {
        # Parsing logic
    }
    .catch {
        case _: raise FileProcessingError {
            filename: filename,
            line_number: current_line,
            column: current_column,
            error_type: "Invalid CSV format",
            suggestion: "Check for missing quotes or commas"
        }
    }
}
```

### 3. Exception Handling Strategy

```valkyrie
# Handle exceptions at the appropriate level
micro application_main() {
    try {
        run_application()
    }
    .catch {
        case ConfigurationError:
            print("Configuration error: ${error.message}")
            print("Please check your configuration file")
            exit(1)
        case NetworkError:
            print("Network error: ${error.message}")
            print("Please check your internet connection")
            exit(2)
        else:
            print("Unexpected error: ${error}")
            log_error(error)
            exit(99)
    }
}

# Don't over-catch exceptions
micro bad_example() {
    try {
        some_operation()
    }
    .catch {
        case error {
            # Doing nothing hides the error
        }
    }
}
```

### 4. Testing Exception Handling

```valkyrie
#[test]
micro test_validation_error() {
    let invalid_user = User { email: "" }
    
    try {
        save_user(invalid_user)
        assert(false, "Expected ValidationError")
    }
    .catch {
        case ValidationError:
            @assert_equal(error.field, "email")
            @assert_equal(error.constraint, "Email cannot be empty")
    }
}

#[test]
micro test_retry_exhausted() {
    let mut call_count = 0
    
    try {
        retry(3, { =>
            call_count += 1
            raise "Always fails"
        })
        assert(false, "Expected RetryExhausted")
    }
    .catch {
        case RetryExhausted:
            @assert_equal(error.attempts, 3)
            @assert_equal(call_count, 3)
    }
}
```
