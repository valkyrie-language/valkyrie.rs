# Literals

Valkyrie supports various literal types for representing constant values in programs.

## Numeric Literals

### Integer Literals

```valkyrie
# Decimal integer
42
-17
0

# Hexadecimal integer
0xFF
0x1A2B

# Binary integer
0b1010
0b11110000

# Octal integer
0o755
0o644

# Integers with underscore separators (improves readability)
1_000_000
0xFF_FF_FF
0b1010_1010
```

### Floating-Point Literals

```valkyrie
# Standard floating-point
3.14
-2.5
0.0

# Scientific notation
1.23e4
-5.67E-3
2.0e+10

# With underscore separators
3.141_592_653
1.234_567e-8
```

### Type Suffixes

Numbers can have type suffixes:

```valkyrie
42i32
100u64
3.14f32
```

## String Literals

Valkyrie's string syntax (S-Grammar) supports interpolation, raw strings, and multiline modes.

```valkyrie
let simple = "Hello"
let raw = r"C:\path"
let interpolated = "Hello, $name or Hello, ${name}"
```

For detailed information on string syntax, please refer to [S-Grammar](./s-grammar.md).

## Character Literals

```valkyrie
# Single character
'a'
'中'
'1'

# Escape characters
'\n'
'\t'
'\\'
'\''
'\$'  # Dollar sign escape

# Unicode characters
'\u{1F600}'  # 😀
'\u{4E2D}'   # 中
```

## Boolean Literals

```valkyrie
# Boolean values
true
false
```

## Null Literal

```valkyrie
# Null value
null
```

## Array Literals

```valkyrie
# Empty array
[]

# Integer array
[1, 2, 3, 4, 5]

# String array
["apple", "banana", "cherry"]

# Mixed type array
[1, "hello", true, null]

# Nested array
[[1, 2], [3, 4], [5, 6]]

# Multiline array
[
    "first",
    "second",
    "third"
]
```

## Object Literals

```valkyrie
# Empty object
{}

# Simple object
{
    name: "Alice",
    age: 30,
    active: true
}

# Nested object
{
    user: {
        name: "Bob",
        profile: {
            email: "bob@example.com",
            phone: "123-456-7890"
        }
    },
    settings: {
        theme: "dark",
        notifications: true
    }
}

# Quoted keys
{
    "first-name": "Charlie",
    "last-name": "Brown",
    "age": 25
}
```

## Tuple Literals

```valkyrie
# Empty tuple
()

# Single-element tuple (requires comma)
(42,)

# Multi-element tuple
(1, 2, 3)
("name", 30, true)

# Nested tuple
((1, 2), (3, 4))

# Multiline tuple
(
    "first",
    "second",
    "third"
)
```

## Range Literals

```valkyrie
# Inclusive range (recommended syntax)
0..=100    # 0 to 100 (includes 100)
1..=10     # 1 to 10 (includes 10)

# Exclusive range
0..<100    # 0 to 99 (excludes 100)

# Unbounded range
..10       # Up to 10 (exclusive)
..=10      # Up to 10 (inclusive)
0..        # From 0 onwards
..         # Full range
```
