# Nyar Virtual Machine Documentation

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/nyar-lang/nyar-vm/workflows/CI/badge.svg)](https://github.com/nyar-lang/nyar-vm/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://nyar-vm.org)

## What is Nyar?

Nyar is a high-performance **virtual machine platform**, **optimization engine**, and **interpreter** designed for modern programming languages. It is **NOT** a programming language itself, but rather provides the runtime infrastructure that programming languages can target.

### Nyar Platform Components

- 🖥️ **Virtual Machine**: High-performance bytecode execution engine
- ⚡ **Optimization Platform**: Advanced JIT compilation and optimization passes
- 🔧 **Multi-Target Compiler**: Generates native code, JavaScript, and WebAssembly
- 🎭 **Runtime Services**: Memory management, garbage collection, and algebraic effects support
- 🛠️ **Developer Tools**: Debugging, profiling, and analysis tools

## Supported Languages

### Valkyrie Programming Language

**Valkyrie** is the primary programming language that targets the Nyar platform. It's a modern functional programming language with algebraic effects, designed to showcase Nyar's capabilities.

```valkyrie
// Valkyrie code compiles to Nyar bytecode
effect Http {
    get(url: String): String
}

micro fetch_user(id: Int) -> User {
    let response = perform Http.get(`/api/users/${id}`);
    parse_json(response)
}
```

### Language Implementation Benefits

By targeting Nyar, language implementers get:

- 🚀 **High Performance**: JIT compilation and advanced optimizations
- 🌐 **Multi-Platform**: Single IR compiles to multiple targets
- 🎯 **Focus on Design**: No need to implement complex runtime systems
- 🔍 **Rich Tooling**: Built-in debugging and profiling support

## Architecture Overview

```
Valkyrie Source Code
        ↓
   Valkyrie Frontend (Parser + Semantic Analysis)
        ↓
      AST (Abstract Syntax Tree)
        ↓
      HIR (High-level IR)
        ↓
      MIR (Mid-level IR)
        ↓
      LIR (Low-level IR)
        ↓
    Nyar VM Platform
        ↓
  Native Code / JavaScript / WebAssembly
```

## Documentation Structure

### For Language Users
- 📚 [Language Guide](guide/) - How to use Valkyrie programming language
- ❓ [FAQ](faq.md) - Frequently asked questions about Valkyrie
- 📖 [Examples](examples/) - Code examples and tutorials

### For Language Implementers
- 🔧 [Development Guide](development/) - How to implement languages targeting Nyar
- 🏗️ [Frontend Implementation](development/valkyrie-frontend.md) - Building language frontends
- 📦 [Backend Integration](development/javascript-backend.md) - Integrating with Nyar backends

### For Platform Maintainers
- ⚙️ [Maintenance Guide](maintenance/) - Internal Nyar platform maintenance
- 🔬 [VM Internals](maintenance/rust-backend.md) - Deep dive into VM implementation
- 📊 [Language Representations](language/) - IR design and implementation

## Quick Start

### Install Nyar Platform

```bash
# Install from source
git clone https://github.com/nyar-lang/nyar-vm.git
cd nyar-vm
cargo install --path .

# Verify installation
nyar --version
```

### Try Valkyrie Language

```bash
# Create a new Valkyrie project
nyar new hello-world --lang valkyrie
cd hello-world

# Build and run
nyar build
nyar run
```

### Compile to Different Targets

```bash
# Compile to JavaScript
nyar build --target js

# Compile to WebAssembly
nyar build --target wasm

# Compile to native binary
nyar build --target native
```

## Platform Benefits

### For Application Developers
- 🎯 **Expressive Language**: Use Valkyrie's modern features like algebraic effects
- 🚀 **High Performance**: Benefit from Nyar's advanced optimizations
- 🌐 **Deploy Anywhere**: Single codebase runs on web, server, and desktop
- 🛠️ **Great Tooling**: Rich IDE support and debugging tools

### For Language Designers
- 🏗️ **Solid Foundation**: Build on proven VM technology
- ⚡ **Performance**: Get JIT compilation and optimizations for free
- 🔧 **Multi-Target**: Automatic support for multiple deployment targets
- 📊 **Analytics**: Built-in profiling and performance analysis

### For Platform Engineers
- 🔬 **Research Platform**: Experiment with new language features
- 📈 **Optimization**: Advanced IR-based optimization pipeline
- 🧪 **Extensible**: Plugin architecture for custom backends
- 📚 **Well-Documented**: Comprehensive documentation and examples

## Community

- 💬 [Discord Server](https://discord.gg/nyar-vm)
- 🐛 [Issue Tracker](https://github.com/nyar-lang/nyar-vm/issues)
- 💡 [Discussions](https://github.com/nyar-lang/nyar-vm/discussions)
- 📧 [Mailing List](https://groups.google.com/g/nyar-vm)

## Contributing

We welcome contributions to both the Nyar platform and Valkyrie language! See our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Areas
- 🔧 VM optimization and performance improvements
- 🌐 New compilation targets and backends
- 📚 Documentation and educational content
- 🛠️ Developer tooling and IDE integration
- 🧪 Testing, benchmarking, and quality assurance

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by LLVM, JVM, and other successful VM platforms
- Built with Rust for memory safety and performance
- Designed for the next generation of programming languages

---

**Ready to explore high-performance language implementation?** [Get started with Nyar!](guide/getting-started.md)