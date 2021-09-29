# Valkyrie Error Handling and Diagnostic Guide

Valkyrie adopts a structured, internationalization-first error handling mechanism. Each error is not just a text message, but a data structure containing rich metadata, designed to provide an IDE-level diagnostic experience.

## 1. Design Principles

- **Structured**: Errors should include error codes, i18n keys, and parameterized data.
- **Internationalization (i18n)**: Error messages should not be hardcoded, but looked up in localization files for different languages through `key`.
- **Lightweight**: Uses `Box<ErrorKind>` pattern to ensure error type size remains minimal, optimizing performance.
- **No External Macros**: Strictly prohibit using `thiserror` or `anyhow`. All error implementations should remain explicit and easy to trace.

## 2. Core Data Structures

### `ValkyrieError`

This is the top-level container for all errors:

```rust
pub structure ValkyrieError {
    pub code: u32,                  // Unique error code, e.g., 0x001
    pub kind: Box<ValkyrieErrorKind>, // Specific error classification (contains i18n key and data)
    pub labels: Vec<ValkyrieLabel>, // Source code annotations (supports i18n)
    pub help: Option<ValkyrieHelp>, // Fix suggestions (supports i18n)
}

pub structure ValkyrieLabel {
    pub span: SourceSpan,
    pub key: &'static str, // i18n key for annotation content
    pub data: BTreeMap<String, String>, // i18n parameters for annotation content
    pub primary: bool,
}

pub structure ValkyrieHelp {
    pub key: &'static str,          // i18n key for fix suggestion
    pub data: BTreeMap<String, String>, // i18n parameters for fix suggestion
}
```

### `ValkyrieErrorKind`

Use enum to classify errors, each variant directly carries the parameterized data needed for that error. `key` is implicitly determined by the variant type:

```rust
pub enum ValkyrieErrorKind {
    UnexpectedToken { expected: Vec<String>, found: String },
    UndefinedVariable { name: String },
    TypeMismatch { expected: String, found: String },
},
    // ...
}

impl ValkyrieErrorKind {
    pub fn key(&self) -> &'static str {
        match self {
            Self::UnexpectedToken { .. } => "error.syntax.unexpected_token",
            Self::UndefinedVariable { .. } => "error.semantic.undefined_variable",
            // ...
        }
    }
}
```

## 3. Implementation Examples

### Triggering a Syntax Error

```rust
impl ValkyrieError {
    pub fn unexpected_token(span: SourceSpan, expected: Vec<String>, found: String) -> Self {
        Self {
            code: 1001,
            kind: Box::new(ValkyrieErrorKind::UnexpectedToken { expected, found }),
            labels: vec![ValkyrieLabel::primary(span, Some("unexpected.token"))],
            help: Some(ValkyrieHelp::new("check.syntax.manual")),
        }
    }
}
```

### Using Builder Pattern for Quick Construction

```rust
let error = ValkyrieError::new(1001, kind)
    .add_label(ValkyrieLabel::primary(span, Some("label.key")))
    .with_help(ValkyrieHelp::new("help.key").with_data("arg", "value"));
```

### Quick Construction for IO Errors

```rust
impl ValkyrieError {
    pub fn io_error(error: std::io::Error, path: Option<Pathbuf>, ) -> Self {
        Self::new(
            0x0001,
            ValkyrieErrorKind::IoError {
                path,
                message: error.to_string(),
            },
        )
    }
}
```

## 4. Internationalization (i18n) Pipeline

1. **Trigger Error**: The compiler constructs `ValkyrieError` when a problem is discovered.
2. **Carry Data**: The error object only carries `key` and `data` (e.g., `{ "name": "foo" }`).
3. **Render Diagnostic**: 
   - The diagnostic system loads translation files based on the current locale (e.g., `zh-CN.toml`).
   - Uses `key` to find the template: `error.type.mismatch = "Type mismatch: expected {expected}, but found {found}"`.
   - Fills values from `data` into the template.

## 5. Best Practices

- **Keep ErrorKind Pure**: Only include necessary data for further analysis or recovery.
- **Deferred Rendering**: Do not format strings when constructing errors. Formatting should only occur during final output to the user.
- **Use Error Code**: Every user-facing error should be assigned a permanent error code for users to reference documentation.
- **Prohibit `anyhow`**: The compiler needs strongly typed errors to decide whether subsequent stages can continue; `anyhow` loses this type information.
- **Prohibit `thiserror`**: We need custom `Diagnostic` implementations; manually implementing `Display` and `Error` traits provides greater flexibility.
