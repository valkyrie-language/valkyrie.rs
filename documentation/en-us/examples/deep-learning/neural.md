# Neural Network Types (Neural)

Neural network types are special class types in Valkyrie designed specifically for machine learning. They provide high-level abstractions for building, training, and inference of neural networks, simplifying the machine learning model development process.

## Basic Neural Network Definition

### Simple Neural Network

```valkyrie
# Basic neural network definition
neural LinearRegression {
    # Network parameters
    𝐖: Tensor<f32>,
    𝐛: f32,
    
    # Constructor
    new(input_size: usize) {
        self.𝐖 = Tensor::random([input_size])
        self.𝐛 = 0.0
    }
    
    # Forward propagation
    forward(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        𝐱.dot(self.𝐖) + self.𝐛
    }
    
    # Loss function
    loss(self, ŷ: Tensor<f32>, 𝐲: Tensor<f32>) -> f32 {
        (ŷ - 𝐲) ^ 2.mean()
    }
}
```

### Multi-layer Neural Network

```valkyrie
neural MultiLayerPerceptron {
    layers: [Layer],
    activation: ActivationFunction,
    
    new(layer_sizes: [usize], activation: ActivationFunction) {
        self.layers = []
        self.activation = activation
        
        for i in 0..layer_sizes.length - 1 {
            let layer = Layer::new(layer_sizes[i], layer_sizes[i + 1])
            self.layers.push(layer)
        }
    }
    
    forward(self, mut 𝐱: Tensor<f32>) -> Tensor<f32> {
        for layer in self.layers {
            𝐱 = layer.forward(𝐱)
            𝐱 = self.activation.apply(𝐱)
        }
        𝐱
    }
    
    # Backpropagation
    backward(mut self, ∇ℒ: Tensor<f32>) {
        let mut ∇ = ∇ℒ
        
        for layer in self.layers.reverse() {
            ∇ = layer.backward(∇)
        }
    }
}
```

## Convolutional Neural Network

```valkyrie
neural ConvolutionalNetwork {
    conv_layers: [ConvLayer],
    pool_layers: [PoolLayer],
    fc_layers: [FullyConnectedLayer],
    
    new(config: CNNConfig) {
        self.conv_layers = config.build_conv_layers()
        self.pool_layers = config.build_pool_layers()
        self.fc_layers = config.build_fc_layers()
    }
    
    forward(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        let mut 𝐱 = 𝐱
        
        # Convolution and pooling layers
        for (conv, pool) in zip(self.conv_layers, self.pool_layers) {
            𝐱 = conv.forward(𝐱)
            𝐱 = pool.forward(𝐱)
        }
        
        # Flatten
        𝐱 = 𝐱.flatten()
        
        # Fully connected layers
        for fc in self.fc_layers {
            𝐱 = fc.forward(𝐱)
        }
        
        𝐱
    }
    
    # Feature extraction
    extract_features(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        let mut 𝐱 = 𝐱
        
        for (conv, pool) in zip(self.conv_layers, self.pool_layers) {
            𝐱 = conv.forward(𝐱)
            𝐱 = pool.forward(𝐱)
        }
        
        𝐱.flatten()
    }
}
```

## Recurrent Neural Network

```valkyrie
neural RecurrentNetwork {
    hidden_size: usize,
    input_size: usize,
    output_size: usize,
    
    # RNN parameters
    𝐖_ih: Tensor<f32>,  # input to hidden
    𝐖_hh: Tensor<f32>,  # hidden to hidden
    𝐖_ho: Tensor<f32>,  # hidden to output
    
    𝐡: Tensor<f32>,     # hidden state
    
    new(input_size: usize, hidden_size: usize, output_size: usize) {
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.output_size = output_size
        
        self.𝐖_ih = Tensor::xavier_uniform([input_size, hidden_size])
        self.𝐖_hh = Tensor::xavier_uniform([hidden_size, hidden_size])
        self.𝐖_ho = Tensor::xavier_uniform([hidden_size, output_size])
        
        self.reset_hidden()
    }
    
    forward(mut self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        # h_t = tanh(W_ih * x_t + W_hh * h_{t-1})
        self.𝐡 = tanh(
            𝐱.matmul(self.𝐖_ih) + self.𝐡.matmul(self.𝐖_hh)
        )
        
        # Output
        self.𝐡.matmul(self.𝐖_ho)
    }
    
    reset_hidden(mut self) {
        self.𝐡 = Tensor::zeros([self.hidden_size])
    }
    
    # Sequence processing
    forward_sequence(mut self, sequence: [Tensor<f32>]) -> [Tensor<f32>] {
        let mut outputs = []
        
        for 𝐱 in sequence {
            let output = self.forward(𝐱)
            outputs.push(output)
        }
        
        outputs
    }
}
```

