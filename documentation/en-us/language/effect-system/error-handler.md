# Error Handling

Valkyrie uses `try` statements and `catch` mechanisms to handle errors. `try` is a standalone statement that can be combined with the type system.

## Try Statement

### Basic Try Syntax

```valkyrie
# try is a standalone statement, returns Result type
let result = try Result⟨string⟩ {
    read_file("config.txt")?
}

# try with error type
let data = try Result⟨Data, ParseError⟩ {
    let content = read_file("data.json")?
    parse_json(content)?
}

# Simplified form
let value = try {
    risky_operation()?
}
```

### Try with Optional Types

```valkyrie
# Handle operations that may fail
let maybe_value = try i32? {
    let input = get_user_input()?
    parse_number(input)?
}

# Chained operations
let result = try User? {
    let id = extract_user_id(request)?
    let user = find_user_by_id(id)?
    validate_user(user)?
}
```

## Catch Handling

### Non-main Control Flow Catch

```valkyrie
# Use .catch to handle errors
let config = try Result⟨Config⟩ {
    read_config_file()?
}
.catch {
    case Fail(FileNotFound(path)): 
        create_default_config(path)
    case Fail(ParseError(msg)):
        log_error(msg)
        Config::default()
    case Fail(error):
        print("Unexpected error: ${error}")
        Config::empty()
}

# Named catch
let data = try Result⟨Data⟩ {
    fetch_remote_data()?
}
catch network_error {
    case Fail(TimeoutError): retry_with_backoff()
    case Fail(ConnectionError(msg)): use_cached_data()
    case Fail(error):
        log_error(error)
        Data::empty()
}
```

### Match-style Catch

```valkyrie
# catch and match are duals, with identical capabilities
let user_data = try Result⟨UserData⟩ {
    let raw = fetch_user_data(user_id)?
    validate_and_parse(raw)?
}
.catch {
    case Fail(ValidationError { field, message }):
        show_field_error(field, message)
        UserData::guest()
    case Fail(NetworkError { code, .. }) if code >= 500:
        # Server error, retry later
        schedule_retry()
        UserData::cached(user_id)
    case Fail(NetworkError { code, .. }) if code >= 400:
        # Client error
        UserData::error(code)
    else: UserData::unknown_error()
}
```

## Error Propagation

### Question Mark Operator

```valkyrie
# ? operator for error propagation
micro process_file(path: string) -> Result⟨string, FileError⟩ {
    let content = read_file(path)?  # If fails, return error directly
    let processed = transform_content(content)?
    validate_result(processed)?
}

# Use in try block
let final_result = try Result⟨ProcessedData⟩ {
    let raw = fetch_data()?
    let cleaned = clean_data(raw)?
    let validated = validate_data(cleaned)?
    process_final(validated)?
}
```

### Error Transformation

```valkyrie
# Automatic error transformation
micro read_and_parse(path: string) -> Result⟨Config, AppError⟩ {
    try Result⟨Config, AppError⟩ {
        let content = read_file(path)?  # FileError -> AppError
        let config = parse_json(content)?  # ParseError -> AppError
        validate_config(config)?  # ValidationError -> AppError
    }
}

# Manual error transformation
let result = try Result⟨Data⟩ {
    fetch_data().map_err { $e -> AppError::Network($e) }?
}
```

## Custom Error Types

```valkyrie
# Define error types
unite AppError {
    Network(NetworkError),
    Parse(ParseError),
    Validation { field: string, message: string },
    IO(IOError)
}

# Implement error transformation
imply AppError: From⟨NetworkError⟩ {
    micro from(err: NetworkError) -> AppError {
        AppError::Network(err)
    }
}

# Use custom error
micro load_user_config(user_id: string) -> Result⟨UserConfig, AppError⟩ {
    try Result⟨UserConfig, AppError⟩ {
        let path = get_config_path(user_id)?
        let content = read_file(path)?
        let config = parse_config(content)?
        validate_user_config(config)?
    }
}
```

## Error Recovery Patterns

### Fallback Strategy

```valkyrie
# Multi-level fallback
let avatar = try Image? {
    load_from_cdn(user_id)?
}
.catch {
    case null: try Image? {
        load_from_cache(user_id)?
    }
    .catch {
        case null: default_avatar()
        else: null
    }
    else: default_avatar()
}

# Retry mechanism
let data = try Result⟨Data⟩ {
    fetch_with_retry(url, max_retries = 3)?
}
.catch {
    case Fail(RetryExhausted(attempts)):
        log_error("Failed after ${attempts} attempts")
        use_fallback_data()
    case Fail(error):
        log_error("Unexpected error: ${error}")
        Data::empty()
}
```

### Partial Recovery

```valkyrie
# Handle partial failures
let results = try Result⟨[ProcessedItem]⟩ {
    items.map { $item ->
        try ProcessedItem? {
            process_item($item)?
        }
        .catch {
            case ProcessingError(msg):
                log_warning("Skipping item: ${msg}")
                None  # Skip failed items
            else: None
        }
    }).filter_map { $x }.collect()
}
```

## Best Practices

### 1. Error Type Design

```valkyrie
# Custom validation error
union ValidationError {
    InvalidFormat { field: string, expected: string },
    ValueOutOfRange { field: string, min: i32, max: i32 },
    RequiredFieldMissing { field: string }
}

# Validation effect definition
effect Validation {
    micro validate_user(user: User) -> Result⟨(), [ValidationError]⟩
}

# Implement validation logic
micro check_user(user: User) -> Result⟨(), [ValidationError]⟩ {
    let mut errors = []
    
    if user.name.is_empty() {
        errors.push(ValidationError::RequiredFieldMissing { field: "name" })
    }
    
    if user.age < 0 || user.age > 150 {
        errors.push(ValidationError::ValueOutOfRange { 
            field: "age", 
            min: 0, 
            max: 150 
        })
    }
    
    if errors.is_empty() {
        Fine(())
    } else {
        Fail(errors)
    }
}

# Contextual information
class ContextualError {
    operation: string,
    context: Map⟨string, string⟩,
    source: Error
}
```

### 2. Error Handling Strategy

```valkyrie
# Handle at the nearest location
micro validate_user_input(input: UserInput) -> Result<ValidatedInput, ValidationError> {
    try Result<ValidatedInput> {
        let email = validate_email(input.email)?
        let age = validate_age(input.age)?
        let name = validate_name(input.name)?
        class: ValidatedInput { email, age, name }
    }
}

# Unified error handling
micro main() {
    let result = try Result<()> {
        run_application()?
    }
    .catch {
        case ConfigError(msg):
            print("Configuration error: ${msg}")
            exit(1)
        case NetworkError(msg):
            print("Network error: ${msg}")
            exit(2)
        case error:
            print("Unexpected error: ${error}")
            exit(99)
    }
}
```

### 3. Resource Management

```valkyrie
# Use RAII pattern
class FileHandle {
    path: String,
    handle: File
}

impl Drop for FileHandle {
    micro drop() {
        self.handle.close()
    }
}

# Safe resource usage
micro process_file_safely(path: String) -> Result<String, FileError> {
    try Result<String> {
        let file = FileHandle::open(path)?
        let content = file.read_all()?
        process_content(content)?
    }  # file automatically closes
}
```
