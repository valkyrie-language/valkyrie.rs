# High-Precision Computing

Valkyrie provides comprehensive support for high-precision numerical computing through arbitrary-precision arithmetic and specialized numeric types.

## Arbitrary-Precision Integers

### BigInt Type

```valkyrie
using std::bigint::BigInt

# Create BigInt from various sources
let a = BigInt.from(12345)
let b = BigInt.from("123456789012345678901234567890")
let c = BigInt.from_hex("FFFFFFFFFFFFFFFF")

# Basic operations
let sum = a + b
let product = a * b
let power = a.pow(100)

# Comparison
if a < b {
    print("a is smaller")
}

# Conversion
let as_string = b.to_string()
let as_hex = b.to_hex()
```

### Factorial Example

```valkyrie
micro factorial(n: i32) -> BigInt {
    let mut result = BigInt.from(1)
    for i in 2..=n {
        result *= BigInt.from(i)
    }
    result
}

# Calculate 100!
let fact_100 = factorial(100)
print("100! = {fact_100}")
```

## Arbitrary-Precision Decimals

### BigDecimal Type

```valkyrie
using std::decimal::{BigDecimal, RoundingMode}

# Create BigDecimal with specified precision
let pi = BigDecimal.from("3.14159265358979323846")
let e = BigDecimal.from("2.71828182845904523536")

# Arithmetic with precision control
let sum = pi + e
let product = pi * e

# Division with rounding
let ratio = pi / e
    .with_precision(50)
    .with_rounding(RoundingMode::HalfUp)

# Mathematical functions
let sqrt_pi = pi.sqrt()
let pi_squared = pi.pow(2)
let ln_pi = pi.ln()
let exp_pi = pi.exp()
```

### Financial Calculations

```valkyrie
micro compound_interest(
    principal: BigDecimal,
    rate: BigDecimal,
    years: i32,
    compounds_per_year: i32
) -> BigDecimal {
    let n = BigDecimal.from(compounds_per_year)
    let r = rate / BigDecimal.from(100)
    
    principal * (1 + r / n).pow(years * compounds_per_year)
}

let initial = BigDecimal.from("10000.00")
let rate = BigDecimal.from("5.5")
let final_amount = compound_interest(initial, rate, 10, 12)
print("Final amount: ${final_amount}")
```

## Fixed-Point Arithmetic

### FixedPoint Type

```valkyrie
using std::fixedpoint::FixedPoint

# 64-bit fixed-point with 32 decimal places
type Money = FixedPoint⟨64, 32⟩

let price: Money = FixedPoint.from("19.99")
let quantity: Money = FixedPoint.from("3")
let total = price * quantity

# No floating-point errors!
print("Total: {total}")  # Exactly 59.97
```

## Rational Numbers

### Rational Type

```valkyrie
using std::rational::Rational

# Exact rational arithmetic
let a = Rational.new(1, 3)  # 1/3
let b = Rational.new(2, 3)  # 2/3

let sum = a + b  # Exactly 1
let product = a * b  # Exactly 2/9

# Automatic reduction
let c = Rational.new(4, 8)  # Automatically becomes 1/2

# Conversion
let as_float = a.to_f64()  # 0.3333...
let as_decimal = a.to_decimal(10)  # "0.3333333333"
```

## Complex Numbers

### Complex Type

```valkyrie
using std::complex::Complex

let z1 = Complex.new(3, 4)   # 3 + 4i
let z2 = Complex.new(1, -2)  # 1 - 2i

# Basic operations
let sum = z1 + z2      # 4 + 2i
let product = z1 * z2  # 11 - 2i

# Complex functions
let magnitude = z1.abs()      # 5.0
let phase = z1.arg()          # atan2(4, 3)
let conjugate = z1.conj()     # 3 - 4i
let sqrt = z1.sqrt()          # Square root
let exp = z1.exp()            # e^(3+4i)
let ln = z1.ln()              # Natural logarithm

# Euler's formula verification
let e_i_pi = Complex::I * Complex::PI
let result = e_i_pi.exp()  # Should be -1
```

## Numerical Precision Control

### Precision Context

```valkyrie
using std::precision::{PrecisionContext, Precision}

# Set global precision
PrecisionContext.set_default(Precision::bits(256))

# Scoped precision
PrecisionContext.with_precision(Precision::decimal(100)) {
    let precise_pi = compute_pi()
    print("π with 100 decimal places: {precise_pi}")
}
```

### Error Analysis

```valkyrie
using std::error_analysis::{Interval, ErrorBounds}

# Interval arithmetic for error tracking
let a = Interval.new(1.0, 0.001)  # 1.0 ± 0.001
let b = Interval.new(2.0, 0.002)  # 2.0 ± 0.002

let sum = a + b  # 3.0 ± 0.003
let product = a * b  # 2.0 ± error bounds computed

print("Result: {product.value} ± {product.error}")
```

## Performance Optimization

### Memoization

```valkyrie
let fib_cache = Memoize::new()

micro fib(n: i32) -> BigInt {
    if n <= 1 { return BigInt.from(n) }
    
    fib_cache.get_or_insert(n) {
        fib(n - 1) + fib(n - 2)
    }
}
```

### Parallel Computation

```valkyrie
using std::parallel

micro parallel_matrix_multiply(a: [[BigDecimal]], b: [[BigDecimal]]) -> [[BigDecimal]] {
    parallel::range(0..a.length).map { i |
        (0..b[0].length).map { j |
            (0..a[0].length).fold(BigDecimal::zero()) { sum, k |
                sum + a[i][k] * b[k][j]
            }
        }.collect()
    }.collect()
}
```

## Best Practices

1. **Choose the Right Type**: Use `BigInt` for integers, `BigDecimal` for decimals, `Rational` for exact fractions
2. **Precision Planning**: Determine required precision before computation
3. **Error Tracking**: Use interval arithmetic for critical calculations
4. **Performance**: Cache frequently used high-precision constants
5. **Testing**: Verify results with known mathematical identities
