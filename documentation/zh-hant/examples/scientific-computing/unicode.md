
## Unicode 希臘字母支持

Valkyrie 原生支持 Unicode 希臘字母，使數學表達式更加直觀和符合學术標準：

```valkyrie
# 基礎希臘字母變量
let α = 0.01      # alpha - 學習率
let β = 0.9       # beta - 動量參數
let γ = 0.99      # gamma - 折扣因子
let δ = 1e-6      # delta - 數值穩定性
let ε = 1e-8      # epsilon - 小量
let ζ = 0.1       # zeta - 正則化強度
let η = 0.001     # eta - 學習率調度
let θ = Matrix::random([784, 128])  # theta - 模型參數
let λ = 0.01      # lambda - 正則化係數
let μ = Array::zeros([128])         # mu - 均值
let ν = Array::ones([128])          # nu - 方差
let ξ = Random::normal(0.0, 1.0)    # xi - 隨機變量
let π = 3.14159265359               # pi - 圓周率
let ρ = 0.95      # rho - 相關係數
let σ = 0.1       # sigma - 標準差
let τ = 1.0       # tau - 時間常數
let φ = 1.618     # phi - 黃金比例
let χ = Array::random([64])         # chi - 卡方分佈
let ψ = Matrix::identity(128)       # psi - 波函數
let ω = 2.0 × π   # omega - 角頻率

# 帶下標的希臘字母
let β₁ = 0.9      # Adam優化器第一動量
let β₂ = 0.999    # Adam優化器第二動量
let σ₁ = 0.1      # 第一層標準差
let σ₂ = 0.05     # 第二層標準差
let θᵢ = Matrix::random([256, 128]) # 第i層參數
let μₜ = Array::zeros([128])        # t時刻均值
let νₜ = Array::zeros([128])        # t時刻方差

# 在神經網絡中使用希臘字母
neural NeuralNetwork {
    θ: Vector<Tensor>,  # 參數向量
    ∇θ: Vector<Tensor>, # 梯度向量
    μ: Vector<Tensor>,  # 動量項
    ν: Vector<Tensor>,  # 二階動量項
}

imply NeuralNetwork {
    # 使用希臘字母的梯度下降
    micro gradient_descent(mut self, α: f32) {
        for (θᵢ, ∇θᵢ) in self.θ.iter_mut().zip(&self.∇θ) {
            *θᵢ = θᵢ - α × ∇θᵢ
        }
    }
    
    # Adam 優化器
    micro adam_step(mut self, α: f32, β₁: f32, β₂: f32, ε: f32, t: usize) {
        loop i in 0..self.θ.length {
            let m = β₁ × self.μ[i] + (1.0 - β₁) × self.∇θ[i]
            let v = β₂ × self.ν[i] + (1.0 - β₂) × self.∇θ[i] ^ 2
            
            let m̂ = m / (1.0 - β₁ ^ t)
            let v̂ = v / (1.0 - β₂ ^ t)
            
            self.θ[i] = self.θ[i] - α × m̂ / (v̂.sqrt() + ε)
            self.μ[i] = m
            self.ν[i] = v
        }
    }
    
    # 帶正則化的損失函數
    micro regularized_loss(self, ŷ: &Tensor, y: &Tensor, λ: f32) -> f32 {
        let ℒ = self.cross_entropy_loss(ŷ, y)
        let Ω = self.l2_regularization(λ)
        ℒ + Ω
    }
}

# 數學函數使用希臘字母
micro σ(x: f64) -> f64 {  # Sigmoid激活函數
    1.0 / (1.0 + (-x).exp())
}

micro φ(x: f64) -> f64 {  # 標準正態分佈CDF
    0.5 × (1.0 + (x / 2.0_f64.sqrt()).erf())
}

micro ψ(x: &Array, θ: &Matrix) -> Array {  # 神經網絡前向傳播
    σ(x.matmul(θ))
}

# 損失函數
micro ℒ(ŷ: &Array, y: &Array) -> f64 {  # 交叉熵損失
    let n = y.length
    -((y × ŷ.log()).sum() + ((1.0 - y) × (1.0 - ŷ).log()).sum()) / n
}

# 正則化項
micro Ω(θ: &Matrix, λ: f64) -> f64 {  # L2正則化
    λ × (θ × θ).sum() / 2.0
}

# 梯度計算
micro ∇ℒ(θ: &Matrix, x: &Array, y: &Array) -> Matrix {  # 損失函數梯度
    let ŷ = ψ(x, θ)
    let δ = ŷ - y
    x.transpose().matmul(&δ)
}

# 物理常數使用希臘字母
const π = 3.14159265358979323846
const φ = 1.618033988749895  # 黃金比例
const γ = 0.5772156649015329  # 歐拉常數
const Δ = 4.669201609102990  # Feigenbaum常數

# 高斯分佈
micro gaussian(x: f64, μ: f64, σ: f64) -> f64 {
    let coefficient = 1.0 / (σ × (2.0 × π).sqrt())
    let exponent = -0.5 × ((x - μ) / σ) ^ 2
    coefficient × exponent.exp()
}

# 統計分佈參數
class BetaDistribution {
    α: f64,  # 形狀參數1
    β: f64,  # 形狀參數2
}

class GammaDistribution {
    α: f64,  # 形狀參數
    β: f64,  # 率參數
}

class DirichletDistribution {
    α: Vector<f64>,  # 濃度參數向量
}

# 概率密度函數
impl BetaDistribution {
    micro pdf(&self, x: f64) -> f64 {
        let Β = gamma_function(self.α) × gamma_function(self.β) / gamma_function(self.α + self.β)
        x ^ (self.α - 1.0) × (1.0 - x) ^ (self.β - 1.0) / Β
    }
}

# 優化算法中的希臘字母
class SGDOptimizer {
    α: f32  # 學習率
    μ: f32  # 動量係數
}

class AdamOptimizer {
    α: f32   # 學習率
    β₁: f32  # 一階動量衰減率
    β₂: f32  # 二階動量衰減率
    ε: f32   # 數值穩定性參數
}

class RMSpropOptimizer {
    α: f32  # 學習率
    ρ: f32  # 衰減率
    ε: f32  # 數值穩定性參數
}
```
