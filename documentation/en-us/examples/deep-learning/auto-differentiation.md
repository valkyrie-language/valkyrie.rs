# Automatic Differentiation

Valkyrie provides a powerful automatic differentiation system supporting both forward and reverse mode automatic differentiation, offering precise and efficient gradient computation for deep learning and scientific computing.

## Basic Concepts

### Differentiable Variables

```valkyrie
use autodiff::*

# Create differentiable variables
let x = Variable::new(2.0)
let y = Variable::new(3.0)

# Basic operations
let z = x × y + x ^ 2

# Compute gradients
let grad = z.backward()
let dx = grad.get(x)  # dz/dx
let dy = grad.get(y)  # dz/dy

print("dz/dx = {}, dz/dy = {}", dx, dy)
```

### Computation Graph

```valkyrie
# Build computation graph
let mut graph = ComputationGraph::new()

let x = graph.variable(2.0, requires_grad: true)
let y = graph.variable(3.0, requires_grad: true)

# Forward propagation
let z1 = graph.add(x, y)      # z1 = x + y
let z2 = graph.mul(z1, x)     # z2 = z1 × x = (x + y) × x
let output = graph.sin(z2)    # output = sin((x + y) × x)

# Backward propagation
graph.backward(output)

# Get gradients
let grad_x = x.grad()
let grad_y = y.grad()
```

## Forward Mode Automatic Differentiation

```valkyrie
# Forward mode - suitable for cases with few input dimensions
class ForwardDual {
    value: f64
    derivative: f64
}

imply ForwardDual {
    micro new(value: f64, derivative: f64) -> Self {
        Self { value, derivative }
    }
    
    micro variable(value: f64) -> Self {
        Self::new(value, 1.0)  # Seed vector
    }
    
    micro constant(value: f64) -> Self {
        Self::new(value, 0.0)
    }
}

# Operator overloading
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

# Using forward mode
let x = ForwardDual::variable(2.0)
let y = ForwardDual::constant(3.0)
let result = x × x + x × y  # f(x) = x² + 3x

print("f(2) = {}, ∂f/∂x(2) = {}", result.value, result.derivative)
```

## Reverse Mode Automatic Differentiation

```valkyrie
# Reverse mode - suitable for cases with few output dimensions
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
        # Initialize gradients
        let mut gradients = [0.0; self.variables.length]
        gradients[output.0] = 1.0
        
        # Reverse traverse operations
        for op in self.operations.iter().rev() {
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
                # ... other operations
            }
        }
        
        # Store gradients
        for (i, grad) in gradients.iter().enumerate() {
            self.variables[i].gradient = *grad
        }
    }
}
```

## Higher-Order Derivatives

```valkyrie
# Compute higher-order derivatives
let x = Variable::new(2.0)
let y = x ^ 4 + 3.0 × x ^ 3 + 2.0 × x ^ 2 + x + 1.0

# First derivative
let dy_dx = y.grad(x)

# Second derivative
let d2y_dx2 = dy_dx.grad(x)

# Third derivative
let d3y_dx3 = d2y_dx2.grad(x)

print("f'(x) = {}", dy_dx.eval_at(x, 2.0))
print("f''(x) = {}", d2y_dx2.eval_at(x, 2.0))
print("f'''(x) = {}", d3y_dx3.eval_at(x, 2.0))
```

## Vectorized Automatic Differentiation

```valkyrie
# Automatic differentiation for vectors and matrices
let x = VectorVariable::new([1.0, 2.0, 3.0])
let W = MatrixVariable::new([
    [0.1, 0.2, 0.3],
    [0.4, 0.5, 0.6]
])
let b = VectorVariable::new([0.1, 0.2])

# Linear transformation
let y = W.matmul(x) + b

# Nonlinear activation
let z = y.sigmoid()

# Loss function
let target = VectorVariable::new([0.8, 0.3])
let loss = (z - target).pow(2).sum()

# Compute gradients
loss.backward()

let grad_W = W.grad()  # Weight gradient
let grad_b = b.grad()  # Bias gradient
let grad_x = x.grad()  # Input gradient
```

