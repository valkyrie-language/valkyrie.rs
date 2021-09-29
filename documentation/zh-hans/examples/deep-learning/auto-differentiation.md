# 自动微分 (Automatic Differentiation)

Valkyrie 提供了强大的自动微分系统，支持前向模式和反向模式自动微分，为深度学习和科学计算提供精确高效的梯度计算。

## 基本概念

### 可微分变量

```valkyrie
use autodiff::*

# 创建可微分变量
let x = Variable::new(2.0)
let y = Variable::new(3.0)

# 基本运算
let z = x × y + x ^ 2

# 计算梯度
let grad = z.backward()
let dx = grad.get(x)  # dz/dx
let dy = grad.get(y)  # dz/dy

print("dz/dx = {}, dz/dy = {}", dx, dy)
```

### 计算图

```valkyrie
# 构建计算图
let mut graph = ComputationGraph::new()

let x = graph.variable(2.0, requires_grad: true)
let y = graph.variable(3.0, requires_grad: true)

# 前向传播
let z1 = graph.add(x, y)      # z1 = x + y
let z2 = graph.mul(z1, x)     # z2 = z1 × x = (x + y) × x
let output = graph.sin(z2)    # output = sin((x + y) × x)

# 反向传播
graph.backward(output)

# 获取梯度
let grad_x = x.grad()
let grad_y = y.grad()
```

## 前向模式自动微分

```valkyrie
# 前向模式 - 适合输入维度较少的情况
class ForwardDual {
    value: f64
    derivative: f64
}

imply ForwardDual {
    micro new(value: f64, derivative: f64) -> Self {
        Self { value, derivative }
    }
    
    micro variable(value: f64) -> Self {
        Self::new(value, 1.0)  # 种子向量
    }
    
    micro constant(value: f64) -> Self {
        Self::new(value, 0.0)
    }
}

# 运算符重载
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

## 反向模式自动微分

```valkyrie
# 反向模式 - 适合输出维度较少的情况
class ReverseTape {
    operations: [Operation]
    variables: [Variable]
}

unite Operation {
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
        
        # 反向遍历操作
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
        
        # 存储梯度
        loop (i, grad) in gradients.iter().enumerate() {
            self.variables[i].gradient = *grad
        }
    }
}
```

## 高阶导数

```valkyrie
# 计算高阶导数
let x = Variable::new(2.0)
let y = x ^ 4 + 3.0 × x ^ 3 + 2.0 × x ^ 2 + x + 1.0

# 一阶导数
let dy_dx = y.grad(x)

# 二阶导数
let d2y_dx2 = dy_dx.grad(x)

# 三阶导数
let d3y_dx3 = d2y_dx2.grad(x)

print("f'(x) = {}", dy_dx.eval_at(x, 2.0))
print("f''(x) = {}", d2y_dx2.eval_at(x, 2.0))
print("f'''(x) = {}", d3y_dx3.eval_at(x, 2.0))
```

## 向量化自动微分

```valkyrie
# 向量和矩阵的自动微分
let x = VectorVariable::new([1.0, 2.0, 3.0])
let W = MatrixVariable::new([
    [0.1, 0.2, 0.3],
    [0.4, 0.5, 0.6]
])
let b = VectorVariable::new([0.1, 0.2])

# 线性变换
let y = W.matmul(x) + b

# 非线性激活
let z = y.sigmoid()

# 损失函数
let target = VectorVariable::new([0.8, 0.3])
let loss = (z - target).pow(2).sum()

# 计算梯度
loss.backward()

let grad_W = W.grad()  # 权重梯度
let grad_b = b.grad()  # 偏置梯度
let grad_x = x.grad()  # 输入梯度
```

## 神经网络层的自动微分

```valkyrie
# 全连接层
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

# 激活函数
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

# 多层感知机
class MLP {
    layers: [LinearLayer]
    activations: [Box<Activation>]
}

imply MLP {
    micro forward(self, mut x: VectorVariable) -> VectorVariable {
        loop (layer, activation) in zip(self.layers, self.activations) {
            x = layer.forward(x)
            x = activation.forward(x)
        }
        x
    }
}
```

## 卷积层的自动微分

```valkyrie
# 卷积操作
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
        𝐲 + self.bias.unsqueeze([0, 2, 3])  # 广播偏置
    }
}

# 池化层
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

## 损失函数

```valkyrie
# 均方误差损失
micro mse_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    (predictions - targets).pow(2).mean()
}

# 交叉熵损失
micro cross_entropy_loss(logits: VectorVariable, targets: VectorVariable) -> Variable {
    let softmax = logits.softmax()
    -(targets × softmax.log()).sum()
}

# 二元交叉熵损失
micro binary_cross_entropy_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    -(targets × predictions.log() + (1.0 - targets) × (1.0 - predictions).log()).mean()
}
```

## 优化器集成

