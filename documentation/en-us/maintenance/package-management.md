# Package Management and Symbol Resolution Mechanism

This document introduces Valkyrie's package management mechanism, focusing on distinguishing Valkyrie's internal symbol resolution mechanism from the external package management mechanism introduced by Legion, and provides integration guidelines for custom package managers.

## 1. Core Concept Distinction

In Valkyrie, "package management" is divided into two levels:

### 1.1 Internal Symbol Resolution Mechanism
This is part of the compiler core (HIR stage). It is responsible for handling `namespace` and `using` statements in source code and binding names to specific symbols (such as functions, classes, constants).

- **Responsibility**: Build symbol tables, handle scopes, verify access permissions.
- **Core Data Structures**: `GlobalSymbol`, `NamespaceMap`.
- **Resolution Logic**: When the compiler encounters `using A::B`, it looks for the top-level namespace named `A` and locates `B` within it.
- **Limitation**: The internal mechanism itself does not know how to load files from disk that have not yet been recognized by the compiler. It relies on an externally provided "filesystem view".

### 1.2 Legion Package Management Mechanism
Legion is Valkyrie's default package manager. It exists outside the compiler core (usually driven by LSP or CLI) and is responsible for mapping the physical filesystem to the compiler's logical view.

> **Note**: Terms such as `vendor` and `registry` are specific concepts of the Legion package manager, not semantics defined by the Valkyrie language itself. The Valkyrie language only focuses on the logical organization of symbols, while Legion is responsible for mapping these logical symbols to physical storage (such as the `vendor` directory) or remote services (such as `jsr`, `npm`, and other registries).

Legion supports two operating modes:

#### Single Project Mode
Driven by `legion.json` in the project root directory.
- **Applicable Scenario**: Independent libraries or applications.
- **Core Logic**: Legion uses the directory where `legion.json` is located as the root, resolves its `dependencies` field, and manages the project's private `vendor` cache (if it exists).
- **Identifier**: `name` and `version` defined in the configuration file serve as the top-level namespace for the package in the symbol system.

#### Workspace Mode
Driven by `legions.json` (note the plural form) in the root directory.
- **Applicable Scenario**: Large projects containing multiple interrelated packages (Monorepo).
- **Core Logic**: 
    - `legions.json` defines the member paths of the workspace (`members`).
    - Member packages can directly reference each other by package name without declaring relative path dependencies in their respective `legion.json`.
    - The workspace shares a unified `vendor` directory, avoiding repeated downloads of the same dependencies.
- **Attribute Inheritance**: Member packages can automatically inherit common attributes from `legions.json` (such as `version`, `description`, `authors`, `license`, etc.) without repeating them in subprojects.
- **Dependency Hoisting**: Workspace-level dependency definitions serve as fallbacks for all member packages, i.e., "workspace dependencies" in the resolution order. Member packages explicitly declare use of the workspace-unified version by setting the dependency value to `true`.

- **Responsibility**: Discover workspace projects, resolve dependency versions, manage `vendor` directory, provide mapping from package names to physical paths.
- **Dependency Resolution Order**:
    1.  **Self Project**: First check files within the current project.
    2.  **Project Dependencies**: Check direct dependencies defined in `legion.json`.
    3.  **Workspace Dependencies**: Check shared dependencies defined in `legions.json`.
    4.  **Vendor Cache**: Recursively search for matching versions in the `%LEGION_ROOT%/vendor/` directory.

---

## 2. Collaboration Process: From Using to File Loading

When a user writes `using my_pkg::utils` in code, the collaboration process is as follows:

1.  **Symbol Resolution Request**: The compiler attempts to find `my_pkg` in the currently known namespaces.
2.  **Trigger Package Lookup**: If `my_pkg` is not found, the compiler (or LSP state manager) requests `LegionManager` to resolve the package name.
3.  **Path Location**: `LegionManager` finds the root directory of `my_pkg` on disk based on the resolution order above (e.g., `vendor/github.com/user/my_pkg`).
4.  **On-demand Indexing**: LSP retrieves all `.vk` source files in that directory and compiles them into the symbol table.
5.  **Symbol Binding**: Once the package is loaded, the internal symbol resolution mechanism can successfully find `utils` and complete binding.

---

## 3. File Architecture Example

### 3.1 Single Project Mode Layout
```text
my_project/
├── legion.json          # Required: Project metadata and dependency definitions
├── library/             # Core: Library source code directory
│   ├── _.vk             # Recommended: Namespace root directory definition
│   └── utils.vk         # Submodule (corresponding to namespace my_project::utils)
├── binary/              # Runtime: Executable instruction directory
│   ├── simple.vk        # Registered as instruction `simple`
│   └── complex/
│       └── main.vk      # Registered as instruction `complex`
├── script/              # Auxiliary: Script directory (registers instructions but does not execute installation)
│   └── setup.vk         # Registered as instruction `setup`
├── test/                # Test: Test code directory
├── vendor/              # Cache: Private dependency cache for this project
└── tests/               # Recommended: Integration test code
```