## Neural Network Layer Automatic Differentiation

```valkyrie
# Fully connected layer
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

# Activation functions
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

# Multi-layer perceptron
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

## Convolutional Layer Automatic Differentiation

```valkyrie
# Convolution operation
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
        𝐲 + self.bias.unsqueeze([0, 2, 3])  # Broadcast bias
    }
}

# Pooling layer
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

## Loss Functions

```valkyrie
# Mean squared error loss
micro mse_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    (predictions - targets).pow(2).mean()
}

# Cross-entropy loss
micro cross_entropy_loss(logits: VectorVariable, targets: VectorVariable) -> Variable {
    let softmax = logits.softmax()
    -(targets × softmax.log()).sum()
}

# Binary cross-entropy loss
micro binary_cross_entropy_loss(predictions: VectorVariable, targets: VectorVariable) -> Variable {
    -(targets × predictions.log() + (1.0 - targets) × (1.0 - predictions).log()).mean()
}
```

## Optimizer Integration

```valkyrie
# Optimizer
trait Optimizer {
    micro step(mut self, parameters: [Variable])
}

# SGD optimizer
class SGD {
    η: f64
    μ: f64
    𝐯: HashMap<VariableId, Tensor>
}

imply SGD: Optimizer {
    micro step(mut self, parameters: [Variable]) {
        for param in parameters {
            if let Some(∇) = param.grad() {
                # Momentum update
                let 𝐯 = self.𝐯.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                
                𝐯 = self.μ × 𝐯 + ∇
                
                # Parameter update
                param.data_mut().sub_assign(self.η × 𝐯)
                
                # Zero gradient
                param.zero_grad()
            }
        }
    }
}

# Adam optimizer
class Adam {
    η: f64
    β₁: f64
    β₂: f64
    ε: f64
    t: i32  # Time step
    𝐦: HashMap<VariableId, Tensor>  # First moment estimate
    𝐯: HashMap<VariableId, Tensor>  # Second moment estimate
}

imply Adam: Optimizer {
    micro step(mut self, parameters: [Variable]) {
        self.t += 1
        
        for param in parameters {
            if let Some(∇) = param.grad() {
                let 𝐦 = self.𝐦.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                let 𝐯 = self.𝐯.entry(param.id())
                    .or_insert_with { Tensor::zeros_like(param.data()) }
                
                # Update biased first moment estimate
                𝐦 = self.β₁ × 𝐦 + (1.0 - self.β₁) × ∇
                
                # Update biased second moment estimate
                𝐯 = self.β₂ × 𝐯 + (1.0 - self.β₂) × ∇ ^ 2
                
                # Bias correction
                let 𝐦_hat = 𝐦 / (1.0 - self.β₁ ^ self.t)
                let 𝐯_hat = 𝐯 / (1.0 - self.β₂ ^ self.t)
                
                # Parameter update
                param.data_mut().sub_assign(
                    self.η × 𝐦_hat / (𝐯_hat.sqrt() + self.ε)
                )
                
                param.zero_grad()
            }
        }
    }
}
```

## Training Configuration and Loop

```valkyrie
# Training configuration class
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

# Implement training method for MLP
imply MLP {
    micro train(mut self, 
                config: TrainingConfig, 
                train_data: [(VectorVariable, VectorVariable)],
                mut optimizer: Optimizer) {
        for epoch in 0..config.epochs {
            let mut total_ℒ = 0.0
            
            for (𝐱, 𝐲̂) in train_data {
                # Forward propagation
                let 𝐲 = self.forward(𝐱)
                let ℒ = mse_loss(𝐲, 𝐲̂)
                
                # Backward propagation
                ℒ.backward()
                
                # Parameter update
                optimizer.step(self.parameters())
                
                total_ℒ += ℒ.value()
            }
           # Output training progress
            print("Epoch {}: Loss = {}", epoch, total_ℒ / train_data.length)
        }
    }
}
```

