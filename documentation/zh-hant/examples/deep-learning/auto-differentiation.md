# 自動微分 (Automatic Differentiation)

Valkyrie 提供了強大的自動微分系統，支持前向模式和反向模式自動微分，為深度學習和科學計算提供精確高效的梯度計算。

## 基本概念

### 可微分變量

```valkyrie
use autodiff::*

# 創建可微分變量
let x = Variable::new(2.0)
let y = Variable::new(3.0)

# 基本運算
let z = x × y + x ^ 2

# 計算梯度
let grad = z.backward()
let dx = grad.get(x)  # dz/dx
let dy = grad.get(y)  # dz/dy

print("dz/dx = {}, dz/dy = {}", dx, dy)
```

### 計算圖

```valkyrie
# 構建計算圖
let mut graph = ComputationGraph::new()

let x = graph.variable(2.0, requires_grad: true)
let y = graph.variable(3.0, requires_grad: true)

# 前向傳播
let z1 = graph.add(x, y)      # z1 = x + y
let z2 = graph.mul(z1, x)     # z2 = z1 × x = (x + y) × x
let output = graph.sin(z2)    # output = sin((x + y) × x)

# 反向傳播
graph.backward(output)

# 獲取梯度
let grad_x = x.grad()
let grad_y = y.grad()
```

## 前向模式自動微分

```valkyrie
# 前向模式 - 適合輸入維度較少的情況
class ForwardDual {
    value: f64
    derivative: f64
}

imply ForwardDual {
    micro new(value: f64, derivative: f64) -> Self {
        Self { value, derivative }
    }
    
    micro variable(value: f64) -> Self {
        Self::new(value, 1.0)  # 種子向量
    }
    
    micro constant(value: f64) -> Self {
        Self::new(value, 0.0)
    }
}

# 運算符重載
imply ForwardDual {
    infix `+`(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
            derivative: self.derivative + other.derivative
        }
    }
    
    infix `×`(self, other: Self) -> Self {
        Self {
            value: self.value × other.value,
            derivative: self.derivative × other.value + self.value × other.derivative
        }
    }
}

# 使用前向模式
let x = ForwardDual::variable(2.0)
let y = ForwardDual::constant(3.0)
let result = x × x + x × y  # f(x) = x² + 3x

print("f(2) = {}, ∂f/∂x(2) = {}", result.value, result.derivative)
```

## 反向模式自動微分

```valkyrie
# 反向模式 - 適合輸出維度較少的情況
class ReverseTape {
    operations: [Operation]
    variables: [Variable]
}

union Operation {
    Add { inputs: array<usize, 2>, output: usize }
    Mul { inputs: array<usize, 2>, output: usize }
    Sin { input: usize, output: usize }
    Exp { input: usize, output: usize }
}

imply ReverseTape {
    micro new() -> Self {
        Self {
            operations: [],
            variables: [],
        }
    }
    
    micro variable(mut self, value: f64) -> VariableId {
        let id = self.variables.length
        self.variables.push(Variable::new(value))
        VariableId(id)
    }

    micro add(mut self, a: VariableId, b: VariableId) -> VariableId {
        let sum = self.variables[a.0].value + self.variables[b.0].value
        let output = self.variable(sum)
        self.operations.push(Operation::Add {
            inputs: [a.0, b.0],
            output: output.0
        })
        output
    }

    micro backward(mut self, output: VariableId) {
        # 初始化梯度
        let mut gradients = [0.0; self.variables.length]
        gradients[output.0] = 1.0
        
        # 反向遍歷操作
        loop op in self.operations.iter().rev() {
            match op {
                Operation::Add { inputs, output } => {
                    gradients[inputs[0]] += gradients[*output]
                    gradients[inputs[1]] += gradients[*output]
                }
                Operation::Mul { inputs, output } => {
                    let [a, b] = *inputs
                    gradients[a] += gradients[*output] × self.variables[b].value
                    gradients[b] += gradients[*output] × self.variables[a].value
                }
                # ... 其他操作
            }
        }
        
        # 存儲梯度
        for (i, grad) in gradients.iter().enumerate() {
            self.variables[i].gradient = *grad
        }
    }
}
```