### 3.2 Workspace Mode Layout
```text
my_workspace/
├── legions.json         # Required: Workspace member management and shared dependencies
├── vendor/              # Recommended: Shared dependency cache for all workspace members
├── projects/
│   ├── core/            # Member project A
│   │   ├── legion.json
│   │   └── library/ ...
│   └── app/             # Member project B
│       ├── legion.json
│       └── library/ ...
```

---

## 4. Legion Directory Structure Specification

Legion enforces the following directory structure to ensure consistency and assigns different behaviors based on directory type:

### 4.1 Core Directory Behaviors
- **`library/`**: Stores core library code. If present, Legion will only load `.vk` or `.valkyrie` files from this directory as library members (the latter is the full suffix, usually recommending the shorthand `.vk`).
- **`binary/`**: Stores executable programs.
    - Files like `binary/simple.vk` or `binary/simple.valkyrie` are automatically registered as instructions with the same name as the file (`simple`).
    - Directories like `binary/complex/main.vk` are registered as instructions with the same name as the directory (`complex`).
    - These instructions are deployed to the system's executable path when the project is installed (`install`).
- **`script/`**: Stores auxiliary scripts.
    - Behaves similarly to `binary/`, files are registered as instructions.
    - **Important Difference**: Script instructions are for local development use only and will not be installed with the project's `install` operation.
- **`test/`**: Stores unit test and integration test code.

### 4.2 Namespace and State Sharing
All code located in `library/`, `binary/`, `script/`, and `test/` shares the **same top-level namespace** in development mode (Dev Mode).

This means:
- Code in `binary` or `script` can directly `using` symbols defined in the same project's `library` without additional configuration.
- They share the project's `dev` state environment during compilation, ensuring development tools (such as LSP) can provide consistent symbol navigation and completion experiences.

### 4.3 Other Directories
- **`vendor/`**: Stores external dependencies. Supports multi-level nesting (such as `vendor/github.com/owner/repo`).

---

## 5. Command Line Tool (CLI) Instructions

Legion provides powerful command-line tools for managing the project lifecycle.

### 5.1 Dependency Management (`add`)

Use the `add` instruction to add new external dependencies to the project.

- **Usage**: `legion add <package> [--vendor <vendor>] [--version <version>]`
- **Parameters**:
    - `<package>`: Package name (e.g., `@std/assert`).
    - `--vendor`: Specify vendor (default: `jsr`). Legion supports obtaining packages from different vendors (such as `jsr`, `npm`).
    - `--version`: Specify version constraint (e.g., `^1.0.0`). If not specified, Legion will automatically obtain the latest stable version from the vendor.
- **Behavior**: The instruction resolves the package's version information and writes it to the `[dependencies]` section of `legion.toml`.

### 5.2 Installation and Deployment (`install`)

The `install` instruction is responsible for downloading dependencies or installing the current project to the system.

#### Local Dependency Installation
Run `legion install` in the project root directory. It will download and sync all declared dependencies to the project's `vendor/` directory according to the definitions in `legion.toml` or `legions.json`.

#### Global Installation (`--global`)
Run `legion install --global` to install the current project to the global environment:
- **Behavior**: 
    1. Copies project source code and metadata to the global storage directory (`%LEGION_HOME%/packages/`).
    2. Creates execution scripts (Shims) for the project's `binary/` instructions in the global binary directory (`%LEGION_HOME%/bin/`).
    3. **Environment Variable**: After successful installation, users need to ensure `%LEGION_HOME%/bin/` has been added to the system's `PATH` environment variable to call the instruction from anywhere.

---

## 6. Custom Package Manager Integration Guide

If you wish to implement an alternative package manager for Valkyrie (e.g., integrating into an existing build system), please follow these guidelines:

### 4.1 Implement Path Mapping Interface
Your package manager must be able to answer the following questions:
-   Given a package name, where is its physical root directory?
-   Given a package directory, which files are its source files?
-   Which packages does the current workspace contain?

### 4.2 Interface with LSP State
In `ServerState`, you need to replace or extend `LegionManager`. Key integration points include:
-   **Workspace Scanning**: Actively inform LSP of all available packages when `set_workspace_root`.
-   **File Watching**: Monitor your package manager's specific configuration files and notify LSP to refresh when changes occur.
-   **Async Resolution**: Ensure your path lookup logic is efficient, preferably supporting async, to avoid blocking LSP's main response loop.

### 4.3 Naming Convention Recommendations
-   Package names should be globally unique (recommend using reverse domain names or organization prefixes).
-   Version numbers should follow the Semantic Versioning (SemVer) specification.
-   The logical entry point of the package should always be clear, avoiding circular dependencies.
