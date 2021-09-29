# 高精度浮點數

Valkyrie 提供了多種高精度數值類型，用於需要極高精度的科學計算場景。

## 基本高精度類型

### Integer 和 Decimal 類型

```valkyrie
# 原生大整數支持
let big_int: Integer = 123456789012345678901234567890
let another_big: Integer = 999999999999999999999999999999999999999

# 原生高精度十進制數
let π: Decimal = 3.141592653589793238462643383279502884197
let e: Decimal = 2.718281828459045235360287471352662497757

# 金融專用 d128 類型
let price: d128 = 19.99
let interest_rate: d128 = 0.05

# 基本運算
let sum = π + e
let product = π × e
let power = π ^ e

# 字符串解析方式（可選）
let parsed_decimal = Decimal::parse("123.456789")
let parsed_integer = Integer::parse("987654321098765432109876543210")
```

### 金融計算範例

```valkyrie
# 十進制精確計算，避免二進制浮點誤差
let price: Decimal = 19.99
let tax_rate: Decimal = 0.08
let total = price × (1.0 + tax_rate)

# 使用 d128 進行金融計算
let principal: d128 = 10000.00
let interest_rate: d128 = 0.05
let years = 10

let compound_interest = principal × (1.0 + interest_rate) ^ years

# 類型限制範例
# let overflow: i128 = 12222222222222222222222222222222  # 編譯錯誤：超出i128範圍
let big_number: Integer = 12222222222222222222222222222222  # 正確：使用Integer類型
```

## 數學函數庫

### 三角函數

```valkyrie
use math::high_precision::*

let angle = π / 4.0  # π/4

# 高精度三角函數
let sin_val = sin(angle)
let cos_val = cos(angle)
let tan_val = tan(angle)
```

# 反三角函數
let asin_val = asin(sin_val)
let acos_val = acos(cos_val)
let atan_val = atan(tan_val)
```

### 指數和對數函數

```valkyrie
let x: Decimal = 2.0

# 指數函數
let exp_x = exp(x)  # e^x
let exp2_x = exp2(x)  # 2^x
let exp10_x = exp10(x)  # 10^x

# 對數函數
let ln_x = ln(x)  # 自然對數
let log2_x = log2(x)  # 以2為底
let log10_x = log10(x)  # 以10為底
```

### 特殊函數

```valkyrie
# 伽馬函數
let gamma_val = gamma(1.5: Decimal)

# 貝塞爾函數
let bessel_j0 = bessel_j0(1.0: Decimal)
let bessel_y0 = bessel_y0(1.0: Decimal)

# 橢圓積分
let elliptic_k = elliptic_k(0.5: Decimal)
let elliptic_e = elliptic_e(0.5: Decimal)
```

## 數值積分

```valkyrie
use numerical::integration::*

# 定義被積函數
let f = { $x: Decimal -> exp(-$x ^ 2) }  # e^(-x²)

# 高斯積分
let gauss_result = gauss_legendre(f, 0.0: Decimal, 1.0: Decimal, 64)

# 自適應積分
let adaptive_result = adaptive_simpson(f, 0.0: Decimal, 1.0: Decimal, 1e-15: Decimal)

# 多重積分
let double_integral = { $x: Decimal, $y: Decimal -> $x ^ 2 + $y ^ 2 }
let result_2d = integrate_2d(double_integral, 
    0.0: Decimal, 1.0: Decimal,
    0.0: Decimal, 1.0: Decimal)
```

## 微分方程求解

```valkyrie
use numerical::ode::*

# 定義微分方程 dy/dx = -y
let ode_func = { $x: Decimal, $y: Decimal -> -$y }

# 初始條件
let x0: Decimal = 0.0
let y0: Decimal = 1.0
let x_end: Decimal = 5.0

# 龍格-庫塔方法
let solution = runge_kutta_4(ode_func, x0, y0, x_end, 1000)

# 高階微分方程組
structure LorenzSystem {
    σ: Decimal,
    ρ: Decimal,
    β: Decimal,
}

imply LorenzSystem {
    micro equations(self, t: Decimal, state: [Decimal; 3]) -> [Decimal; 3] {
        let [x, y, z] = state
        [
            self.σ × (y - x),
            x × (self.ρ - z) - y,
            x × y - self.β × z
        ]
    }
}

let lorenz = LorenzSystem {
    σ: 10.0,
    ρ: 28.0,
    β: 8.0 / 3.0,
}

