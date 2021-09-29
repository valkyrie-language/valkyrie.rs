# Frequently Asked Questions (FAQ)

This page collects common questions and answers about Valkyrie development.

## Language Fundamentals

### Q: What makes Valkyrie different from other functional languages?

A: Valkyrie's unique features include:
- **Algebraic Effect System**: Native support for algebraic effects, elegant side effect handling
- **Multi-target Compilation**: Can compile to WebAssembly, JavaScript, and native code
- **Modern Syntax**: Combines functional features with modern syntax design
- **Progressive Adoption**: Seamless integration with existing JavaScript/TypeScript projects
- **Strong Type Inference**: Advanced type system, reducing explicit type annotations

### Q: What are algebraic effects? Why are they important?

A: Algebraic effects are an abstraction mechanism for handling side effects:
- **Unified Abstraction**: Unified handling of exceptions, async, state management, and other side effects
- **Composability**: Effects can be freely combined and nested
- **Inversion of Control**: The caller decides how to handle effects, not the callee
- **Type Safety**: Effects are reflected in the type system

```valkyrie
class State<T> {
    get(): T
    set(value: T): void
}

micro counter() -> i32 {
    let current = @State::get()
    @State::set(current + 1)
    current + 1
}
```

### Q: What data types does Valkyrie support?

A: Valkyrie supports a rich type system:
- **Primitive Types**: i32, f32, utf8, bool, void
- **Container Types**: List⟨T⟩, Array⟨T⟩, Map⟨K, V⟩, Set⟨T⟩
- **Optional Types**: Option⟨T⟩ (Some { value: T } | None)
- **Result Types**: Result⟨T, E⟩ (Fine { value: T } | Fail { error: E })
- **Function Types**: (A, B) -> C
- **Algebraic Data Types**: Custom sum and product types
- **Effect Types**: Function types with effect annotations

## Syntax and Features

### Q: How to define and use algebraic data types?

A: Use an explicitly tagged `unite` definition. The standard form is `[tag(XXXKind)] unite XXX { }`:

```valkyrie
// Sum type (explicitly tagged closed variant family)
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T };
    Fail { error: E };
}

// Product type (struct)
struct User {
    id: i32;
    name: utf8;
    email: utf8?;
}

// Pattern matching
match result {
    case Fine { value }: print("Success: {value}");
    case Fail { error }: print("Error: {error}");
};
```

### Q: How to handle asynchronous operations?

A: Valkyrie supports native `async/await` syntax:

```valkyrie
micro fetch_user_data(id: i32) -> User {
    let response = fetch("/api/users/{id}").await;
    parse_json(response);
}

// await can also be used at the top level
let user = fetch_user_data(42).await;
print("User: {user.name}");
```

### Q: How to handle errors?

A: Valkyrie provides multiple error handling methods:

```valkyrie
// 1. Using Result type
micro divide(a: f64, b: f64) -> Result⟨f64, utf8⟩ {
    if b == 0.0 {
        Fail { error: "Division by zero" };
    } else {
        Fine { value: a / b };
    };
}

// 2. Using exception effect
class Exception {
    throw(message: utf8): Never
}

micro safe_divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        Exception::throw("Division by zero");
    } else {
        a / b
    }
}
```

## Compilation and Deployment

### Q: How does Valkyrie compile to different targets?

A: Valkyrie supports multi-target compilation:

```bash
# Compile to WebAssembly
legion build --target wasm

# Compile to JavaScript
legion build --target js

# Compile to native code
legion build --target native

# Compile to TypeScript definitions
legion build --target ts-defs
```

### Q: How to integrate with existing JavaScript projects?

A: Valkyrie provides seamless integration:

```valkyrie
// Import external modules
using hxo::std::fetch;
using hxo::std::console;

// Public function definition
@export(js)
micro greet(name: utf8) -> utf8 {
    "Hello, {name}!"
}

// Using JavaScript objects
micro process_data(data: JSObject) -> JSObject {
    // Processing logic
    data
}
```