## Training and Optimization

### Trainer

```valkyrie
neural Trainer⟨N⟩ where N: Neural {
    model: N,
    optimizer: Optimizer,
    loss_function: LossFunction,
    
    new(model: N, optimizer: Optimizer, loss_function: LossFunction) {
        self.model = model
        self.optimizer = optimizer
        self.loss_function = loss_function
    }
    
    # Single step training
    train_step(mut self, 𝐱: Tensor<f32>, 𝐲: Tensor<f32>) -> f32 {
        # Forward propagation
        let ŷ = self.model.forward(𝐱)
        
        # Compute loss
        let ℒ = self.loss_function.compute(ŷ, 𝐲)
        
        # Backpropagation
        let ∇ℒ = self.loss_function.gradient(ŷ, 𝐲)
        self.model.backward(∇ℒ)
        
        # Update parameters
        self.optimizer.step(self.model.parameters())
        
        ℒ
    }
    
    # Batch training
    train_epoch(mut self, dataloader: DataLoader) -> f32 {
        let mut total_ℒ = 0.0
        let mut batch_count = 0
        
        for (𝐱, 𝐲) in dataloader {
            let ℒ = self.train_step(𝐱, 𝐲)
            total_ℒ += ℒ
            batch_count += 1
        }
        
        total_ℒ / f32(batch_count)
    }
    
    # Validation
    validate(self, dataloader: DataLoader) -> f32 {
        let mut total_ℒ = 0.0
        let mut batch_count = 0
        
        for (𝐱, 𝐲) in dataloader {
            let ŷ = self.model.forward(𝐱)
            let ℒ = self.loss_function.compute(ŷ, 𝐲)
            total_ℒ += ℒ
            batch_count += 1
        }
        
        total_ℒ / f32(batch_count)
    }
}
```

### Optimizers

```valkyrie
neural SGDOptimizer {
    η: f32,
    μ: f32,
    𝐯: {utf8: Tensor<f32>},
    
    new(η: f32, μ: f32 = 0.0) {
        self.η = η
        self.μ = μ
        self.𝐯 = {}
    }
    
    step(mut self, parameters: {utf8: Parameter}) {
        for (name, param) in parameters {
            if !self.𝐯.contains_key(name) {
                self.𝐯[name] = Tensor::zeros_like(param.gradient)
            }
            
            # Momentum update
            self.𝐯[name] = self.μ × self.𝐯[name] + param.gradient
            
            # Parameter update
            param.data -= self.η × self.𝐯[name]
            
            # Zero gradient
            param.gradient.zero_()
        }
    }
}

neural AdamOptimizer {
    η: f32,
    β₁: f32,
    β₂: f32,
    ε: f32,
    
    𝐦: {utf8: Tensor<f32>},  # First moment estimate
    𝐯: {utf8: Tensor<f32>},  # Second moment estimate
    t: i32,  # Time step
    
    new(η: f32 = 0.001, β₁: f32 = 0.9, β₂: f32 = 0.999, ε: f32 = 1e-8) {
        self.η = η
        self.β₁ = β₁
        self.β₂ = β₂
        self.ε = ε
        self.𝐦 = {}
        self.𝐯 = {}
        self.t = 0
    }
    
    step(mut self, parameters: {utf8: Parameter}) {
        self.t += 1
        
        for (name, param) in parameters {
            if !self.𝐦.contains_key(name) {
                self.𝐦[name] = Tensor::zeros_like(param.gradient)
                self.𝐯[name] = Tensor::zeros_like(param.gradient)
            }
            
            # Update biased first moment estimate
            self.𝐦[name] = self.β₁ × self.𝐦[name] + (1.0 - self.β₁) × param.gradient
            
            # Update biased second moment estimate
            self.𝐯[name] = self.β₂ × self.𝐯[name] + (1.0 - self.β₂) × param.gradient ^ 2
            
            # Bias correction
            let 𝐦̂ = self.𝐦[name] / (1.0 - self.β₁ ^ f32(self.t))
            let 𝐯̂ = self.𝐯[name] / (1.0 - self.β₂ ^ f32(self.t))
            
            # Parameter update
            param.data -= self.η × 𝐦̂ / (𝐯̂.sqrt() + self.ε)
            
            # Zero gradient
            param.gradient.zero_()
        }
    }
}
```

