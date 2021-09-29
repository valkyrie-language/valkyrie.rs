# Valkyrie Language Quick Start

Welcome to the Valkyrie language! Valkyrie is a modern functional programming language that provides a powerful type system, flexible module system, and rich language features.

## What is Valkyrie?

Valkyrie is a multi-paradigm programming language that offers:

- 🎯 **Powerful Type System**: Supports generics, higher-kinded types, type inference, and other advanced features
- 🚀 **Modern Syntax**: Concise and intuitive syntax with support for pattern matching, closures, and other modern features
- 🔒 **Memory Safety**: Garbage collector automatically manages memory, preventing memory leaks
- ⚡ **High Performance**: Zero-cost abstractions, compile-time optimization
- 🔧 **Flexible Module System**: Namespace-based module organization

## Basic Syntax

### Variable Definition

```valkyrie
# Immutable variable
let name = "Alice"
let age = 30
let is_active = true

# Mutable variable
let mut counter = 0
let mut items = []

# Explicit type annotation
let score: i32 = 95
let price: f64 = 29.99
let message: String = "Hello"
```

### Function Definition

```valkyrie
# Basic function definition
micro greet() {
    print("Hello, World!")
}

# Function with parameters and return value
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# Multi-parameter function
micro calculate(x: f64, y: f64, operation: String) -> f64 {
    if operation == "add" {
        x + y
    } else if operation == "multiply" {
        x * y
    } else {
        0.0
    }
}
```

### Basic Data Types

```valkyrie
# Integer types
let a: i32 = 42
let b: u64 = 100

# Floating-point types
let x: f32 = 3.14
let y: f64 = 2.718281828

# Boolean type
let flag: bool = true

# Character and string
let ch: char = 'A'
let text: String = "Hello, World!"

# Array type
let numbers: [i32; 5] = [1, 2, 3, 4, 5]
let dynamic: [String] = ["a", "b", "c"]

# Tuple type
let point: (f64, f64) = (3.0, 4.0)
let mixed: (String, i32, bool) = ("test", 42, true)
```

## Control Flow

### Conditional Statements

```valkyrie
# if statement
if x > 0 {
    print("positive")
} else {
    print("non-positive")
}

# if expression
let result = if x > 0 { "positive" } else { "non-positive" }

# Multiple conditions
if score >= 90 {
    grade = "A"
} else if score >= 80 {
    grade = "B"
} else {
    grade = "F"
}
```

### Loop Statements

```valkyrie
# while loop
while counter < 10 {
    print(counter)
    counter = counter + 1
}

# for loop
for i in 0..10 {
    print(i)
}

# Iterate over array
for item in items {
    print(item)
}

# Infinite loop
loop {
    if should_break {
        break
    }
}
```

## Pattern Matching

```valkyrie
# Basic pattern matching
match value {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    case _: "other"
}

# Range matching
match score {
    case 90..=100: "A"
    case 80..=89: "B"
    case 70..=79: "C"
    case _: "F"
}

# Tuple destructuring
match point {
    case (0, 0): "Origin"
    case (x, 0): "On X-axis at ${ x }"
    case (0, y): "On Y-axis at ${ y }"
    case (x, y): "Point at (${ x }, ${ y })"
}
```

## Type Definition

### Record Type

```valkyrie
# Basic record type
type Point = {
    x: f64,
    y: f64,
}

# Generic record type
type Container<T> = {
    value: T,
    metadata: String,
}
```

### Union Type

```valkyrie
# Basic union type
union Result<T, E> {
    Fine { value: T },
    Fail { error: E }
}

# Using union type
let result: Result<i32, String> = Fine { value: 42 }
match result {
    case Fine { value }: print("Success: ${ value }")
    case Fail { error }: print("Error: ${ error }")
}
```

### Class Definition

