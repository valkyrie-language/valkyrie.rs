
## Object-Oriented Programming (Object-Oriented Programming)

Valkyrie supports class-based object-oriented programming, providing features such as class definitions, constructors, methods, and inheritance.

### Special Class Types (Special Class Types)

- [Neural Network Type (Neural)](../examples/deep-learning/neural.md) - Special class type for machine learning
- [Widget Component Type (Widget)](./widget.md) - Special class type for UI development

### Field Definitions (Field Definitions)

```valkyrie
# Basic field definitions
name: utf8
age: i32
is_active: bool = true  # Default value

# Access control
public username: utf8
private password: utf8
protected internal_id: i64

# Read-only fields
readonly created_at: DateTime
```

### Class Definitions (Class Definitions)

Use the `class` keyword to define structured data types.

```valkyrie
class Person {
    name: utf8
    age: i32
    
    micro constructor(self, name: utf8, age: i32) {
        self.name = name
        self.age = age
    }

    micro greet(self) {
        print("Hello, I'm ${self.name}")
    }
}
```

### Method Definitions (Method Definitions)

```valkyrie
imply Person {
    # Instance method
    micro say_hello(self) {
        print("Hello, I'm ${self.name}")
    }

    # Mutable method
    micro set_age(mut self, new_age: i32) {
        self.age = new_age
    }

    # Static method
    micro static create_anonymous() -> Person {
        Person { name: "Anonymous", age: 0 }
    }

    # Method with return value
    micro get_info(self) -> utf8 {
        "${self.name} is ${self.age} years old"
    }
}
```

### Inheritance (Inheritance)

Classes can inherit from one or more classes by adding parentheses after the class name.

```valkyrie
class Student(Person) {
    student_id: utf8
}
```

### Full Object Lifecycle (Full Object Lifecycle)

In Valkyrie, an object's lifecycle consists of four key stages, ensuring the integrity of resource allocation, initialization, cleanup, and return.

| Stage | Responsibility | Trait / Method | Caller |
| :--- | :--- | :--- | :--- |
| **1. Allocate** | Memory Acquisition | `Allocator::allocate` | Container (Box/GC) |
| **2. Initialize** | State Preparation | `micro constructor` | Compiler/Constructor |
| **3. Finalize** | State Cleanup | `Finalize::finalize` | Container (Box/GC) |
| **4. Delocate** | Memory Return | `Allocator::delocate` | Container (Box/GC) |

#### Finalizers (Finalizers)

`Finalize` is a core trait used to define the logic for cleaning up an object before it is destroyed.

```valkyrie
trait Finalize {
    # Finalizer: performs release of non-memory resources
    micro finalize(mut self)
}
```

Example:

```valkyrie
class FileWrapper {
    handle: i32
    
    micro constructor(mut self, path: utf8) {
        # Stage 2: Initialize
        self.handle = open_file(path)
    }
}

# Stage 3: Finalize
imply FileWrapper: Finalize {
    micro finalize(mut self) {
        close_file(self.handle)
    }
}
```

#### Automated Management (Automated Management)

In most cases, you don't need to manually call these methods:

- **Deterministic Release**: For value types on the stack (`structure`), the compiler automatically inserts the call chain: `finalize` -> `delocate` at the end of the scope.
- **GC Integration**: For managed objects (`class`), the Nyar GC triggers these stages sequentially when reclaiming objects.
- **No Lifecycle Annotations**: This four-stage model allows Valkyrie to manage resources flexibly and safely without explicit lifecycle annotations.

## Flags (Flags)

### Basic Flags (Basic Flags)

```valkyrie
# Simple flags
flags FilePermissions {
    READ = 1,
    WRITE = 2,
    EXECUTE = 4
}

# Using flags
let perms = FilePermissions::READ | FilePermissions::WRITE
if perms.contains(FilePermissions::READ) {
    print("Readable")
}

# Complex flags
flags WindowStyle {
    RESIZABLE = 0x01,
    MINIMIZABLE = 0x02,
    MAXIMIZABLE = 0x04,
    CLOSABLE = 0x08,
    TITLEBAR = 0x10,
    BORDER = 0x20,
    
    # Combined flags
    DEFAULT = RESIZABLE | MINIMIZABLE | MAXIMIZABLE | CLOSABLE | TITLEBAR | BORDER,
    DIALOG = CLOSABLE | TITLEBAR | BORDER
}
```

### Flag Operations (Flag Operations)