## Neural Network Type Integration

Based on the automatic differentiation system, Valkyrie provides specialized neural network types that simplify the construction and training of deep learning models:

```valkyrie
# Neural network type definition
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

# Multi-layer neural network
neural MultiLayerPerceptron {
    layers: [LinearLayer],
    activation: ActivationFunction,
    
    new(layer_sizes: [usize], activation: ActivationFunction) {
        self.layers = []
        self.activation = activation
        
        for i in 0..layer_sizes.length - 1 {
            let layer = LinearLayer::new(layer_sizes[i], layer_sizes[i + 1])
            self.layers.push(layer)
        }
    }
    
    forward(self, mut input: TensorVariable) -> TensorVariable {
        for layer in self.layers {
            input = layer.forward(input)
            input = self.activation.apply(input)
        }
        input
    }
    
    # Automatic differentiation supported backpropagation
    backward(mut self, loss_gradient: TensorVariable) {
        # Gradients are automatically propagated through the computation graph
        loss_gradient.backward()
    }
}
```

## Performance Optimization

### Computation Graph Optimization

```valkyrie
# Computation graph optimization
class GraphOptimizer {
    fusion_rules: [FusionRule]
}

imply GraphOptimizer {
    micro optimize(self, mut graph: ComputationGraph) {
        # Operator fusion
        self.fuse_operations(graph)
        
        # Memory optimization
        self.optimize_memory(graph)
        
        # Parallelization
        self.parallelize(graph)
    }
    
    micro fuse_operations(self, mut graph: ComputationGraph) {
        # Fuse consecutive linear operations
        # e.g., MatMul + Add -> FusedLinear
        for rule in self.fusion_rules {
            rule.apply(graph)
        }
    }
}
```

### Memory Management

```valkyrie
# Gradient checkpointing
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
                checkpoints.insert(i, 𝐲.detach())  # Detach computation graph
            }
            
            activations.push(𝐲)
        }
        
        activations.into_iter().last().unwrap()
    }
}
```

## Best Practices

### 1. Numerical Stability

```valkyrie
# Numerically stable softmax
micro stable_softmax(logits: TensorVariable) -> TensorVariable {
    let max_logits = logits.max(dim: -1, keepdim: true)
    let shifted = logits - max_logits
    let exp_shifted = shifted.exp()
    exp_shifted / exp_shifted.sum(dim: -1, keepdim: true)
}

# Numerically stable log-sum-exp
micro log_sum_exp(𝐱: TensorVariable) -> Variable {
    let max_𝐱 = 𝐱.max()
    max_𝐱 + (𝐱 - max_𝐱).exp().sum().log()
}
```

### 2. Gradient Clipping

```valkyrie
# Gradient norm clipping
micro clip_grad_norm(parameters: [Variable], max_norm: f64) {
    let total_norm = parameters.iter()
        .filter_map { $.grad() }
        .map { $.norm() ^ 2 }
        .sum⟨f64⟩()
        .sqrt()
    
    if total_norm > max_norm {
        let clip_coef = max_norm / total_norm
        for param in parameters {
            if let Some(mut ∇) = param.grad_mut() {
                ∇ ×= clip_coef
            }
        }
    }
}
```

### 3. Memory Efficiency

```valkyrie
# In-place operations to reduce memory allocation
micro efficient_update(mut param: TensorVariable, ∇: TensorVariable, η: f64) {
    param.sub_assign(η × ∇)  # In-place update, avoid temporary tensors
}

# Gradient accumulation
class GradientAccumulator {
    accumulated_steps: usize
    target_steps: usize
}

imply GradientAccumulator {
    micro accumulate_and_step(mut self, ℒ: Variable, mut optimizer: Optimizer, parameters: [Variable]) {
        # Scale loss
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

Valkyrie's automatic differentiation system provides powerful and efficient gradient computation capabilities for deep learning, supporting complex neural network architectures and training strategies while maintaining good performance and numerical stability.