## 高階導數

```valkyrie
# 計算高階導數
let x = Variable::new(2.0)
let y = x ^ 4 + 3.0 × x ^ 3 + 2.0 × x ^ 2 + x + 1.0

# 一階導數
let dy_dx = y.grad(x)

# 二階導數
let d2y_dx2 = dy_dx.grad(x)

# 三階導數
let d3y_dx3 = d2y_dx2.grad(x)

print("f'(x) = {}", dy_dx.eval_at(x, 2.0))
print("f''(x) = {}", d2y_dx2.eval_at(x, 2.0))
print("f'''(x) = {}", d3y_dx3.eval_at(x, 2.0))
```

## 向量化自動微分

```valkyrie
# 向量和矩陣的自動微分
let x = VectorVariable::new([1.0, 2.0, 3.0])
let W = MatrixVariable::new([
    [0.1, 0.2, 0.3],
    [0.4, 0.5, 0.6]
])
let b = VectorVariable::new([0.1, 0.2])

# 線性變換
let y = W.matmul(x) + b

# 非線性激活
let z = y.sigmoid()

# 損失函數
let target = VectorVariable::new([0.8, 0.3])
let loss = (z - target).pow(2).sum()

# 計算梯度
loss.backward()

let grad_W = W.grad()  # 權重梯度
let grad_b = b.grad()  # 偏置梯度
let grad_x = x.grad()  # 輸入梯度
```

## 神經網絡層的自動微分

```valkyrie
# 全連接層
class LinearLayer {
    W: MatrixVariable
    b: VectorVariable
}

imply LinearLayer {
    micro new(input_size: usize, output_size: usize) -> Self {
        Self {
            W: MatrixVariable::random([output_size, input_size]),
            b: VectorVariable::zeros(output_size),
        }
    }
    
    micro forward(self, input: VectorVariable) -> VectorVariable {
        self.W.matmul(input) + self.b
    }
}

# 激活函數
trait Activation {
    micro forward(self, x: VectorVariable) -> VectorVariable
}

class ReLU;
imply ReLU: Activation {
    micro forward(self, x: VectorVariable) -> VectorVariable {
        x.max(VectorVariable::zeros(x.length))
    }
}

class Sigmoid;
imply Sigmoid: Activation {
    micro forward(self, x: VectorVariable) -> VectorVariable {
        1.0 / (1.0 + (-x).exp())
    }
}

# 多層感知機
class MLP {
    layers: [LinearLayer]
    activations: [Box<Activation>]
}

imply MLP {
    micro forward(self, mut x: VectorVariable) -> VectorVariable {
        for (layer, activation) in zip(self.layers, self.activations) {
            x = layer.forward(x)
            x = activation.forward(x)
        }
        x
    }
}
```

## 卷積層的自動微分

```valkyrie
# 卷積操作
class Conv2D {
    # [out_channels, in_channels, kernel_h, kernel_w]
    kernel: TensorVariable  
    bias: VectorVariable
    stride: array<usize, 2>
    padding: array<usize, 2>
}

imply Conv2D {
    micro forward(self, 𝐱: TensorVariable) -> TensorVariable {
        # input: [batch, in_channels, height, width]
        let 𝐲 = 𝐱.conv2d(self.kernel, self.stride, self.padding)
        𝐲 + self.bias.unsqueeze([0, 2, 3])  # 廣播偏置
    }
}

# 池化層
class MaxPool2D {
    kernel_size: array<usize, 2>
    stride: array<usize, 2>
}

imply MaxPool2D {
    micro forward(self, 𝐱: TensorVariable) -> TensorVariable {
        𝐱.max_pool2d(self.kernel_size, self.stride)
    }
}
```

## 損失函數

```valkyrie
# 均方誤差損失
micro mse_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    (predictions - targets).pow(2).mean()
}

# 交叉熵損失
micro cross_entropy_loss(logits: VectorVariable, targets: VectorVariable) -> Variable {
    let softmax = logits.softmax()
    -(targets × softmax.log()).sum()
}

# 二元交叉熵損失
micro binary_cross_entropy_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    -(targets × predictions.log() + (1.0 - targets) × (1.0 - predictions).log()).mean()
}
```

