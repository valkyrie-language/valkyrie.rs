# 异构計算

在 Valkyrie 中，异构計算通過統一的數組抽象實現跨设备的高性能計算，专注于传统機器學習算法的加速。

## 數組類型體系

- `Array` 是一段內存中的數據
- `Array<T, N>` 是 `array<T, N>` 的語法糖，表示固定大小數組
- `ArrayND` 是异构計算、機器學習的基礎多維數組類型
- `Array1D` 是 `ArrayND` 的 type alias，專門用於一維數組操作

ArrayND 可以選擇不同的设备 (device) 和佈局 (layout)，系統会自動進行優化。

## 基本用法

### 創建數組

```valkyrie
# 創建不同維度的數組
let 𝐯 = Array1D::zeros(1000)           # 向量
let 𝐌 = ArrayND::zeros([1000, 1000])   # 矩陣
let 𝐓 = ArrayND::zeros([32, 3, 224, 224])  # 四維張量

# 從數據創建
let 𝐝 = [1.0, 2.0, 3.0, 4.0]
let 𝐚 = Array1D::from(𝐝)
let 𝐫 = 𝐚.reshape([2, 2])  # 重塑為 2x2 矩陣
```

### 设备選擇

```valkyrie
# 在不同设备上創建數組
let 𝐚_cpu = ArrayND::zeros([1024, 1024]).on_device(Device::CPU)
let 𝐚_gpu = ArrayND::zeros([1024, 1024]).on_device(Device::GPU)

# 设备間轉換
let 𝐠 = 𝐚_cpu.to_device(Device::GPU)
let 𝐜 = 𝐚_gpu.to_cpu()
```

## 佈局優化

### 自動佈局選擇

```valkyrie
# 系統会根據操作自動選擇最优佈局
let 𝐀 = ArrayND::random([1024, 1024])
let 𝐁 = ArrayND::random([1024, 1024])

# 矩陣乘法会自動選擇最优的內存佈局
let 𝐂 = 𝐀.matmul(𝐁)  # 自動優化

# 手动指定佈局（通常不需要）
let 𝐀_row = 𝐀.with_layout(Layout::RowMajor)
let 𝐀_col = 𝐀.with_layout(Layout::ColMajor)
```

### 異步數據传输

```valkyrie
# 異步设备間传输
async micro process_large_data() {
    let 𝐃 = ArrayND::load("dataset.bin")
    let 𝐆 = 𝐃.to_device_async(Device::GPU).await
    
    # GPU計算
    let 𝐑 = 𝐆.matmul(𝐆.transpose())
    
    # 传回CPU保存
    let 𝐂 = 𝐑.to_cpu_async().await
    𝐂.save("result.bin")
}
```

## 常用操作

### 數學運算

```valkyrie
# 基本運算（自動選擇最优设备和佈局）
let 𝐀 = ArrayND::random([1000, 1000])
let 𝐁 = ArrayND::random([1000, 1000])

# 矩陣運算
let sum = 𝐀 + 𝐁
let product = 𝐀 ⊙ 𝐁  # 逐元素乘法（Hadamard積）
let matmul = 𝐀 · 𝐁     # 矩陣乘法
let transpose = 𝐀ᵀ

# 統計運算
let μ = 𝐀.mean()      # 均值
let Σ = 𝐀.sum()       # 求和
let max_val = max(𝐀)  # 最大值
let σ = 𝐀.std()       # 標準差
```

## 機器學習應用

### 線性回歸

```valkyrie
# 線性回歸模型
class LinearRegression {
    weights: ArrayND,
    bias: f32,
}

impl LinearRegression {
    micro new(n_features: usize) -> Self {
        Self {
            weights: ArrayND::zeros([n_features]),
            bias: 0.0,
        }
    }
    
    micro fit(mut self, 𝐗: ArrayND, 𝐲: ArrayND, α: f32, epochs: usize) {
        let n = 𝐗.shape()[0] as f32  # 樣本數量
        
        loop _ in 0..epochs {
            # 預測
            let ŷ = 𝐗 · self.weights + self.bias
            
            # 計算梯度
            let ε = ŷ - 𝐲  # 誤差
            let ∇w = 𝐗ᵀ · ε / n  # 權重梯度
            let ∇b = ε.mean()      # 偏置梯度
            
            # 更新參數（梯度下降）
            self.weights = self.weights - α * ∇w
            self.bias -= α * ∇b
        }
    }
    
    micro predict(self, 𝐗: ArrayND) -> ArrayND {
        𝐗 · self.weights + self.bias
    }
}
```

### 支持向量機 (SVM)

```valkyrie
# SVM 分類器
class SVM {
    weights: ArrayND,
    bias: f32,
    C: f32,  # 正則化參數
}

impl SVM {
    micro new(n_features: usize, C: f32) -> Self {
        Self {
            weights: ArrayND::zeros([n_features]),
            bias: 0.0,
            C,
        }
    }
    
    micro fit(mut self, 𝐗: ArrayND, 𝐲: ArrayND, α: f32, epochs: usize) {
        loop _ in 0..epochs {
            loop i in 0..𝐗.shape()[0] {
                let 𝐱ᵢ = 𝐗.row(i)
                let yᵢ = 𝐲[i]
                
                let decision = 𝐱ᵢ · self.weights + self.bias
                
                if yᵢ * decision < 1.0 {
                    # 支持向量，更新參數
                    self.weights = self.weights + α * (yᵢ * 𝐱ᵢ - 2.0 * self.C * self.weights)
                    self.bias += α * yᵢ
                } else {
                    # 正確分類，只應用正則化
                    self.weights = self.weights - α * 2.0 * self.C * self.weights
                }
            }
        }
    }
    
    micro predict(self, 𝐗: ArrayND) -> ArrayND {
        sign(𝐗 · self.weights + self.bias)
    }
}
```

