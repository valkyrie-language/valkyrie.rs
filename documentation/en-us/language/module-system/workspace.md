# Workspace Mode

## Overview

Workspace Mode is Valkyrie's multi-project management mode, defined through the `legions.json` file. When a project root directory contains a `legions.json` file, the directory is recognized as workspace mode.

## File Structure

```
workspace-root/
├── legions.json          # Workspace configuration file
├── project-a/
│   ├── legion.json       # Project A configuration
│   ├── library/
│   │   └── _.vk         # Recommended: namespace root directory definition
│   └── binary/
│       └── main.vk      # Registered as command `main`
├── project-b/
│   ├── legion.json       # Project B configuration
│   ├── library/
│   │   └── _.vk         # Library code directory
│   └── binary/
│       ├── tool1.vk     # Registered as command `tool1`
│       └── tool2/
│           └── _.vk     # Registered as command `tool2`
└── shared/
    └── common/
        └── _.vk
```

## legions.json Configuration

`legions.json` is a JSON5 format configuration file that supports comments and more flexible syntax:

```json5
{
    // Workspace basic information
    "name": "valkyrie-workspace",
    "version": "1.0.0",
    "description": "Valkyrie language workspace",
    
    // Private workspace identifier
    "private": true,
    
    // Member project list
    "members": [
        "projects/*",
        "tools/build-tools"
    ],
    
    // Excluded directories
    "exclude": [
        "legacy/*",
        "experiments/*",
        "temp/*"
    ],
    
    // Default members (for quick builds)
    "default-members": [
        "projects/valkyrie-core",
        "projects/valkyrie-std"
    ],
    
    // Workspace-level scripts
    "scripts": {
        "build": "legion build --release",
        "test": "legion test --release",
        "fmt": "legion fmt --all",
        "clean": "legion clean",
        "publish": "git push && git push --tags --prune",
        "upgrade": "legion upgrade --workspace"
    },
    
    // Shared dependency configuration
    "dependencies": {
        "shared": {
            "serde": "^1.0",
            "tokio": "^1.0"
        }
    },
    
    // Build configuration
    "build": {
        "profile": {
            "release": {
                "lto": true,
                "opt-level": "s"
            }
        }
    }
}
```

## Semantic Features

### 1. Project Discovery

- **Automatic Discovery**: Automatically discover sub-projects based on `members` patterns
- **Explicit Exclusion**: Exclude unnecessary directories through `exclude`

### 2. Property Inheritance

Workspace supports automatic property inheritance mechanism. If member projects do not explicitly define certain metadata, they will automatically inherit from the workspace root's `legions.json`.

#### Workspace Configuration (`legions.json`)
```json5
{
    "name": "my-workspace",
    "version": "1.2.3",
    "description": "A unified workspace",
    "authors": ["Valkyrie Team"],
    "license": "MIT",
    "members": ["projects/*"],
    "dependencies": {
        "shared_lib": "1.0.0"
    }
}
```

#### Member Project Configuration (`projects/my-pkg/legion.json`)
Common metadata (such as version, authors, license, etc.) is **automatically inherited**, no additional configuration needed:

```json5
{
    "name": "my-pkg",
    // version automatically inherited as 1.2.3
    // description automatically inherited as "A unified workspace"
    
    "dependencies": {
        // Dependencies are not automatically inherited, must be explicitly declared
        // To use workspace-defined version, simply set to true
        "shared_lib": true
    }
}
```

Automatically inherited properties include:
- `version`
- `description`
- `authors`
- `license`
- `repository`
- `homepage`
- `edition`

### 3. Dependency Sharing

- **Direct Reference**: Packages within the workspace can directly `using` each other by name.
- **Dependency Declaration**: Although dependency versions can be inherited from the workspace, each package still needs to explicitly list the dependencies it uses in its `legion.json`.
- **Dependency Hoisting**: Dependencies defined in `legions.json` serve as a shared pool, member projects reference them via `true`.
- **Recursive Search**: Supports wildcard patterns for recursive project discovery

### 2. Dependency Management

- **Shared Dependencies**: Define shared dependency versions at workspace level
- **Version Unification**: Ensure all member projects use consistent dependency versions
- **Dependency Resolution**: Optimized dependency resolution and build caching

### 3. Build Coordination

- **Parallel Build**: Supports parallel building of member projects
- **Incremental Build**: Intelligently detects changes, only builds necessary projects
- **Build Order**: Automatically determines build order based on dependency relationships

### 4. Script Execution

- **Workspace Scripts**: Script commands executed at workspace level
- **Batch Operations**: Execute same operation on all member projects
- **Conditional Execution**: Conditionally execute scripts based on project status

## Inter-Project Dependencies

### Internal Dependencies

```json5
// project-a/legion.json
{
    "dependencies": {
        "project-b": { "path": "../project-b" },
        "shared-utils": true
    }
}
```

### Dependency Resolution Rules

1. **Path Dependencies**: Use relative paths to reference other member projects
2. **Workspace Dependencies**: Use `true` to reference workspace-level dependencies
3. **Version Constraints**: Support version ranges and exact version constraints

## Development Workflow

### 1. Initialize Workspace

```bash
# Create new workspace
mkdir my-workspace
cd my-workspace

# Initialize legions.json
echo '{ "private": true, "members": ["projects/*"] }' > legions.json
```

### 2. Add Member Projects

```bash
# Create new project
mkdir projects/my-project
cd projects/my-project

# Initialize project configuration
echo '{ "name": "my-project", "version": "0.1.0" }' > legion.json
```

### 3. Build and Test

```bash
# Build all projects
valkyrie build

# Test all projects
valkyrie test

# Build specific project
valkyrie build --package my-project

# Run binary programs
v tool1                    # Run binary/tool1.vk
v tool2                    # Run binary/tool2/_.vk
```

## Best Practices

### 1. Project Organization

- **Logical Grouping**: Organize projects by function or layer
- **Clear Naming**: Use consistent project naming conventions
- **Complete Documentation**: Provide complete documentation for each project

### 2. Dependency Management

- **Version Locking**: Lock key dependency versions at workspace level
- **Minimal Dependencies**: Avoid introducing unnecessary dependencies
- **Regular Updates**: Periodically update and review dependencies

### 3. Build Optimization

- **Cache Utilization**: Fully utilize build cache
- **Parallel Build**: Reasonably configure parallel build parameters
- **Incremental Build**: Optimize code structure to support incremental builds

## Tool Integration

### IDE Support

- **Project Import**: IDE automatically recognizes and imports workspace structure
- **Code Navigation**: Cross-project code navigation and reference lookup
- **Debugging Support**: Unified debugging and run configuration

### CI/CD Integration

- **Build Matrix**: Supports multi-project build matrices
- **Test Reports**: Aggregated test results and coverage reports
- **Deployment Coordination**: Coordinated multi-project deployment workflows

## Migration Guide

### From Single Project to Workspace

1. **Create legions.json**: Create workspace configuration in root directory
2. **Reorganize Project Structure**: Move existing code to sub-project directories
3. **Update Dependencies**: Adjust inter-project dependency relationships
4. **Test Build**: Verify build and testing of new structure

### Compatibility Considerations

- **Backward Compatibility**: Maintain compatibility with existing toolchains
- **Gradual Migration**: Support gradual project migration
- **Tool Adaptation**: Ensure development tools correctly recognize new structure