```valkyrie
# 优化器
trait Optimizer {
    micro step(mut self, parameters: [Variable])
}

# SGD优化器
class SGD {
    η: f64
    μ: f64
    𝐯: HashMap<VariableId, Tensor>
}

imply SGD: Optimizer {
    micro step(mut self, parameters: [Variable]) {
        loop param in parameters {
            if let Some(∇) = param.grad() {
                # 动量更新
                let 𝐯 = self.𝐯.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                
                𝐯 = self.μ × 𝐯 + ∇
                
                # 参数更新
                param.data_mut().sub_assign(self.η × 𝐯)
                
                # 清零梯度
                param.zero_grad()
            }
        }
    }
}

# Adam优化器
class Adam {
    η: f64
    β₁: f64
    β₂: f64
    ε: f64
    t: i32  # 时间步
    𝐦: HashMap<VariableId, Tensor>  # 一阶矩估计
    𝐯: HashMap<VariableId, Tensor>  # 二阶矩估计
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
                
                # 更新偏置一阶矩估计
                𝐦 = self.β₁ × 𝐦 + (1.0 - self.β₁) × ∇
                
                # 更新偏置二阶矩估计
                𝐯 = self.β₂ × 𝐯 + (1.0 - self.β₂) × ∇ ^ 2
                
                # 偏置校正
                let 𝐦_hat = 𝐦 / (1.0 - self.β₁ ^ self.t)
                let 𝐯_hat = 𝐯 / (1.0 - self.β₂ ^ self.t)
                
                # 参数更新
                param.data_mut().sub_assign(
                    self.η × 𝐦_hat / (𝐯_hat.sqrt() + self.ε)
                )
                
                param.zero_grad()
            }
        }
    }
}
```

## 训练配置与循环

```valkyrie
# 训练配置类
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

# 为 MLP 实现训练方法
imply MLP {
    micro train(mut self, 
                config: TrainingConfig, 
                train_data: [(VectorVariable, VectorVariable)],
                mut optimizer: Optimizer) {
        loop epoch in 0..config.epochs {
            let mut total_ℒ = 0.0
            
            loop (𝐱, 𝐲̂) in train_data {
                # 前向传播
                let 𝐲 = self.forward(𝐱)
                let ℒ = mse_loss(𝐲, 𝐲̂)
                
                # 反向传播
                ℒ.backward()
                
                # 参数更新
                optimizer.step(self.parameters())
                
                total_ℒ += ℒ.value()
            }
           # 输出训练进度
            print("Epoch {}: Loss = {}", epoch, total_ℒ / train_data.length)
        }
    }
}
```

## 神经网络类型集成

基于自动微分系统，Valkyrie 提供了专门的神经网络类型，简化深度学习模型的构建和训练：

```valkyrie
# 神经网络类型定义
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

# 多层神经网络
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
    
    # 自动微分支持的反向传播
    backward(mut self, loss_gradient: TensorVariable) {
        # 梯度会自动通过计算图传播
        loss_gradient.backward()
    }
}
```

## 性能优化

### 计算图优化

```valkyrie
# 计算图优化
class GraphOptimizer {
    fusion_rules: [FusionRule]
}

imply GraphOptimizer {
    micro optimize(self, mut graph: ComputationGraph) {
        # 算子融合
        self.fuse_operations(graph)
        
        # 内存优化
        self.optimize_memory(graph)
        
        # 并行化
        self.parallelize(graph)
    }
    
    micro fuse_operations(self, mut graph: ComputationGraph) {
        # 融合连续的线性操作
        # 例如：MatMul + Add -> FusedLinear
        loop rule in self.fusion_rules {
            rule.apply(graph)
        }
    }
}
```

### 内存管理

```valkyrie
# 梯度检查点
class GradientCheckpointing {
    checkpoint_layers: [usize]
}

imply GradientCheckpointing {
    micro forward_with_checkpointing(self, model: MLP, 𝐱: TensorVariable) -> TensorVariable {
        let mut activations = [𝐱]
        let mut checkpoints = HashMap::new()
        
        loop (i, layer) in model.layers.iter().enumerate() {
            let 𝐲 = layer.forward(activations.last().unwrap())
            
            if self.checkpoint_layers.contains(i) {
                checkpoints.insert(i, 𝐲.detach())  # 分离计算图
            }
            
            activations.push(𝐲)
        }
        
        activations.into_iter().last().unwrap()
    }
}
```

## 最佳实践

### 1. 数值稳定性

```valkyrie
# 数值稳定的softmax
micro stable_softmax(logits: TensorVariable) -> TensorVariable {
    let max_logits = logits.max(dim: -1, keepdim: true)
    let shifted = logits - max_logits
    let exp_shifted = shifted.exp()
    exp_shifted / exp_shifted.sum(dim: -1, keepdim: true)
}

# 数值稳定的log-sum-exp
micro log_sum_exp(𝐱: TensorVariable) -> Variable {
    let max_𝐱 = 𝐱.max()
    max_𝐱 + (𝐱 - max_𝐱).exp().sum().log()
}
```

### 2. 梯度裁剪

```valkyrie
# 梯度范数裁剪
micro clip_grad_norm(parameters: [Variable], max_norm: f64) {
    let total_norm = parameters.iter()
        .filter_map { %.grad() }
        .map { %.norm() ^ 2 }
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

### 3. 内存效率

```valkyrie
# 就地操作减少内存分配
micro efficient_update(mut param: TensorVariable, ∇: TensorVariable, η: f64) {
    param.sub_assign(η × ∇)  # 就地更新，避免临时张量
}

# 梯度累积
class GradientAccumulator {
    accumulated_steps: usize
    target_steps: usize
}

imply GradientAccumulator {
    micro accumulate_and_step(mut self, ℒ: Variable, mut optimizer: Optimizer, parameters: [Variable]) {
        # 缩放损失
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

Valkyrie 的自动微分系统为深度学习提供了强大而高效的梯度计算能力，支持复杂的神经网络架构和训练策略，同时保持了良好的性能和数值稳定性。