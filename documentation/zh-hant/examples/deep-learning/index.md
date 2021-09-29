# 深度學習

Valkyrie 為深度學習提供了現代化的編程體驗，專注於神經網絡的構建和訓練。本文檔重點介紹深度學習特有的概念和技術。

## 核心特性

- **自動微分**: 內置自動微分系統，支持前向和反向模式
- **神經網絡層**: 豐富的預定義層和自定義層支持
- **優化器**: 多種優化算法實現
- **損失函數**: 常用損失函數的高效實現

## 張量與梯度

### 可微分張量

```valkyrie
# 創建需要梯度的張量
let x = Tensor::random([32, 784]).requires_grad()
let w = Tensor::random([784, 128]).requires_grad()
let b = Tensor::zeros([128]).requires_grad()

# 前向傳播
let y = x.matmul(&w) + &b
let activated = y.relu()
```

### 梯度計算

```valkyrie
# 計算損失函數的梯度
let loss = cross_entropy(&predictions, &targets)

# 反向傳播計算梯度
loss.backward()

# 獲取參數梯度
let grad_w = w.grad()  # 權重梯度
let grad_b = b.grad()  # 偏置梯度

# 梯度清零（為下次迭代準備）
w.zero_grad()
b.zero_grad()
```

## 神經網絡層

### 全連接層

```valkyrie
# 創建全連接層
let linear = Linear::new(784, 128)  # 輸入784維，輸出128維

# 前向傳播
let output = linear.forward(&input)

# 帶激活函數的層
let hidden = linear.forward(&input).relu()
let output = output_layer.forward(&hidden).softmax()
```

### 卷積層

```valkyrie
# 2D卷積層
let conv = Conv2d::new(
    in_channels: 3,
    out_channels: 64,
    kernel_size: 3,
    stride: 1,
    padding: 1
)

# 卷積操作
let feature_maps = conv.forward(&images)  # [N, 3, H, W] -> [N, 64, H, W]

# 池化層
let pooled = feature_maps.max_pool2d(kernel_size: 2, stride: 2)
```
```

## 優化器

### SGD 優化器

```valkyrie
# 隨機梯度下降
let mut optimizer = SGD::new(
    parameters: model.parameters(),
    learning_rate: 0.01,
    momentum: 0.9
)

# 優化步驟
optimizer.zero_grad()
loss.backward()
optimizer.step()
```

### Adam 優化器

```valkyrie
# Adam 優化器
let mut optimizer = Adam::new(
    parameters: model.parameters(),
    learning_rate: 0.001,
    beta1: 0.9,
    beta2: 0.999
)

# 學習率調度
let scheduler = StepLR::new(&optimizer, step_size: 30, gamma: 0.1)
scheduler.step()  # 每30個epoch降低學習率
```

## 損失函數

```valkyrie
# 交叉熵損失（分類）
let loss = cross_entropy(&logits, &targets)

# 均方誤差損失（回歸）
let loss = mse_loss(&predictions, &targets)

# 二元交叉熵損失
let loss = binary_cross_entropy(&sigmoid_output, &binary_targets)
```

## 深度學習模型

### 卷積神經網絡

```valkyrie
# 構建CNN模型
let mut model = Sequential::new()
    .add(Conv2d::new(3, 32, 3))  # 輸入通道3，輸出通道32，卷積核3x3
    .add(ReLU::new())
    .add(MaxPool2d::new(2))       # 2x2最大池化
    .add(Conv2d::new(32, 64, 3))
    .add(ReLU::new())
    .add(MaxPool2d::new(2))
    .add(Flatten::new())
    .add(Linear::new(64 * 7 * 7, 128))
    .add(ReLU::new())
    .add(Linear::new(128, 10))    # 10類分類

# 前向傳播
let predictions = model.forward(&images)
```

### 循環神經網絡

```valkyrie
# LSTM網絡
let lstm = LSTM::new(
    input_size: 100,
    hidden_size: 256,
    num_layers: 2,
    dropout: 0.2
)

# 處理序列數據
let (output, (hidden, cell)) = lstm.forward(&sequence, None)
let predictions = linear.forward(&output[:, -1, :])  # 使用最後時刻的輸出
```
```

## 訓練流程

### 完整訓練循環

```valkyrie
# 訓練函數
micro train_model(model: &mut Sequential, train_loader: &DataLoader, epochs: usize) {
    let mut optimizer = Adam::new(model.parameters(), 0.001)
    
    loop epoch in 0..epochs {
        let mut total_loss = 0.0
        let mut num_batches = 0
        
        for (batch_x, batch_y) in train_loader {
            # 前向傳播
            let predictions = model.forward(&batch_x)
            let loss = cross_entropy(&predictions, &batch_y)
            
            # 反向傳播
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

### 模型評估

```valkyrie
# 評估函數
micro evaluate_model(model: &Sequential, test_loader: &DataLoader) -> f32 {
    model.eval()  # 設置為評估模式
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
## 文檔導航

### [自動微分](auto-differentiation.md)
詳細介紹 Valkyrie 的自動微分系統，包括前向模式、反向模式和計算圖構建。

### [Einstein 操作符](einstein-operators.md)
學習使用 Einstein 記號進行張量操作，包括重排、約簡和複雜的張量變換。

## 總結

本文檔展示了 Valkyrie 深度學習框架的核心功能：

- **自動微分**: 自動計算梯度，支持複雜的神經網絡訓練
- **神經網絡層**: 豐富的預定義層，包括全連接層、卷積層、循環層等
- **優化器**: SGD、Adam 等優化算法的高效實現
- **損失函數**: 常用損失函數，支持分類和回歸任務
- **訓練流程**: 完整的模型訓練和評估流程

Valkyrie 專注於提供直觀易用的深度學習 API，讓開發者能夠快速構建和訓練神經網絡模型。
