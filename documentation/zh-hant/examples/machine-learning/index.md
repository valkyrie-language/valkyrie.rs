# 機器學習

Valkyrie 為機器學習提供了基於 Array 和 ArrayND 的高效數值計算支持，专注于實用的機器學習算法和數據處理。

## 相關範例

- [網絡爬虫](web-crawler.md) - 用於數據收集的高性能異步爬虫框架

## 核心數組類型

- `Array1D` - 一維數組，用於向量和特徵處理
- `ArrayND` - 多維數組，用於矩陣運算和批量數據處理
- 內置异构計算支持，可在 CPU/GPU 間無缝切换

## 數據處理

### 基本數據操作

```valkyrie
# 創建和加載數據
let features = ArrayND::from_csv("data/features.csv")  # 特徵矩陣
let labels = Array1D::from_csv("data/labels.csv")     # 標籤向量

# 數據預處理
let normalized = features.normalize()  # 標準化
let centered = features - features.mean(axis=0)  # 中心化

# 數據分割
let (X_train, X_test, y_train, y_test) = train_test_split(
    features, labels, test_size=0.2, random_state=42
)
```

### 特徵工程

```valkyrie
# 特徵縮放
let scaled_features = features.standardize()  # 標準化 (z-score)
let normalized_features = features.normalize(min=0.0, max=1.0)  # 歸一化

# 特徵選擇
let selected = features.select_by_variance(threshold=0.01)
let top_features = features.select_k_best(k=10, target=labels)

# 特徵變換
let polynomial = features.polynomial_features(degree=2)
let log_transformed = features.log_transform()
```

## 機器學習算法

### 線性模型

```valkyrie
# 線性回歸 - 使用矩陣運算
let 𝐗 = X_train.add_bias_column()  # 添加偏置列
let 𝐰 = (𝐗ᵀ · 𝐗)⁻¹ · 𝐗ᵀ · y_train  # 最小二乘解
let predictions = X_test.add_bias_column() · 𝐰

# 岭回歸 - 帶正則化
let λ = 0.1
let 𝐈 = ArrayND::eye(𝐗.shape()[1])
let 𝐰_ridge = (𝐗ᵀ · 𝐗 + λ * 𝐈)⁻¹ · 𝐗ᵀ · y_train

# 邏輯回歸 - sigmoid激活
let σ = |z: ArrayND| 1.0 / (1.0 + (-z).exp())  # sigmoid函數
let logits = X_test · 𝐰
let probabilities = σ(logits)
```

### 聚類算法

```valkyrie
# K-Means 聚類 - 使用數組操作
let k = 3
let centroids = ArrayND::random([k, X_train.shape()[1]])  # 隨機初始化质心

loop iteration in 0..100 {
    # 計算距離矩陣
    let distances = X_train.cdist(centroids)  # [n_samples, k]
    let assignments = distances.argmin(axis=1)  # 最近质心索引
    
    # 更新质心
    loop cluster in 0..k {
        let mask = assignments.eq(cluster)
        let cluster_points = X_train.masked_select(mask)
        if cluster_points.shape()[0] > 0 {
            centroids.row_mut(cluster).copy_from(cluster_points.mean(axis=0))
        }
    }
}
```

### K-Means 聚類

```valkyrie
# K-Means 聚類 - 使用數組操作
let k = 3
let centroids = ArrayND::random([k, X_train.shape()[1]])  # 隨機初始化质心

loop iteration in 0..100 {
    # 計算距離矩陣
    let distances = X_train.cdist(centroids)  # [n_samples, k]
    let assignments = distances.argmin(axis=1)  # 最近质心索引
    
    # 更新质心
    loop cluster in 0..k {
        let mask = assignments.eq(cluster)
        let cluster_points = X_train.masked_select(mask)
        if cluster_points.shape()[0] > 0 {
            centroids.row_mut(cluster).copy_from(cluster_points.mean(axis=0))
        }
    }
}
```

## 降維算法

```valkyrie
# 主成分分析 (PCA) - 使用矩陣分解
let 𝐗_centered = X_train - μ  # 中心化，μ = X_train.mean(axis=0)
let Σ = 𝐗_centeredᵀ · 𝐗_centered / (X_train.shape()[0] - 1)  # 协方差矩陣
let (λ, 𝐕) = Σ.eig()  # 特徵分解：λ為特徵值，𝐕為特徵向量

# 選擇前k個主成分
let k = 2
let 𝐕_k = 𝐕.slice([.., 0..k])  # 前k個特徵向量
let 𝐗_pca = 𝐗_centered · 𝐕_k  # 投影到主成分空間
```

## 模型評估

```valkyrie
# 基本評估指標 - 使用數組計算
let correct = y_true.eq(y_pred).sum()  # 正確預測數量
let accuracy = correct.to_f64() / y_true.length as f64

# 回歸指標
let ε = y_true - y_pred  # 殘差
let MSE = (ε² ).mean()   # 均方誤差
let MAE = |ε|.mean()     # 平均绝對誤差
let RMSE = √MSE          # 均方根誤差
```

## 數組操作範例

```valkyrie
# 數據分割 - 使用數組索引
let n_samples = X.shape()[0]
let train_size = (n_samples as f64 * 0.8) as usize
let indices = ArrayND::arange(n_samples).shuffle()  # 隨機打乱索引

let train_indices = indices.slice([0..train_size])
let test_indices = indices.slice([train_size..])

let X_train = X.index_select(train_indices, axis=0)
let X_test = X.index_select(test_indices, axis=0)
let y_train = y.index_select(train_indices, axis=0)
let y_test = y.index_select(test_indices, axis=0)
```
        
        # 當緩衝區满時進行批量更新
## 總結

本文檔展示了如何使用 Valkyrie 的 Array 和 ArrayND 類型進行機器學習任務。重點关注數組操作的实际使用，包括：

- 數據預處理和特徵工程
- 基本機器學習算法的數組實現
- 聚類和降維的矩陣運算
- 模型評估的數組計算
- 數據分割和索引操作

通過這些範例，您可以了解如何利用 Valkyrie 的內置數組功能來構建機器學習應用。
for (name, (mean, std)) in results {
    print("{}: {:.3} (+/- {:.3})", name, mean, std * 2)
}
```

### 3. 特徵工程自動化

```valkyrie
# 自動特徵工程
let feature_engineer = AutoFeatureEngineering::new()
    .polynomial_features(degree: 2)
    .interaction_features(true)
    .log_transform(columns: vec!["income", "age"])
    .binning(column: "age", bins: 5)

let engineered_features = feature_engineer.fit_transform(X)
```

Valkyrie 的機器學習特性提供了完整的工具鏈，從數據預處理到模型部署，支持传统機器學習和現代深度學習方法，為數據科學家和機器學習工程师提供了強大而靈活的開發環境。