## 優化器集成

```valkyrie
# 優化器
trait Optimizer {
    micro step(mut self, parameters: [Variable])
}

# SGD優化器
class SGD {
    η: f64
    μ: f64
    𝐯: HashMap<VariableId, Tensor>
}

imply SGD: Optimizer {
    micro step(mut self, parameters: [Variable]) {
        loop param in parameters {
            if let Some(∇) = param.grad() {
                # 動量更新
                let 𝐯 = self.𝐯.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                
                𝐯 = self.μ × 𝐯 + ∇
                
                # 參數更新
                param.data_mut().sub_assign(self.η × 𝐯)
                
                # 清零梯度
                param.zero_grad()
            }
        }
    }
}

# Adam優化器
class Adam {
    η: f64
    β₁: f64
    β₂: f64
    ε: f64
    t: i32  # 時間步
    𝐦: HashMap<VariableId, Tensor>  # 一階矩估計
    𝐯: HashMap<VariableId, Tensor>  # 二階矩估計
}

imply Adam: Optimizer {
    micro step(mut self, parameters: [Variable]) {
        self.t += 1
        
        loop param in parameters {
            if let Some(∇) = param.grad() {
                let 𝐦 = self.𝐦.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                let 𝐯 = self.𝐯.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                
                # 更新偏置一階矩估計
                𝐦 = self.β₁ × 𝐦 + (1.0 - self.β₁) × ∇
                
                # 更新偏置二階矩估計
                𝐯 = self.β₂ × 𝐯 + (1.0 - self.β₂) × ∇ ^ 2
                
                # 偏置校正
                let 𝐦_hat = 𝐦 / (1.0 - self.β₁ ^ self.t)
                let 𝐯_hat = 𝐯 / (1.0 - self.β₂ ^ self.t)
                
                # 參數更新
                param.data_mut().sub_assign(
                    self.η × 𝐦_hat / (𝐯_hat.sqrt() + self.ε)
                )
                
                param.zero_grad()
            }
        }
    }
}
```

## 訓練配置與循環

```valkyrie
# 訓練配置類
class TrainingConfig {
    epochs: usize
    batch_size: usize
    η: f64
    optimizer_type: utf8
}

imply TrainingConfig {
    micro default() -> Self {
        Self {
            epochs: 10,
            batch_size: 32,
            η: 0.001,
            optimizer_type: "Adam",
        }
    }
}

# 為 MLP 實現訓練方法
imply MLP {
    micro train(mut self, 
                config: TrainingConfig, 
                train_data: [(VectorVariable, VectorVariable)],
                mut optimizer: Optimizer) {
        loop epoch in 0..config.epochs {
            let mut total_ℒ = 0.0
            
            for (𝐱, 𝐲̂) in train_data {
                # 前向傳播
                let 𝐲 = self.forward(𝐱)
                let ℒ = mse_loss(𝐲, 𝐲̂)
                
                # 反向傳播
                ℒ.backward()
                
                # 參數更新
                optimizer.step(self.parameters())
                
                total_ℒ += ℒ.value()
            }
           # 輸出訓練進度
            print("Epoch {}: Loss = {}", epoch, total_ℒ / train_data.length)
        }
    }
}
```

## 神經網絡類型集成

基於自動微分系統，Valkyrie 提供了專門的神經網絡類型，簡化深度學習模型的構建和訓練：

```valkyrie
# 神經網絡類型定義
neural LinearRegression {
    weights: TensorVariable,
    bias: Variable,
    
    new(input_size: usize) {
        self.weights = TensorVariable::random([input_size])
        self.bias = Variable::new(0.0)
    }
    
    forward(self, input: TensorVariable) -> Variable {
        input.dot(self.weights) + self.bias
    }
    
    loss(self, predicted: Variable, target: Variable) -> Variable {
        (predicted - target).pow(2).mean()
    }
}

# 多層神經網絡
neural MultiLayerPerceptron {
    layers: [LinearLayer],
    activation: ActivationFunction,
    
    new(layer_sizes: [usize], activation: ActivationFunction) {
        self.layers = []
        self.activation = activation
        
        loop i in 0..layer_sizes.length - 1 {
            let layer = LinearLayer::new(layer_sizes[i], layer_sizes[i + 1])
            self.layers.push(layer)
        }
    }
    
    forward(self, mut input: TensorVariable) -> TensorVariable {
        loop layer in self.layers {
            input = layer.forward(input)
            input = self.activation.apply(input)
        }
        input
    }
    
    # 自動微分支持的反向傳播
    backward(mut self, loss_gradient: TensorVariable) {
        # 梯度會自動通過計算圖傳播
        loss_gradient.backward()
    }
}
```

