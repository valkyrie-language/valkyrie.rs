# Scientific Computing

Valkyrie provides powerful scientific computing capabilities, combining high-performance numerical operations with expressive syntax.

## Key Features

### High-Precision Arithmetic
- **BigInt**: Arbitrary-precision integers for calculations beyond native types
- **BigDecimal**: Precise decimal arithmetic without floating-point errors
- **Rational**: Exact rational number arithmetic

### Graph Algorithms
- Efficient graph representations
- Classic algorithms: BFS, DFS, Dijkstra, Kruskal
- Topological sorting and cycle detection

### Unicode Processing
- Full Unicode support in source code
- Mathematical symbol operators
- Internationalization support

## Example: Numerical Integration

```valkyrie
using std::math

# Simpson's rule for numerical integration
micro integrate(f: micro(f64) -> f64, a: f64, b: f64, n: i32) -> f64 {
    let h = (b - a) / n
    let mut sum = f(a) + f(b)
    
    for i in 1..n {
        let x = a + i * h
        sum += if i % 2 == 0 { 2 * f(x) } else { 4 * f(x) }
    }
    
    sum * h / 3
}

# Calculate π using integration
let pi_approx = integrate({ 4 / (1 + $ * $) }, 0, 1, 1000)
print("π ≈ ${pi_approx}")
```

## Example: Matrix Operations

```valkyrie
class Matrix⟨T⟩ {
    data: [[T]]
    rows: usize
    cols: usize
}

imply Matrix⟨f64⟩ {
    micro multiply(self, other: Matrix⟨f64⟩) -> Matrix⟨f64⟩ {
        let mut result = Matrix::zeros(self.rows, other.cols)
        
        for i in 0..self.rows {
            for j in 0..other.cols {
                for k in 0..self.cols {
                    result[i][j] += self[i][k] * other[k][j]
                }
            }
        }
        
        result
    }
    
    micro determinant(self) -> f64 {
        # LU decomposition method
        # ...
    }
    
    micro inverse(self) -> Option⟨Matrix⟨f64⟩⟩ {
        # Gauss-Jordan elimination
        # ...
    }
}
```

## Chapter Contents

- **[High-Precision Computing](./high-precision.md)**: BigInt, BigDecimal, and exact arithmetic
- **[Graph Algorithms](./graph.md)**: Graph representations and algorithms
- **[Unicode Processing](./unicode.md)**: Unicode support and mathematical symbols
