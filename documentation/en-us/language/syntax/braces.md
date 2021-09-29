# Braces Syntax (ApplyBlock)

Valkyrie's braces syntax, known as **ApplyBlock**, is a unified syntactic structure that provides flexible configuration and construction capabilities.

## Core Concept

An `ApplyBlock` is a code block enclosed in braces `{ }` that follows a function call or object construction. The content inside the block is interpreted based on context:

```valkyrie
# ApplyBlock after constructor
let person = Person {
    name = "Alice"
    age = 30
}

# ApplyBlock after function call
configure_server {
    host = "localhost"
    port = 8080
}
```

## Unified Operations

ApplyBlock unifies four core operations:

### 1. Field Assignment

```valkyrie
let config = Config {
    host = "localhost"
    port = 8080
    debug = true
}
```

### 2. Method Calls

```valkyrie
let builder = StringBuilder {
    .append("Hello")
    .append(" ")
    .append("World")
}
```

### 3. Event Binding

```valkyrie
Button("Click me") {
    on_click {
        print("Clicked!")
    }
}
```

### 4. Child Node Injection

```valkyrie
Column {
    Text("First")
    Text("Second")
    Text("Third")
}
```

## Application Scenario: Object Construction

ApplyBlock provides a clean syntax for the Builder pattern:

```valkyrie
class HttpRequest {
    url: string
    method: HttpMethod
    headers: Map<string, string>
    body: Option<string>
}

let request = HttpRequest {
    url = "https://api.example.com"
    method = HttpMethod.POST
    headers = {
        "Content-Type": "application/json",
        "Authorization": "Bearer token"
    }
    body = Some("{\"key\": \"value\"}")
}
```

## Semantic Interpretation

The compiler interprets ApplyBlock content based on the target type:

1. **Structure/Class**: Assignments map to field initialization
2. **Builder Types**: Method calls chain on the builder
3. **UI Components**: Nested blocks become child elements
4. **Configuration**: Mixed assignments and method calls

## Type Safety

ApplyBlock is fully type-checked:

```valkyrie
let config = Config {
    host = "localhost"  # OK: string
    port = "8080"       # Error: expected i32, found string
}
```

## Nested ApplyBlocks

ApplyBlocks can be nested for complex structures:

```valkyrie
let app = Application {
    database = DatabaseConfig {
        host = "db.example.com"
        port = 5432
        name = "myapp"
    }
    
    server = ServerConfig {
        host = "0.0.0.0"
        port = 8080
    }
}
```

## Comparison with Other Syntaxes

| Feature | ApplyBlock | JSON | YAML |
|---------|------------|------|------|
| Type safety | Full | None | None |
| Method calls | Yes | No | No |
| Code execution | Yes | No | No |
| IDE support | Complete | Limited | Limited |

---
**Related Sections**:
- [V-Grammar](./v-grammar.md) - UI-specific ApplyBlock usage
- [X-Grammar](./x-grammar.md) - XML-like syntax for UI
