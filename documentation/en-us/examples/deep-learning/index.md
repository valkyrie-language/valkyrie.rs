# Deep Learning

Valkyrie provides a modern programming experience for deep learning, focusing on neural network construction and training. This document highlights concepts and techniques specific to deep learning.

## Core Features

- **Automatic Differentiation**: Built-in autodiff system supporting forward and reverse modes
- **Neural Network Layers**: Rich set of predefined layers and custom layer support
- **Optimizers**: Multiple optimization algorithm implementations
- **Loss Functions**: Efficient implementations of common loss functions

## Tensors and Gradients

### Differentiable Tensors

```valkyrie
# Create tensors requiring gradients
let x = Tensor::random([32, 784]).requires_grad()
let w = Tensor::random([784, 128]).requires_grad()
let b = Tensor::zeros([128]).requires_grad()

# Forward propagation
let y = x.matmul(&w) + &b
let activated = y.relu()
```

### Gradient Computation

```valkyrie
# Compute loss function gradients
let loss = cross_entropy(&predictions, &targets)

# Backward propagation to compute gradients
loss.backward()

# Get parameter gradients
let grad_w = w.grad()  # Weight gradient
let grad_b = b.grad()  # Bias gradient

# Zero gradients (prepare for next iteration)
w.zero_grad()
b.zero_grad()
```

## Neural Network Layers

### Fully Connected Layer

```valkyrie
# Create a fully connected layer
let linear = Linear::new(784, 128)  # 784 input dimensions, 128 output dimensions

# Forward propagation
let output = linear.forward(&input)

# Layer with activation function
let hidden = linear.forward(&input).relu()
let output = output_layer.forward(&hidden).softmax()
```

### Convolutional Layer

```valkyrie
# 2D Convolutional layer
let conv = Conv2d::new(
    in_channels: 3,
    out_channels: 64,
    kernel_size: 3,
    stride: 1,
    padding: 1
)

# Convolution operation
let feature_maps = conv.forward(&images)  # [N, 3, H, W] -> [N, 64, H, W]

# Pooling layer
let pooled = feature_maps.max_pool2d(kernel_size: 2, stride: 2)
```
```

## Optimizers

### SGD Optimizer

```valkyrie
# Stochastic Gradient Descent
let mut optimizer = SGD::new(
    parameters: model.parameters(),
    learning_rate: 0.01,
    momentum: 0.9
)

# Optimization step
optimizer.zero_grad()
loss.backward()
optimizer.step()
```

### Adam Optimizer

```valkyrie
# Adam optimizer
let mut optimizer = Adam::new(
    parameters: model.parameters(),
    learning_rate: 0.001,
    beta1: 0.9,
    beta2: 0.999
)

# Learning rate scheduling
let scheduler = StepLR::new(&optimizer, step_size: 30, gamma: 0.1)
scheduler.step()  # Decrease learning rate every 30 epochs
```

## Loss Functions

```valkyrie
# Cross-entropy loss (classification)
let loss = cross_entropy(&logits, &targets)

# Mean squared error loss (regression)
let loss = mse_loss(&predictions, &targets)

# Binary cross-entropy loss
let loss = binary_cross_entropy(&sigmoid_output, &binary_targets)
```

## Deep Learning Models

### Convolutional Neural Network

```valkyrie
# Build a CNN model
let mut model = Sequential::new()
    .add(Conv2d::new(3, 32, 3))  # 3 input channels, 32 output channels, 3x3 kernel
    .add(ReLU::new())
    .add(MaxPool2d::new(2))       # 2x2 max pooling
    .add(Conv2d::new(32, 64, 3))
    .add(ReLU::new())
    .add(MaxPool2d::new(2))
    .add(Flatten::new())
    .add(Linear::new(64 * 7 * 7, 128))
    .add(ReLU::new())
    .add(Linear::new(128, 10))    # 10-class classification

# Forward propagation
let predictions = model.forward(&images)
```

### Recurrent Neural Network

```valkyrie
# LSTM network
let lstm = LSTM::new(
    input_size: 100,
    hidden_size: 256,
    num_layers: 2,
    dropout: 0.2
)

# Process sequence data
let (output, (hidden, cell)) = lstm.forward(&sequence, None)
let predictions = linear.forward(&output[:, -1, :])  # Use last timestep output
```
```

## Training Workflow

### Complete Training Loop

```valkyrie
# Training function
micro train_model(model: &mut Sequential, train_loader: &DataLoader, epochs: usize) {
    let mut optimizer = Adam::new(model.parameters(), 0.001)
    
    for epoch in 0..epochs {
        let mut total_loss = 0.0
        let mut num_batches = 0
        
        for (batch_x, batch_y) in train_loader {
            # Forward propagation
            let predictions = model.forward(&batch_x)
            let loss = cross_entropy(&predictions, &batch_y)
            
            # Backward propagation
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            num_batches += 1
        }
        
        let avg_loss = total_loss / num_batches as f32
        print("Epoch {}: Average Loss = {:.4}", epoch + 1, avg_loss)
    }
}
```

### Model Evaluation

```valkyrie
# Evaluation function
micro evaluate_model(model: &Sequential, test_loader: &DataLoader) -> f32 {
    model.eval()  # Set to evaluation mode
    let mut correct = 0
    let mut total = 0
    
    with_no_grad(|| {
        for (batch_x, batch_y) in test_loader {
            let predictions = model.forward(&batch_x)
            let predicted_classes = predictions.argmax(dim: 1)
            
            correct += (predicted_classes == batch_y).sum().item() as usize
            total += batch_y.size(0)
        }
    })
    
    correct as f32 / total as f32
}
```
## Document Navigation

### [Automatic Differentiation](auto-differentiation.md)
Detailed introduction to Valkyrie's automatic differentiation system, including forward mode, reverse mode, and computation graph construction.

### [Einstein Operators](einstein-operators.md)
Learn to use Einstein notation for tensor operations, including rearrange, reduce, and complex tensor transformations.

## Summary

This document showcased the core features of Valkyrie's deep learning framework:

- **Automatic Differentiation**: Automatic gradient computation supporting complex neural network training
- **Neural Network Layers**: Rich set of predefined layers, including fully connected, convolutional, recurrent layers, etc.
- **Optimizers**: Efficient implementations of SGD, Adam, and other optimization algorithms
- **Loss Functions**: Common loss functions supporting classification and regression tasks
- **Training Workflow**: Complete model training and evaluation pipelines

Valkyrie focuses on providing an intuitive and easy-to-use deep learning API, enabling developers to quickly build and train neural network models.