```valkyrie
# Basic class definition
class Person {
    name: String
    age: i32
    
    new(name: String, age: i32) -> Self {
        Self { name, age }
    }
    
    greet(self) {
        print("Hello, I'm ${self.name}")
    }
    
    get_info(self) -> String {
        "${self.name} is ${self.age} years old"
    }
}

# Using class
let person = Person::new("Alice", 30)
person.greet()
let info = person.get_info()
```

## Module System

### Namespace Declaration

```valkyrie
# Declare namespace
namespace math.geometry {
    class Point {
        x: f64
        y: f64
    }
    
    micro distance(p1: Point, p2: Point) -> f64 {
        let dx = p1.x - p2.x
        let dy = p1.y - p2.y
        (dx * dx + dy * dy).sqrt()
    }
}
```

### Import System

```valkyrie
# Import entire namespace
using math.geometry.*

# Selective import
using math.geometry.{Point, distance}

# Renamed import
using math.geometry.Point as GeomPoint

# Use imported content
micro main() {
    let p1 = Point { x: 0.0, y: 0.0 }
let p2 = Point { x: 3.0, y: 4.0 }
    let dist = distance(p1, p2)
    @assert_equal(dist, 5.0)
}
```

## Literals

### Numeric Literals

```valkyrie
# Integer literals
42
0xFF        # Hexadecimal
0b1010      # Binary
0o755       # Octal
1_000_000   # With separators

# Floating-point literals
3.14
1.23e4      # Scientific notation
3.141_592_653  # With separators
```

### String Literals

```valkyrie
# Regular string
"Hello, World!"
'Single-quoted string'

# Escape sequences
"Newline: \n"
"Tab: \t"
"Unicode: \u{1F600}"  # 😀 emoji

# Raw string
r"C:\Users\Name\Documents"
r"""Multi-line raw string
Escape sequences not processed"""

# String interpolation
let name = "Alice"
let age = 30
let message = "Hello, ${name}! You are ${age} years old."
```

### Other Literals

```valkyrie
# Array literal
[1, 2, 3, 4, 5]
["a", "b", "c"]

# Object literal
{
    name: "Alice",
    age: 30,
    active: true
}

# Tuple literal
(1, 2, 3)
("name", 30, true)

# Range literal
0..=100     # Inclusive range
1..<10      # Exclusive range

# Regular expression literal
re"hello"
re"\d+"
re"[a-zA-Z]+"
```

## Closures and Higher-Order Functions

```valkyrie
# Basic closure syntax
let square = { $x * $x }
let add = { $x + $y }

# Explicit parameter types
let multiply = { $x: i32, $y: i32 -> $x * $y }

# Multi-statement closure
let complex = {
    let result = $x * 2
    result + 1
}

# Higher-order function usage
let numbers = [1, 2, 3, 4, 5]
let squares = numbers.map { $x * $x }
let evens = numbers.filter { $x % 2 == 0 }
let sum = numbers.reduce { $acc + $x }
```

## Type Functions (mezzo)

```valkyrie
# Type function definition
mezzo IsEven(z: Type) -> bool {
    # Check if type z represents an even number
    match z {
        i32 if z % 2 == 0 => true,
        _ => false
    }
}

# Type mapping
mezzo MapType<T>(input: T) -> T {
    # Map transformation on input type
    match input {
        i32 => i64,
        f32 => f64,
        _ => input
    }
}
```

## Next Steps

Now that you understand the basic syntax and features of Valkyrie, you can:

1. **Learn More**: Check out the [Language Features Guide](./features.md)
2. **Type System**: Learn about advanced features of the [Type System](../language/type-system/index.md)
3. **Pattern Matching**: Master the powerful features of [Pattern Matching](../language/pattern-match.md)
4. **Module System**: Learn about the organization of the [Module System](../language/modules.md)
5. **Metaprogramming**: Explore advanced usage of [Metaprogramming](../language/meta-programming/index.md)

Start your Valkyrie programming journey!
