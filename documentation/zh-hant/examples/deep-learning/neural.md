# 神經網絡類型 (Neural)

神經網絡類型是 Valkyrie 中專門為機器學習設計的特殊類類型。它提供了構建、訓練和推理神經網絡的高級抽象，簡化了機器學習模型的開發過程。

## 基本神經網絡定義

### 簡單神經網絡

```valkyrie
# 基本神經網絡定義
neural LinearRegression {
    # 網絡參數
    𝐖: Tensor<f32>,
    𝐛: f32,
    
    # 構造函數
    new(input_size: usize) {
        self.𝐖 = Tensor::random([input_size])
        self.𝐛 = 0.0
    }
    
    # 前向傳播
    forward(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        𝐱.dot(self.𝐖) + self.𝐛
    }
    
    # 損失函數
    loss(self, ŷ: Tensor<f32>, 𝐲: Tensor<f32>) -> f32 {
        (ŷ - 𝐲) ^ 2.mean()
    }
}
```

### 多層神經網絡

```valkyrie
neural MultiLayerPerceptron {
    layers: [Layer],
    activation: ActivationFunction,
    
    new(layer_sizes: [usize], activation: ActivationFunction) {
        self.layers = []
        self.activation = activation
        
        loop i in 0..layer_sizes.length - 1 {
            let layer = Layer::new(layer_sizes[i], layer_sizes[i + 1])
            self.layers.push(layer)
        }
    }
    
    forward(self, mut 𝐱: Tensor<f32>) -> Tensor<f32> {
        loop layer in self.layers {
            𝐱 = layer.forward(𝐱)
            𝐱 = self.activation.apply(𝐱)
        }
        𝐱
    }
    
    # 反向傳播
    backward(mut self, ∇ℒ: Tensor<f32>) {
        let mut ∇ = ∇ℒ
        
        loop layer in self.layers.reverse() {
            ∇ = layer.backward(∇)
        }
    }
}
```

## 卷積神經網絡

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
        
        # 卷積和池化層
        for (conv, pool) in zip(self.conv_layers, self.pool_layers) {
            𝐱 = conv.forward(𝐱)
            𝐱 = pool.forward(𝐱)
        }
        
        # 展平
        𝐱 = 𝐱.flatten()
        
        # 全連接層
        loop fc in self.fc_layers {
            𝐱 = fc.forward(𝐱)
        }
        
        𝐱
    }
    
    # 特徵提取
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

## 循環神經網絡

```valkyrie
neural RecurrentNetwork {
    hidden_size: usize,
    input_size: usize,
    output_size: usize,
    
    # RNN 參數
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
        
        # 輸出
        self.𝐡.matmul(self.𝐖_ho)
    }
    
    reset_hidden(mut self) {
        self.𝐡 = Tensor::zeros([self.hidden_size])
    }
    
    # 序列處理
    forward_sequence(mut self, sequence: [Tensor<f32>]) -> [Tensor<f32>] {
        let mut outputs = []
        
        loop 𝐱 in sequence {
            let output = self.forward(𝐱)
            outputs.push(output)
        }
        
        outputs
    }
}
```

## 訓練和優化

### 訓練器

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
    
    # 單步訓練
    train_step(mut self, 𝐱: Tensor<f32>, 𝐲: Tensor<f32>) -> f32 {
        # 前向傳播
        let ŷ = self.model.forward(𝐱)
        
        # 計算損失
        let ℒ = self.loss_function.compute(ŷ, 𝐲)
        
        # 反向傳播
        let ∇ℒ = self.loss_function.gradient(ŷ, 𝐲)
        self.model.backward(∇ℒ)
        
        # 更新參數
        self.optimizer.step(self.model.parameters())
        
        ℒ
    }
    
    # 批量訓練
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
    
    # 驗證
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

### 優化器

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
            
            # 動量更新
            self.𝐯[name] = self.μ × self.𝐯[name] + param.gradient
            
            # 參數更新
            param.data -= self.η × self.𝐯[name]
            
            # 清零梯度
            param.gradient.zero_()
        }
    }
}

neural AdamOptimizer {
    η: f32,
    β₁: f32,
    β₂: f32,
    ε: f32,
    
    𝐦: {utf8: Tensor<f32>},  # 一階矩估計
    𝐯: {utf8: Tensor<f32>},  # 二階矩估計
    t: i32,  # 時間步
    
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
            
            # 更新偏置一階矩估計
            self.𝐦[name] = self.β₁ × self.𝐦[name] + (1.0 - self.β₁) × param.gradient
            
            # 更新偏置二階矩估計
            self.𝐯[name] = self.β₂ × self.𝐯[name] + (1.0 - self.β₂) × param.gradient ^ 2
            
            # 偏置校正
            let 𝐦̂ = self.𝐦[name] / (1.0 - self.β₁ ^ f32(self.t))
            let 𝐯̂ = self.𝐯[name] / (1.0 - self.β₂ ^ f32(self.t))
            
            # 參數更新
            param.data -= self.η × 𝐦̂ / (𝐯̂.sqrt() + self.ε)
            
            # 清零梯度
            param.gradient.zero_()
        }
    }
}
```

## 預訓練模型

```valkyrie
neural PretrainedModel {
    backbone: ConvolutionalNetwork,
    classifier: FullyConnectedLayer,
    
    # 加載預訓練模型
    from_pretrained(model_path: utf8) -> Self {
        let checkpoint = load_checkpoint(model_path)
        let mut model = Self::new(checkpoint.config)
        model.load_state_dict(checkpoint.state_dict)
        model
    }
    
    # 微調
    fine_tune(mut self, num_classes: usize, freeze_backbone: bool = true) {
        if freeze_backbone {
            self.backbone.freeze_parameters()
        }
        
        # 替換分類器
        let feature_size = self.backbone.get_output_size()
        self.classifier = FullyConnectedLayer::new(feature_size, num_classes)
    }
    
    forward(self, 𝐱: Tensor<f32>) -> Tensor<f32> {
        let features = self.backbone.extract_features(𝐱)
        self.classifier.forward(features)
    }
}
```

## 模型保存和加載

```valkyrie
neural ModelCheckpoint {
    # 保存模型
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
    
    # 加載模型
    load⟨N⟩(path: utf8) -> N where N: Neural {
        let checkpoint: Checkpoint = deserialize_from_file(path)
        let mut model = N::new(checkpoint.config)
        model.load_state_dict(checkpoint.state_dict)
        model
    }
}
```

## 最佳實踐

### 1. 模型設計原則

```valkyrie
# 模組化設計
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

### 2. 數據預處理

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

### 3. 模型評估

```valkyrie
neural ModelEvaluator {
    # 分類準確率
    accuracy(predictions: Tensor<f32>, targets: Tensor<i32>) -> f32 {
        let predicted_classes = predictions.argmax(dim: 1)
        let correct = (predicted_classes == targets).sum()
        f32(correct) / f32(targets.length)
    }
    
    # 混淆矩陣
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

Neural 類型為 Valkyrie 提供了強大的機器學習能力，通過高級抽象簡化了神經網絡的構建和訓練過程，同時保持了足夠的靈活性來支持各種複雜的模型架構。