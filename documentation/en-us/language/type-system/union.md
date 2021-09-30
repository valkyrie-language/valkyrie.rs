# Unite Types

Unite types are a powerful type system feature in Valkyrie that represent multiple possible values, defined with an explicit `tag`. The standard form is `[tag(XXXKind)] unite XXX { }`, and the language no longer auto-generates a tag type.

## Basic Unite Types

### Simple Unite Types

```valkyrie
# Result type - represents operation success or failure
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T }
    Fail { error: E }
}

# Option type - represents value presence or absence
[tag(OptionKind)]
unite Option⟨T⟩ {
    Some { value: T }
    None
}
```

### Complex Unite Types

```valkyrie
# JSON value type
[tag(JsonValueKind)]
unite JsonValue {
    Null,
    Bool { value: bool },
    Number { value: f64 },
    String { value: string },
    Array { items: [JsonValue] },
    Object { fields: {string: JsonValue} }
}

# Expression AST
[tag(ExpressionKind)]
unite Expression {
    Literal { value: i32 },
    Variable { name: string },
    Binary {
        left: Expression,
        operator: string,
        right: Expression
    }
}
```

## Using Unite Types

### Pattern Matching

```valkyrie
# Basic pattern matching
let result: Result⟨i32, string⟩ = Fine { value: 42 }
match result {
    case Fine { value }: print("Success: {value}")
    case Fail { error }: print("Failure: {error}")
}
```

# Nested pattern matching
let nested: Result⟨Option⟨i32⟩, string⟩ = Fine { value: Some { value: 42 } }
match nested {
    case Fine { value: Some { value } }: print("Value: {value}")
    case Fine { value: None }: print("No value")
    case Fail { error }: print("Error: {error}")
}
```

### if let Expression

```valkyrie
# Simplified pattern matching
if let Fine { value } = result {
    print("Successfully got value: {value}")
}

# With else branch
if let Some { value } = option {
    process(value)
} else {
    print("Option is empty")
}
```

## Unite Type Methods

### Associated Methods

```valkyrie
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T },
    Fail { error: E },
    
    # Check if successful
    micro is_ok(self) -> bool {
        if let Fine { .. } = self {
            true
        } else {
            false
        }
    }
    
    # Check if failed
    micro is_err(self) -> bool {
        if let Fail { .. } = self {
            true
        } else {
            false
        }
    }
    
    # Get value (may panic)
    micro unwrap(self) -> T {
        if let Fine { value } = self {
            value
        } else {
            panic("Called unwrap on Fail")
        }
    }
    
    # Safe value retrieval
    micro unwrap_or(self, default: T) -> T {
        if let Fine { value } = self {
            value
        } else {
            default
        }
    }
    
    # Map success value
    micro map⟨U⟩(self, f: micro(T) -> U) -> Result⟨U, E⟩ {
        if let Fine { value } = self {
            Fine { value: f(value) }
        } else if let Fail { error } = self {
            Fail { error }
        }
    }
    
    # Map error value
    micro map_err⟨F⟩(self, f: micro(E) -> F) -> Result⟨T, F⟩ {
        if let Fine { value } = self {
            Fine { value }
        } else if let Fail { error } = self {
            Fail { error: f(error) }
        }
    }
}
```

### Option Type Methods

```valkyrie
[tag(OptionKind)]
unite Option<T> {
    Some { value: T },
    None,
    
    # Check if has value
    micro is_some(self) -> bool {
        if let Some { .. } = self {
            true
        } else {
            false
        }
    }
    
    # Check if empty
    micro is_none(self) -> bool {
        if let None = self {
            true
        } else {
            false
        }
    }
    
    # Map value
    micro map<U>(self, f: micro(T) -> U) -> Option<U> {
        if let Some { value } = self {
            Some { value: f(value) }
        } else {
            None
        }
    }
    
    # Filter value
    micro filter(self, predicate: micro(T) -> bool) -> Option<T> {
        if let Some { value } = self {
            if predicate(value) {
                Some { value }
            } else {
                None
            }
        } else {
            None
        }
    }
}
```

## Advanced Features

### Generic Unite Types

```valkyrie
# Multi-parameter generics
[tag(EitherKind)]
unite Either<L, R> {
    Left { value: L },
    Right { value: R }
}

# Constrained generics
[tag(ContainerKind)]
unite Container⟨T⟩ where T: Clone {
    Single { item: T },
    Multiple { items: [T] }
}
```

### Recursive Unite Types

```valkyrie
# Linked list
[tag(ListKind)]
unite List⟨T⟩ {
    Empty,
    Node {
        value: T,
        next: List⟨T⟩
    }
}

# Binary tree
[tag(TreeKind)]
unite Tree⟨T⟩ {
    Leaf { value: T },
    Branch {
        left: Tree⟨T⟩,
        right: Tree⟨T⟩
    }
}
```

## Best Practices

### 1. Use Descriptive Variant Names

```valkyrie
# Good naming
[tag(HttpResponseKind)]
unite HttpResponse {
    Success { data: String, status: u16 },
    ClientError { message: String, code: u16 },
    ServerError { message: String, code: u16 },
    NetworkError { reason: String }
}

# Avoid overly simple naming
[tag(BadKind)]
unite Bad {
    A { x: i32 },
    B { y: String }
}
```

### 2. Reasonable Field Naming

```valkyrie
# When there's only one field, use value
[tag(OptionKind)]
unite Option<T> {
    Some { value: T },
    None
}

# Multiple fields use descriptive names
[tag(PersonKind)]
unite Person {
    Student { name: String, grade: i32 },
    Teacher { name: String, subject: String }
}
```

### 3. Provide Convenience Methods

```valkyrie
[tag(ValidationResultKind)]
unite ValidationResult<T> {
    Valid { data: T },
    Invalid { errors: [String] },
    
    # Convenience method
    micro is_valid(self) -> bool {
        matches!(self, Valid { .. })
    }
    
    micro get_errors(self) -> [String] {
        if let Invalid { errors } = self {
            errors
        } else {
            []
        }
    }
}
```

### 4. Error Handling Pattern

```valkyrie
# Use Result for error handling
micro divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Fail { error: "Division by zero" }
    } else {
        Fine { value: a / b }
    }
}

# Chained error handling
micro process_data(input: String) -> Result<ProcessedData, Error> {
    input
        .parse()
        .map_err { Error::ParseError($e) }?
        .validate()
        .map_err { Error::ValidationError($e) }?
        .transform()
        .map_err { Error::TransformError($e) }
}
```

Unite types are a core feature of Valkyrie's type system, providing a type-safe way to handle multiple possible values, especially suitable for error handling, state representation, and data modeling scenarios.