### Q: How is the performance? What optimizations are available?

A: Valkyrie provides various performance optimizations:
- **Tail Call Optimization**: Automatic tail recursion optimization
- **Inline Optimization**: Small functions are automatically inlined
- **Dead Code Elimination**: Removes unused code
- **Effect Optimization**: Compile-time effect handling optimization
- **Memory Management**: Smart garbage collection and memory reuse

## Tools and Ecosystem

### Q: What development tools are supported?

A: Valkyrie provides a complete toolchain:
- **Compiler**: `legion` CLI tool
- **Package Manager**: Built-in dependency management
- **Formatter**: `legion fmt` code formatting
- **Language Server**: VS Code, Vim, and other editor support
- **Debugger**: Source-level debugging support
- **Testing Framework**: Built-in unit testing and integration testing

### Q: What is the configuration file format?

A: Valkyrie uses `voc.config.von` as the configuration file:

```von
name: "my-project"
version: "0.1.0"
dependencies: {
    "std": "0.1.0"
}
```

### Q: How to manage multi-package workspaces?

A: Use `legions.von` to manage workspaces:

```von
workspace: {
    members: [
        "packages/*"
    ]
}
```

### Q: How to write and run tests?

A: Use the built-in testing framework:

```valkyrie
// Unit test
#test
micro test_addition() {
    @assert_eq(add(2, 3), 5)
    @assert_eq(add(-1, 1), 0)
}

// Property test
#test
micro test_addition_commutative() {
    forall (a: i32, b: i32) {
        @assert_eq(add(a, b), add(b, a))
    }
}

// Effect test
#test
micro test_state_effect() {
    let result = try {
        counter()
    } catch State::get || {
        resume 0;
    } catch State::set |value| {
        resume ();
    }
    @assert_eq(result, 1);
}
```

### Q: How to manage project dependencies?

A: Use the `voc.config.von` configuration file:

```von
{
    name: "my-project",
    version: "0.1.0",
    authors: ["Your Name <your.email@example.com>"],
    dependencies: {
        std: "1.0",
        http: "0.3",
        json: "0.2"
    },
    build: {
        targets: ["js", "wasm"],
        optimization: "release"
    }
}
```

## Learning and Community

### Q: How to learn Valkyrie?

A: Recommended learning path:
1. **Basic Syntax**: Start with functional programming concepts
2. **Type System**: Understand algebraic data types and pattern matching
3. **Algebraic Effects**: Master effect definition and handling
4. **Practical Projects**: Build small applications
5. **Advanced Features**: Learn performance optimization and tool usage

### Q: What learning resources are available?

A: Available learning resources:
- **Official Tutorial**: [Getting Started Guide](/guide/getting-started)
- **Example Projects**: [Code Examples](/examples/)
- **API Documentation**: Complete standard library documentation
- **Community Forum**: GitHub Discussions
- **Video Tutorials**: YouTube channel

### Q: How to contribute to the Valkyrie project?

A: Ways to contribute:
1. **Report Issues**: Submit bug reports and feature requests
2. **Improve Documentation**: Enhance documentation and examples
3. **Write Code**: Implement new features or fix issues
4. **Testing Feedback**: Use pre-release versions and provide feedback
5. **Community Support**: Help other users solve problems

### Q: What is Valkyrie's development roadmap?

A: Main development directions:
- **Language Features**: Module system, macro system, concurrency primitives
- **Tool Improvements**: Better error messages, debugging experience, IDE support
- **Performance Optimization**: Compilation speed, runtime performance, memory usage
- **Ecosystem Building**: Standard library expansion, third-party packages, framework support
- **Platform Support**: More compilation targets, mobile platforms, embedded systems

---

If you don't find the answer to your question here, please:
- Check the [Official Documentation](/guide/)
- Submit a [GitHub Issue](https://github.com/valkyrie-lang/valkyrie/issues)
- Join the [Community Discussion](https://github.com/valkyrie-lang/valkyrie/discussions)
- Join the [Discord Community](https://discord.gg/valkyrie-lang)