## 多设备並行

### K-Means 聚類並行化

```valkyrie
# 並行 K-Means 聚類
class ParallelKMeans {
    centroids: ArrayND,
    k: usize,
    devices: [Device],
}

impl ParallelKMeans {
    micro fit(mut self, data: ArrayND, max_iters: usize) {
        let n_devices = self.devices.length
        let chunk_size = data.shape()[0] / n_devices
        
        loop _ in 0..max_iters {
            # 並行計算距離和分配
            let assignments: [ArrayND] = data.chunks(chunk_size)
                .zip(self.devices)
                .par_iter()
                .map { $1, $2 ->
                    let chunk_gpu = $1.to_device($2)
                    let centroids_gpu = self.centroids.to_device($2)
                    
                    # 計算距離矩陣
                    let distances = chunk_gpu.cdist(centroids_gpu)
                    distances.argmin(1)  # 最近质心索引
                }
                .collect()
            
            # 更新质心
            self.update_centroids(data, assignments)
        }
    }
    
    micro update_centroids(mut self, data: ArrayND, assignments: [ArrayND]) {
        loop k in 0..self.k {
            let mask = assignments.iter()
                .map { $.eq(k) }
                .reduce { $1.concat($2) }
                .unwrap()
            
            let cluster_points = data.masked_select(mask)
            if cluster_points.shape()[0] > 0 {
                self.centroids.row_mut(k).copy_from(cluster_points.mean(0))
            }
        }
    }
}
```

### 隨機森林並行化

```valkyrie
# 並行隨機森林
structure ParallelRandomForest {
    trees: [DecisionTree],
    n_trees: usize,
    allocator: Allocator,
}

impl ParallelRandomForest {
    micro new(n_trees: usize, allocator: Allocator) -> Self {
        Self { trees: [], n_trees, allocator }
    }
    micro fit(mut self, X: ArrayND, y: ArrayND) {
        # 並行訓練決策樹
        self.trees = (0..self.n_trees)
            .into_par_iter()
            .map {
                # 自助采样
                let (X_sample, y_sample) = bootstrap_sample(X, y)
                
                # 訓練單棵樹
                let mut tree = DecisionTree::new()
                tree.fit(X_sample, y_sample)
                tree
            }
            .collect()
    }
    
    micro predict(self, X: ArrayND) -> ArrayND {
        # 並行預測
        let predictions: [ArrayND] = self.trees
            .par_iter()
            .map { $.predict(X) }
            .collect()
        
        # 投票聚合
        majority_vote(predictions)
    }
}
```

## 內存優化策略

### 數據分塊處理

```valkyrie
# 大數據集分塊處理
structure ChunkedProcessor {
    chunk_size: usize,
    memory_limit: usize,
    allocator: Allocator,
}

impl ChunkedProcessor {
    micro new(chunk_size: usize, memory_limit: usize, allocator: Allocator) -> Self {
        Self { chunk_size, memory_limit, allocator }
    }
    micro process_large_dataset(self, data: ArrayND, algorithm: Algorithm) -> ArrayND {
        let total_size = data.shape()[0]
        let mut results = Vec::new()
        
        loop start in (0..total_size).step_by(self.chunk_size) {
            let end = (start + self.chunk_size).min(total_size)
            let chunk = data.slice([start..end, ..])
            
            # 處理單個數據塊
            let chunk_result = algorithm.process(chunk)
            results.push(chunk_result)
            
            # 檢查內存使用
            if self.get_memory_usage() > self.memory_limit {
                self.gc_collect()  # 強制垃圾回收
            }
        }
        
        # 合並結果
        ArrayND::concat(results, 0)
    }
    
    micro get_memory_usage(self) -> usize {
        # 獲取當前內存使用量
        std::mem::size_of_val(self) + self.estimate_array_memory()
    }
}
```

### 內存使用檢查

```valkyrie
# 簡單的內存使用监控
micro check_memory_usage() {
    let cpu_arrays = ArrayND::get_cpu_memory_usage()
    let gpu_arrays = ArrayND::get_gpu_memory_usage()
    
    print("CPU 內存使用: {:.2} MB", cpu_arrays as f64 / 1024.0 / 1024.0)
    print("GPU 內存使用: {:.2} MB", gpu_arrays as f64 / 1024.0 / 1024.0)
    
    if gpu_arrays > GPU_LIMIT {
        print("GPU 內存不足，建議使用 CPU 或減少批次大小")
    }
}

#### 2. 性能基准測試 (Benchmarking)

```valkyrie
use std.time.Instant

micro benchmark(name: string, f: micro()) {
    let start = Instant::now()
    f()
    let duration = Instant::now() - start
    print("{} 耗時: {:.2}ms", name, duration.as_millis())
}

micro main() {
    let size = (1024, 1024)
    let a = Array::random(size)
    let b = Array::random(size)
    
    # 測試 CPU 性能
    let cpu_time = benchmark("CPU Matrix Mult") {
        compute(Backend::CPU) { a * b }
    }
    
    # 測試 GPU 性能
    let gpu_time = benchmark("GPU Matrix Mult") {
        compute(Backend::GPU) { a * b }
    }
    
    print("GPU 加速比: {:.2}x", cpu_time.as_millis() as f64 / gpu_time.as_millis() as f64)
}
```
