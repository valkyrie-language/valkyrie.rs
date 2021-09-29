# Project Mode

## Overview

Project Mode is Valkyrie's single-project management mode, defined through the `legion.json` file. When a directory contains a `legion.json` file but no `legions.json` file, the directory is recognized as an independent project mode.

## File Structure

```
project-root/
├── legion.json           # Project configuration file
├── library/
│   └── _.vk             # Recommended: namespace root directory definition (_.valkyrie also supported)
├── binary/
│   ├── main.vk          # Registered as command `main`
│   ├── tool1.vk         # Registered as command `tool1`
│   └── tool2/
│       └── _.vk         # Registered as command `tool2`
├── tests/               # Test files
├── docs/                # Documentation directory
├── examples/            # Example code
└── target/              # Build output
```

## legion.json Configuration

`legion.json` is a JSON5 format configuration file that supports comments and more flexible syntax:

### Basic Configuration

```json5
{
    // Project basic information
    "name": "my-valkyrie-project",
    "version": "1.0.0",
    "description": "A Valkyrie language project",
    "type": "application", // or "library", "plugin", "tool"
    
    // Author information
    "authors": [
        "Developer Name <email@example.com>"
    ],
    
    // License
    "license": "MIT",
    
    // Repository information
    "repository": "https://github.com/user/my-valkyrie-project",
    "homepage": "https://my-project.dev",
    "documentation": "https://docs.my-project.dev",
    
    // Publish configuration
    "publish": true,
    "private": false,
    
    // Language version
    "edition": "2024",
    
    // Entry files
    "main": "binary/main.vk",
    "lib": "library/_.vk"
}
```

### Dependency Management

```json5
{
    // Runtime dependencies
    "dependencies": {
        "valkyrie-std": "^1.0.0",
        "serde": {
            "version": "^1.0",
            "features": ["derive"]
        },
        "local-lib": {
            "path": "../local-lib"
        },
        "git-dep": {
            "git": "https://github.com/user/repo.git",
            "branch": "main"
        }
    },
    
    // Build-time dependencies
    "build-dependencies": {
        "build-script": "^0.1.0"
    },
    
    // Development dependencies
    "dev-dependencies": {
        "test-framework": "^2.0.0",
        "benchmark": "^1.5.0"
    },
    
    // Optional dependencies
    "optional-dependencies": {
        "feature-x": "^1.0.0",
        "feature-y": "^2.0.0"
    }
}
```

### Features

```json5
{
    // Feature definitions
    "features": {
        "default": ["std", "serde"],
        "std": [],
        "serde": ["dep:serde"],
        "async": ["dep:tokio"],
        "full": ["std", "serde", "async"]
    }
}
```

### Build Configuration

```json5
{
    // Build settings
    "build": {
        "target": "native", // or "wasm", "js", "llvm"
        "optimization": "release", // or "debug", "size", "speed"
        "output": "target/",
        "incremental": true,
        "parallel": true,
        "cache": true
    },
    
    // Compiler options
    "compiler": {
        "warnings": "deny",
        "errors": "abort",
        "debug-info": true,
        "strip-symbols": false
    },
    
    // Linker options
    "linker": {
        "lto": true,
        "strip": false,
        "static": false
    }
}
```

### Script Commands

```json5
{
    // Custom scripts
    "scripts": {
        "build": "valkyrie build --release",
        "test": "valkyrie test",
        "run": "valkyrie run",
        "clean": "valkyrie clean",
        "fmt": "valkyrie fmt",
        "lint": "valkyrie lint",
        "doc": "valkyrie doc --open",
        "publish": "valkyrie publish",
        "install": "valkyrie install",
        "dev": "valkyrie run --watch",
        "benchmark": "valkyrie bench"
    }
}
```

## Project Types

### 1. Application

```json5
{
    "type": "application",
    "main": "binary/main.vk",
    "build": {
        "target": "native",
        "executable": "my-app"
    }
}
```

### 2. Library

```json5
{
    "type": "library",
    "lib": "library/_.vk",
    "build": {
        "target": "library",
        "crate-type": ["lib", "dylib"]
    }
}
```

### 3. Plugin

```json5
{
    "type": "plugin",
    "plugin": {
        "interface": "valkyrie-plugin-api",
        "version": "1.0.0"
    }
}
```

### 4. Tool

```json5
{
    "type": "tool",
    "bin": [
        {
            "name": "my-tool",
            "path": "src/bin/tool.vk"
        }
    ]
}
```

## Semantic Features

### 1. Dependency Resolution

- **Version Constraints**: Supports semantic versioning constraints
- **Path Dependencies**: Supports local path dependencies
- **Git Dependencies**: Supports Git repository dependencies
- **Conditional Dependencies**: Feature-based conditional dependencies

### 2. Feature System

- **Default Features**: Automatically enabled feature sets
- **Optional Features**: On-demand enabled functionality features
- **Feature Composition**: Dependencies between features
- **Feature Propagation**: Propagation mechanism for dependency features