## Pretrained Models

```valkyrie
neural PretrainedModel {
    backbone: ConvolutionalNetwork,
    classifier: FullyConnectedLayer,
    
    # Load pretrained model
    from_pretrained(model_path: utf8) -> Self {
        let checkpoint = load_checkpoint(model_path)
        let mut model = Self::new(checkpoint.config)
        model.load_state_dict(checkpoint.state_dict)
        model
    }
    
    # Fine-tuning
    fine_tune(mut self, num_classes: usize, freeze_backbone: bool = true) {
        if freeze_backbone {
            self.backbone.freeze_parameters()
        }
        
        # Replace classifier
        let feature_size = self.backbone.get_output_size()
        self.classifier = FullyConnectedLayer::new(feature_size, num_classes)
    }
    
    forward(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        let features = self.backbone.extract_features(𝐱)
        self.classifier.forward(features)
    }
}
```

## Model Saving and Loading

```valkyrie
neural ModelCheckpoint {
    # Save model
    save⟨N⟩(model: N, path: utf8) where N: Neural {
        let state_dict = model.state_dict()
        let config = model.get_config()
        
        let checkpoint = Checkpoint {
            state_dict,
            config,
            timestamp: now(),
        }
        
        serialize_to_file(checkpoint, path)
    }
    
    # Load model
    load⟨N⟩(path: utf8) -> N where N: Neural {
        let checkpoint: Checkpoint = deserialize_from_file(path)
        let mut model = N::new(checkpoint.config)
        model.load_state_dict(checkpoint.state_dict)
        model
    }
}
```

## Best Practices

### 1. Model Design Principles

```valkyrie
# Modular design
neural ResNetBlock {
    conv1: ConvLayer,
    conv2: ConvLayer,
    shortcut: Option<ConvLayer>,
    
    new(in_channels: usize, out_channels: usize, stride: usize = 1) {
        self.conv1 = ConvLayer::new(in_channels, out_channels, 3, stride, 1)
        self.conv2 = ConvLayer::new(out_channels, out_channels, 3, 1, 1)
        
        if stride != 1 || in_channels != out_channels {
            self.shortcut = Some(ConvLayer::new(in_channels, out_channels, 1, stride, 0))
        } else {
            self.shortcut = None
        }
    }
    
    forward(self, input: Tensor<f32>) -> Tensor<f32> {
        let mut out = self.conv1.forward(input)
        out = relu(out)
        out = self.conv2.forward(out)
        
        let shortcut = if let Some(sc) = self.shortcut {
            sc.forward(input)
        } else {
            input
        }
        
        relu(out + shortcut)
    }
}
```

### 2. Data Preprocessing

```valkyrie
neural DataPreprocessor {
    mean: Tensor<f32>,
    std: Tensor<f32>,
    
    new(mean: [f32], std: [f32]) {
        self.mean = Tensor::from(mean)
        self.std = Tensor::from(std)
    }
    
    normalize(self, input: Tensor<f32>) -> Tensor<f32> {
        (input - self.mean) / self.std
    }
    
    denormalize(self, input: Tensor<f32>) -> Tensor<f32> {
        input × self.std + self.mean
    }
}
```

### 3. Model Evaluation

```valkyrie
neural ModelEvaluator {
    # Classification accuracy
    accuracy(predictions: Tensor<f32>, targets: Tensor<i32>) -> f32 {
        let predicted_classes = predictions.argmax(dim: 1)
        let correct = (predicted_classes == targets).sum()
        f32(correct) / f32(targets.length)
    }
    
    # Confusion matrix
    confusion_matrix(predictions: Tensor<f32>, targets: Tensor<i32>, num_classes: usize) -> Tensor<i32> {
        let predicted_classes = predictions.argmax(dim: 1)
        let mut matrix = Tensor::zeros([num_classes, num_classes])
        
        for (pred, target) in zip(predicted_classes, targets) {
            matrix[usize(target)][usize(pred)] += 1
        }
        
        matrix
    }
}
```

Neural types provide Valkyrie with powerful machine learning capabilities, simplifying the construction and training of neural networks through high-level abstractions while maintaining sufficient flexibility to support various complex model architectures.