## 性能優化

### 計算圖優化

```valkyrie
# 計算圖優化
class GraphOptimizer {
    fusion_rules: [FusionRule]
}

imply GraphOptimizer {
    micro optimize(self, mut graph: ComputationGraph) {
        # 算子融合
        self.fuse_operations(graph)
        
        # 內存優化
        self.optimize_memory(graph)
        
        # 並行化
        self.parallelize(graph)
    }
    
    micro fuse_operations(self, mut graph: ComputationGraph) {
        # 融合連續的線性操作
        # 例如：MatMul + Add -> FusedLinear
        loop rule in self.fusion_rules {
            rule.apply(graph)
        }
    }
}
```

### 內存管理

```valkyrie
# 梯度檢查點
class GradientCheckpointing {
    checkpoint_layers: [usize]
}

imply GradientCheckpointing {
    micro forward_with_checkpointing(self, model: MLP, 𝐱: TensorVariable) -> TensorVariable {
        let mut activations = [𝐱]
        let mut checkpoints = HashMap::new()
        
        for (i, layer) in model.layers.iter().enumerate() {
            let 𝐲 = layer.forward(activations.last().unwrap())
            
            if self.checkpoint_layers.contains(i) {
                checkpoints.insert(i, 𝐲.detach())  # 分離計算圖
            }
            
            activations.push(𝐲)
        }
        
        activations.into_iter().last().unwrap()
    }
}
```

## 最佳實踐

### 1. 數值穩定性

```valkyrie
# 數值穩定的softmax
micro stable_softmax(logits: TensorVariable) -> TensorVariable {
    let max_logits = logits.max(dim: -1, keepdim: true)
    let shifted = logits - max_logits
    let exp_shifted = shifted.exp()
    exp_shifted / exp_shifted.sum(dim: -1, keepdim: true)
}

# 數值穩定的log-sum-exp
micro log_sum_exp(𝐱: TensorVariable) -> Variable {
    let max_𝐱 = 𝐱.max()
    max_𝐱 + (𝐱 - max_𝐱).exp().sum().log()
}
```

### 2. 梯度裁剪

```valkyrie
# 梯度範數裁剪
micro clip_grad_norm(parameters: [Variable], max_norm: f64) {
    let total_norm = parameters.iter()
        .filter_map { $.grad() }
        .map { $.norm() ^ 2 }
        .sum⟨f64⟩()
        .sqrt()
    
    if total_norm > max_norm {
        let clip_coef = max_norm / total_norm
        loop param in parameters {
            if let Some(mut ∇) = param.grad_mut() {
                ∇ ×= clip_coef
            }
        }
    }
}
```

### 3. 內存效率

```valkyrie
# 就地操作減少內存分配
micro efficient_update(mut param: TensorVariable, ∇: TensorVariable, η: f64) {
    param.sub_assign(η × ∇)  # 就地更新，避免臨時張量
}

# 梯度累積
class GradientAccumulator {
    accumulated_steps: usize
    target_steps: usize
}

imply GradientAccumulator {
    micro accumulate_and_step(mut self, ℒ: Variable, mut optimizer: Optimizer, parameters: [Variable]) {
        # 縮放損失
        let scaled_ℒ = ℒ / self.target_steps
        scaled_ℒ.backward()
        
        self.accumulated_steps += 1
        
        if self.accumulated_steps >= self.target_steps {
            optimizer.step(parameters)
            self.accumulated_steps = 0
        }
    }
}
```

Valkyrie 的自動微分系統為深度學習提供了強大而高效的梯度計算能力，支持複雜的神經網絡架構和訓練策略，同時保持了良好的性能和數值穩定性。