```valkyrie
flags Permissions {
    READ = 1,
    write = 2,
    execute = 4,
    
    # Methods
    micro has_read(self) -> bool {
        self.contains(Permissions::read)
    }
    
    micro add_write(mut self) {
        self |= Permissions::write
    }
    
    micro remove_execute(mut self) {
        self &= !Permissions::execute
    }
}
```

## Traits (Traits)

### Basic Traits (Basic Traits)

```valkyrie
# Simple trait
trait Display {
    micro to_string(self) -> utf8
}

# Trait with default implementation
trait Debug {
    micro debug(self) -> utf8
    
    # Default implementation
    micro print_debug(self) {
        print(self.debug())
    }
}

# Generic trait
trait Iterator<T> {
    micro next(mut self) -> T?
    
    # Default methods
    micro collect(mut self) -> [T] {
        let mut result = []
        while let item = self.next()? {
            result.push(item)
        }
        result
    }
    
    micro map<U>(self, f: micro(T) -> U) -> MapIterator<T, U> {
        MapIterator::new(self, f)
    }
}
```

### Trait Implementation (Trait Implementation)

```valkyrie
# Implementing a trait for a type
imply Person: Display {
    micro to_string(self) -> utf8 {
        "${self.name} (${self.age} years old)"
    }
}

imply Person: Debug {
    micro debug(self) -> utf8 {
        "Person { name: \"${self.name}\", age: ${self.age} }"
    }
}

# Conditional implementation
imply<T> T?: Display where T: Display {
    micro to_string(self) -> utf8 {
        match self {
            value? => "Some(${value.to_string()})",
            _ => "None"
        }
    }
}
```

### Trait Constraints (Trait Constraints)

```valkyrie
# Trait constraints in functions
micro print_items<T>(items: [T]) where T: Display {
    for item in items {
        print(item.to_string())
    }
}

# Multiple constraints
micro process<T>(value: T) -> utf8 
where 
    T: Display + Debug + Clone 
{
    let cloned = value.clone()
    "Display: ${value.to_string()}, Debug: ${cloned.debug()}"
}

# Associated types
trait Collect<T> {
    type Output
    
    micro collect(self) -> Self::Output
}
```

## Type Aliases (Type Aliases)

```valkyrie
# Simple type aliases
type UserId = i64
type UserName = utf8
type Coordinates = (f64, f64)

# Generic type aliases
type Result<T> = Result<T, utf8>
type HashMap<K, V> = std::collections::HashMap<K, V>

# Function type aliases
type Handler = micro(Request) -> Response
type Predicate<T> = micro(T) -> bool
```

## Constants (Constants)

```valkyrie
# Basic constants
const PI: f64 = 3.14159265359
const MAX_USERS: i32 = 1000
const APP_NAME: utf8 = "MyApp"

# Complex constants
const DEFAULT_CONFIG: Config = Config {
    timeout: 30,
    retries: 3,
    debug: false
}

# Computed constants
const BUFFER_SIZE: usize = 1024 * 1024  # 1MB
const HALF_PI: f64 = PI / 2.0
```

## Modules (Modules)

```valkyrie
# Module declaration
mod utils {
    public micro helper_function() {
        # Implementation
    }
    
    public class UtilityClass {
        # Implementation
    }
}

# Using modules
using utils::helper_function
using utils::UtilityClass

# Re-exporting
public using utils::*
```

## Generics (Generics)

```valkyrie
# Generic function
micro swap<T>(mut a: T, mut b: T) {
    let temp = a
    a = b
    b = temp
}

# Generic class
class Container<T> {
    value: T
    
    new(value: T) {
        Self { value }
    }
    
    get(self) -> T {
        self.value
    }
    
    set(mut self, new_value: T) {
        self.value = new_value
    }
}

# Constrained generics
class SortedList<T> where T: Ord {
    items: [T]
    
    insert(mut self, item: T) {
        # Maintain sorted insertion
        let pos = self.items.binary_search(item).default { $e }
        self.items.insert(pos, item)
    }
}
```


## Attributes and Decorators (Attributes and Decorators)

```valkyrie
# Attribute decorators
@derive(Debug, Clone, PartialEq)
class Point {
    x: f64
    y: f64
}

@test
micro test_addition() {
    @assert_equal(2 + 2, 4)
}

@deprecated("Use new_function instead")
micro old_function() {
    # Deprecated function
}

@inline
micro fast_calculation(x: i32) -> i32 {
    x * x + 2 * x + 1
}
```
