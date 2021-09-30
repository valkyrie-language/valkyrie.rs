# Unicode Processing

Valkyrie has first-class support for Unicode, enabling natural use of mathematical symbols and international characters in source code.

## Unicode in Source Code

### Mathematical Operators

Valkyrie allows mathematical symbols as operators, making code more readable:

```valkyrie
# Traditional operators
let sum = a + b
let product = a * b
let power = a ** b

# Unicode mathematical operators
let sum = a + b        # Same as +
let product = a × b    # Same as *
let power = a ** b     # Same as **
let union = a ∪ b      # Set union
let intersection = a ∩ b  # Set intersection
let element = x ∈ set  # Set membership
let not_element = x ∉ set  # Not in set
```

### Greek Letters in Identifiers

Use Greek letters for mathematical variables:

```valkyrie
# Physics calculations
let π = 3.14159265359
let θ = angle_in_radians
let ω = angular_velocity
let λ = wavelength
let Σ = sum_of_values

# Mathematical formulas
micro circle_area(r: f64) -> f64 {
    π × r²  # Using × and ²
}

micro kinetic_energy(m: f64, v: f64) -> f64 {
    ½ × m × v²  # Using ½ and ×
}
```

### Subscripts and Superscripts

```valkyrie
# Chemical formulas
let H₂O = water_molecule
let CO₂ = carbon_dioxide

# Mathematical notation
let x₁ = first_coordinate
let x₂ = second_coordinate
let xₙ = nth_element

# Vector notation
let v⃗ = velocity_vector
let F⃗ = force_vector
```

## String Processing

### Unicode-Aware String Operations

```valkyrie
using std::unicode

let text = "Hello, 世界! 🌍"

# Character-aware operations
let chars = text.chars()  # Iterator over Unicode characters
let len = text.char_count()  # 11, not byte count

# Normalization
let normalized = text.normalize(NormalizationForm::NFC)

# Case conversion (Unicode-aware)
let upper = text.to_uppercase()  # "HELLO, 世界! 🌍"
let lower = text.to_lowercase()  # "hello, 世界! 🌍"
```

### Unicode Properties

```valkyrie
using std::unicode::{UnicodeProperties, GeneralCategory}

micro analyze_text(text: string) {
    for char in text.chars() {
        if char.is_letter() {
            print("'{char}' is a letter")
        }
        if char.is_digit() {
            print("'{char}' is a digit")
        }
        if char.is_emoji() {
            print("'{char}' is an emoji")
        }
        
        # Get Unicode category
        let category = char.general_category()
        print("'{char}' category: {category}")
    }
}
```

## Text Segmentation

### Grapheme Clusters

Handle user-perceived characters correctly:

```valkyrie
using std::unicode::segmentation

let text = "café"  # e + combining acute accent

# Wrong: byte iteration
for byte in text.bytes() {
    # Iterates over 5 bytes, not 4 characters
}

# Right: grapheme iteration
for grapheme in text.graphemes() {
    # Iterates over 4 grapheme clusters
}

# Reverse by grapheme
let reversed = text.graphemes().rev().collect()
```

### Word Boundaries

```valkyrie
using std::unicode::segmentation

let text = "Hello, world! 你好世界"

# Split by words (Unicode-aware)
let words = text.split_words()
# ["Hello", ",", " ", "world", "!", " ", "你好", "世界"]

# Word count
let word_count = text.word_count()  # 4 words
```

## Encoding Support

### Character Encodings

```valkyrie
using std::encoding

# Convert between encodings
let utf8_bytes = text.to_utf8()
let utf16_bytes = text.to_utf16()
let latin1_bytes = text.to_latin1()

# Decode from bytes
let from_utf8 = string.from_utf8(bytes)
let from_utf16 = string.from_utf16(bytes)
```

### BOM Handling

```valkyrie
using std::encoding::Bom

micro read_file_with_bom(path: string) -> string {
    let bytes = std::fs::read(path)?
    
    # Detect and handle BOM
    let (encoding, content) = Bom::detect_and_strip(bytes)?
    
    encoding.decode(content)
}
```

## Internationalization

### Locale-Aware Formatting

```valkyrie
using std::i18n::{Locale, format_number, format_date}

let locale = Locale::from("zh-CN")

# Number formatting
let formatted_number = format_number(1234567.89, locale)
# Chinese: "1,234,567.89"
# German: "1.234.567,89"

# Date formatting
let formatted_date = format_date(Date::now(), locale)
# Chinese: "2024年1月15日"
# US: "January 15, 2024"
```

### Collation

```valkyrie
using std::i18n::Collator

let collator = Collator::new(Locale::from("zh-CN"))

# Sort strings according to locale rules
let mut words = ["苹果", "香蕉", "橙子", "葡萄"]
words.sort_by { |a, b| collator.compare(a, b) }
```

## Regular Expressions

### Unicode Regex Support

```valkyrie
using std::regex

# Match Unicode letters
let letter_pattern = regex(r"\p{L}+")  # Any Unicode letter

# Match Chinese characters
let chinese_pattern = regex(r"\p{Script=Han}+")

# Match emojis
let emoji_pattern = regex(r"\p{Emoji}+")

# Case-insensitive (Unicode-aware)
let case_insensitive = regex(r"(?i)hello")
# Matches "hello", "HELLO", "Hello", etc.
```

## Best Practices

1. **Always use character iteration**, not byte iteration
2. **Normalize text** before comparison or storage
3. **Use grapheme clusters** for user-facing operations
4. **Consider locale** for sorting and formatting
5. **Test with diverse Unicode** including emojis, combining characters, and RTL text

## Unicode Operator Reference

| Symbol | Name | Valkyrie Equivalent |
|--------|------|---------------------|
| × | Multiplication | `*` |
| ÷ | Division | `/` |
| ± | Plus-minus | `±` (literal) |
| ≠ | Not equal | `!=` |
| ≤ | Less or equal | `<=` |
| ≥ | Greater or equal | `>=` |
| ∈ | Element of | `in` |
| ∉ | Not element of | `not in` |
| ∪ | Union | `\|` (for sets) |
| ∩ | Intersection | `&` (for sets) |
| ⊂ | Subset | `<` (for sets) |
| ⊃ | Superset | `>` (for sets) |
| √ | Square root | `sqrt()` |
| ∞ | Infinity | `f64::infinity()` |