### 3. Build System

- **Incremental Compilation**: Only compile changed parts
- **Parallel Build**: Multi-core parallel compilation
- **Cache Mechanism**: Build result caching
- **Cross-Compilation**: Supports multi-target platform compilation

### 4. Package Management

- **Version Management**: Automatic version number management
- **Publishing Process**: Standardized publishing workflow
- **Dependency Locking**: Ensures build reproducibility
- **Security Checks**: Dependency security auditing

## Development Workflow

### 1. Project Initialization

```bash
# Create new project
valkyrie new my-project
cd my-project

# Or initialize existing directory
valkyrie init
```

### 2. Dependency Management

```bash
# Add dependency
valkyrie add serde@^1.0

# Add development dependency
valkyrie add --dev test-framework

# Update dependencies
valkyrie update

# Remove dependency
valkyrie remove old-dep
```

### 3. Build and Test

```bash
# Build project
valkyrie build

# Run project
valkyrie run

# Run tests
valkyrie test

# Generate documentation
valkyrie doc

# Run binary programs
v main                     # Run binary/main.vk
v tool1                    # Run binary/tool1.vk
v tool2                    # Run binary/tool2/_.vk
```

### 4. Publishing Process

```bash
# Check project
valkyrie check

# Run full tests
valkyrie test --all-features

# Publish to repository
valkyrie publish
```

## Configuration Examples

### Web Application Project

```json5
{
    "name": "web-app",
    "version": "1.0.0",
    "type": "application",
    "main": "src/main.vk",
    
    "dependencies": {
        "valkyrie-web": "^2.0.0",
        "valkyrie-router": "^1.5.0",
        "valkyrie-templates": "^1.0.0"
    },
    
    "features": {
        "default": ["server"],
        "server": ["dep:valkyrie-web"],
        "client": ["dep:valkyrie-wasm"]
    },
    
    "build": {
        "target": "wasm",
        "optimization": "size"
    },
    
    "scripts": {
        "dev": "valkyrie run --watch --features server",
        "build-client": "valkyrie build --target wasm --features client",
        "serve": "valkyrie run --release"
    }
}
```

### Library Project

```json5
{
    "name": "data-structures",
    "version": "2.1.0",
    "type": "library",
    "lib": "src/lib.vk",
    "description": "High-performance data structures for Valkyrie",
    
    "authors": ["Library Team <team@example.com>"],
    "license": "Apache-2.0",
    "repository": "https://github.com/team/data-structures",
    
    "dependencies": {
        "valkyrie-std": "^1.0.0"
    },
    
    "dev-dependencies": {
        "criterion": "^0.5.0",
        "proptest": "^1.0.0"
    },
    
    "features": {
        "default": ["std"],
        "std": [],
        "no-std": [],
        "serde": ["dep:serde"],
        "parallel": ["dep:rayon"]
    },
    
    "build": {
        "optimization": "speed",
        "lto": true
    }
}
```

## Best Practices

### 1. Project Structure

- **Clear Layering**: Reasonably organize source code structure
- **Modular Design**: Use module system to organize code
- **Test Coverage**: Write tests for all public APIs
- **Complete Documentation**: Provide complete API documentation

### 2. Dependency Management

- **Minimal Dependencies**: Only add necessary dependencies
- **Version Locking**: Use precise version constraints
- **Regular Updates**: Periodically update dependency versions
- **Security Auditing**: Regularly perform security audits

### 3. Build Optimization

- **Incremental Build**: Utilize incremental compilation features
- **Parallel Build**: Enable parallel compilation
- **Cache Utilization**: Fully utilize build cache
- **Target Optimization**: Optimize for target platforms

### 4. Version Management

- **Semantic Versioning**: Follow semantic versioning specification
- **Changelog**: Maintain detailed changelog
- **Backward Compatibility**: Maintain API backward compatibility
- **Deprecation Policy**: Reasonable API deprecation policy

## Tool Integration

### IDE Support

- **Project Recognition**: IDE automatically recognizes project configuration
- **Dependency Management**: Graphical dependency management interface
- **Build Integration**: Integrated build and run functionality
- **Debugging Support**: Complete debugging functionality support

### CI/CD Integration

- **Automatic Build**: Configuration-based automatic builds
- **Test Execution**: Automated test execution
- **Quality Checks**: Code quality and security checks
- **Automatic Publishing**: Tag-based automatic publishing

## Migration and Compatibility

### Configuration Migration

- **Version Upgrade**: Configuration file version upgrades
- **Format Conversion**: Conversion from other formats
- **Backward Compatibility**: Maintain backward compatibility
- **Migration Tools**: Provide automatic migration tools

### Ecosystem Compatibility

- **Standard Compliance**: Follow community standards
- **Toolchain Integration**: Integration with existing toolchains
- **Platform Support**: Multi-platform support
- **Interoperability**: Interoperability with other languages