let initial_state: [Decimal; 3] = [1.0, 1.0, 1.0]
let lorenz_solution = solve_ode_system(lorenz.equations, 0.0: Decimal, initial_state, 20.0: Decimal, 10000)
```

## 線性代數

```valkyrie
use linalg::high_precision::*

# 高精度矩陣
let matrix = Matrix::new([
    [1.0: Decimal, 2.0: Decimal],
    [3.0: Decimal, 4.0: Decimal]
])

# 矩陣運算
let det = matrix.determinant()
let inv = matrix.inverse()
let eigenvalues = matrix.eigenvalues()

# 線性方程組求解
let A = Matrix::new([
    [2.0: Decimal, 1.0: Decimal],
    [1.0: Decimal, 3.0: Decimal]
])
let b: [Decimal] = [5.0, 6.0]
let x = A.solve(b)  # 求解 Ax = b
```

## 統計計算

```valkyrie
use statistics::high_precision::*

# 高精度統計函數
let data: [Decimal] = [1.0, 2.0, 3.0, 4.0, 5.0]

let mean = calculate_mean(data)
let variance = calculate_variance(data)
let std_dev = calculate_std_deviation(data)

# 概率分佈
let normal_dist = NormalDistribution::new(
    0.0: Decimal,  # 均值
    1.0: Decimal   # 標準差
)

let test_value: Decimal = 1.5
let pdf_value = normal_dist.pdf(test_value)
let cdf_value = normal_dist.cdf(test_value)
```

## 性能優化

### 精度控制

```valkyrie
use std::gc::GC_ALLOCATOR

# 動態精度調整
structure AdaptivePrecision {
    min_precision: u32,
    max_precision: u32,
    ε: Decimal,
}

imply AdaptivePrecision {
    micro compute_with_adaptive_precision⟨F⟩(self, f: F, x: Decimal, allocator: Allocator = GC_ALLOCATOR) -> Decimal 
    where F: Fn(Decimal, Allocator) -> Decimal {
        let mut precision = self.min_precision
        let mut prev_result: Decimal = 0.0
        
        loop {
            let x_prec = x.with_precision(precision, allocator)
            let result = f(x_prec, allocator)
            
            if precision > self.min_precision {
                let diff = (result - prev_result).abs()
                if diff < self.ε {
                    return result
                }
            }
            
            if precision >= self.max_precision {
                return result
            }
            
            prev_result = result
            precision ×= 2
        }
    }
}
```

### 並行計算

```valkyrie
use parallel::*

# 並行數值積分
micro parallel_integration(f: impl Fn(Decimal) -> Decimal + Sync, 
                       a: Decimal, b: Decimal, n_threads: usize) -> Decimal {
    let chunk_size = (b - a) / n_threads
    
    let results = (0..n_threads).into_par_iter().map {
        let start = a + $ × chunk_size
        let end = start + chunk_size
        gauss_legendre(f, start, end, 32)
    }.collect⟨[_]⟩()
    
    results.into_iter().sum()
}
```

## 最佳實踐

### 1. 精度選擇

```valkyrie
# 根據問題需求選擇合適的精度
let financial_precision = 128  # 金融計算
let scientific_precision = 256  # 科學計算
let research_precision = 1024   # 研究級計算
```

### 2. 誤差控制

```valkyrie
# 相對誤差和绝對誤差控制
structure ErrorControl {
    abs_ε: Decimal,
    rel_ε: Decimal,
}

imply ErrorControl {
    micro check_convergence(self, current: Decimal, previous: Decimal) -> bool {
        let abs_error = (current - previous).abs()
        let rel_error = abs_error / current.abs()
        
        abs_error < self.abs_ε || rel_error < self.rel_ε
    }
}
```

### 3. 內存管理

```valkyrie
# 避免不必要的高精度計算
micro optimize_computation(x: f64) -> Decimal {
    # 先用普通精度估算
    let estimate = x.sin()
    
    # 只在需要時使用高精度
    if estimate.abs() > 0.1 {
        x.sin()
    } else {
        # 在接近零的地方使用高精度
        let high_prec_x = x.with_precision(256)
        high_prec_x.sin()
    }
}
```

高精度浮點數為 Valkyrie 提供了處理要求極高精度的科學計算問題的能力，適用於金融計算、天體力學、量子計算等領域。