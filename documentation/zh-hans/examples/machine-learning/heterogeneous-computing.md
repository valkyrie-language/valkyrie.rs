# 异构计算

在 Valkyrie 中，异构计算通过统一的数组抽象实现跨设备的高性能计算，专注于传统机器学习算法的加速。

## 数组类型体系

- `Array` 是一段内存中的数据
- `Array<T, N>` 是 `array<T, N>` 的语法糖，表示固定大小数组
- `ArrayND` 是异构计算、机器学习的基础多维数组类型
- `Array1D` 是 `ArrayND` 的 type alias，专门用于一维数组操作

ArrayND 可以选择不同的设备 (device) 和布局 (layout)，系统会自动进行优化。

## 基本用法

### 创建数组

```valkyrie
# 创建不同维度的数组
let 𝐯 = Array1D::zeros(1000)           # 向量
let 𝐌 = ArrayND::zeros([1000, 1000])   # 矩阵
let 𝐓 = ArrayND::zeros([32, 3, 224, 224])  # 四维张量

# 从数据创建
let 𝐝 = [1.0, 2.0, 3.0, 4.0]
let 𝐚 = Array1D::from(𝐝)
let 𝐫 = 𝐚.reshape([2, 2])  # 重塑为 2x2 矩阵
```

### 设备选择

```valkyrie
# 在不同设备上创建数组
let 𝐚_cpu = ArrayND::zeros([1024, 1024]).on_device(Device::CPU)
let 𝐚_gpu = ArrayND::zeros([1024, 1024]).on_device(Device::GPU)

# 设备间转换
let 𝐠 = 𝐚_cpu.to_device(Device::GPU)
let 𝐜 = 𝐚_gpu.to_cpu()
```

## 布局优化

### 自动布局选择

```valkyrie
# 系统会根据操作自动选择最优布局
let 𝐀 = ArrayND::random([1024, 1024])
let 𝐁 = ArrayND::random([1024, 1024])

# 矩阵乘法会自动选择最优的内存布局
let 𝐂 = 𝐀.matmul(𝐁)  # 自动优化

# 手动指定布局（通常不需要）
let 𝐀_row = 𝐀.with_layout(Layout::RowMajor)
let 𝐀_col = 𝐀.with_layout(Layout::ColMajor)
```

### 异步数据传输

```valkyrie
# 异步设备间传输
async micro process_large_data() {
    let 𝐃 = ArrayND::load("dataset.bin")
    let 𝐆 = 𝐃.to_device_async(Device::GPU).await
    
    # GPU计算
    let 𝐑 = 𝐆.matmul(𝐆.transpose())
    
    # 传回CPU保存
    let 𝐂 = 𝐑.to_cpu_async().await
    𝐂.save("result.bin")
}
```

## 常用操作

### 数学运算

```valkyrie
# 基本运算（自动选择最优设备和布局）
let 𝐀 = ArrayND::random([1000, 1000])
let 𝐁 = ArrayND::random([1000, 1000])

# 矩阵运算
let sum = 𝐀 + 𝐁
let product = 𝐀 ⊙ 𝐁  # 逐元素乘法（Hadamard积）
let matmul = 𝐀 · 𝐁     # 矩阵乘法
let transpose = 𝐀ᵀ

# 统计运算
let μ = 𝐀.mean()      # 均值
let Σ = 𝐀.sum()       # 求和
let max_val = max(𝐀)  # 最大值
let σ = 𝐀.std()       # 标准差
```

## 机器学习应用

### 线性回归

```valkyrie
# 线性回归模型
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
        let n = 𝐗.shape()[0] as f32  # 样本数量
        
        loop _ in 0..epochs {
            # 预测
            let ŷ = 𝐗 · self.weights + self.bias
            
            # 计算梯度
            let ε = ŷ - 𝐲  # 误差
            let ∇w = 𝐗ᵀ · ε / n  # 权重梯度
            let ∇b = ε.mean()      # 偏置梯度
            
            # 更新参数（梯度下降）
            self.weights = self.weights - α * ∇w
            self.bias -= α * ∇b
        }
    }
    
    micro predict(self, 𝐗: ArrayND) -> ArrayND {
        𝐗 · self.weights + self.bias
    }
}
```

### 支持向量机 (SVM)

```valkyrie
# SVM 分类器
class SVM {
    weights: ArrayND,
    bias: f32,
    C: f32,  # 正则化参数
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
                    # 支持向量，更新参数
                    self.weights = self.weights + α * (yᵢ * 𝐱ᵢ - 2.0 * self.C * self.weights)
                    self.bias += α * yᵢ
                } else {
                    # 正确分类，只应用正则化
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

## 多设备并行

### K-Means 聚类并行化

```valkyrie
# 并行 K-Means 聚类
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
            # 并行计算距离和分配
            let assignments: [ArrayND] = data.chunks(chunk_size)
                .zip(self.devices)
                .par_iter()
                .map { %1, %2 ->
                    let chunk_gpu = %1.to_device(%2)
                    let centroids_gpu = self.centroids.to_device(%2)
                    
                    # 计算距离矩阵
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
                .map { %.eq(k) }
                .reduce { %1.concat(%2) }
                .unwrap()
            
            let cluster_points = data.masked_select(mask)
            if cluster_points.shape()[0] > 0 {
                self.centroids.row_mut(k).copy_from(cluster_points.mean(0))
            }
        }
    }
}
```

### 随机森林并行化

```valkyrie
# 并行随机森林
structure ParallelRandomForest {
    trees: [DecisionTree],
    n_trees: usize,
    allocator: Allocator,
}

impl ParallelRandomForest {
    micro new(n_trees: usize, allocator: Allocator) -> Self {
        Self { trees: [], n_trees: n_trees, allocator: allocator }
    }
    micro fit(mut self, X: ArrayND, y: ArrayND) {
        # 并行训练决策树
        self.trees = (0..self.n_trees)
            .into_par_iter()
            .map {
                # 自助采样
                let (X_sample, y_sample) = bootstrap_sample(X, y)
                
                # 训练单棵树
                let mut tree = DecisionTree::new()
                tree.fit(X_sample, y_sample)
                tree
            }
            .collect()
    }
    
    micro predict(self, X: ArrayND) -> ArrayND {
        # 并行预测
        let predictions: [ArrayND] = self.trees
            .par_iter()
            .map { %.predict(X) }
            .collect()
        
        # 投票聚合
        majority_vote(predictions)
    }
}
```

## 内存优化策略

### 数据分块处理

```valkyrie
# 大数据集分块处理
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
            
            # 处理单个数据块
            let chunk_result = algorithm.process(chunk)
            results.push(chunk_result)
            
            # 检查内存使用
            if self.get_memory_usage() > self.memory_limit {
                self.gc_collect()  # 强制垃圾回收
            }
        }
        
        # 合并结果
        ArrayND::concat(results, 0)
    }
    
    micro get_memory_usage(self) -> usize {
        # 获取当前内存使用量
        std::mem::size_of_val(self) + self.estimate_array_memory()
    }
}
```

### 内存使用检查

```valkyrie
# 简单的内存使用监控
micro check_memory_usage() {
    let cpu_arrays = ArrayND::get_cpu_memory_usage()
    let gpu_arrays = ArrayND::get_gpu_memory_usage()
    
    print("CPU 内存使用: {:.2} MB", cpu_arrays as f64 / 1024.0 / 1024.0)
    print("GPU 内存使用: {:.2} MB", gpu_arrays as f64 / 1024.0 / 1024.0)
    
    if gpu_arrays > GPU_LIMIT {
        print("GPU 内存不足，建议使用 CPU 或减少批次大小")
    }
}

#### 2. 性能基准测试 (Benchmarking)

```valkyrie
use std.time.Instant

micro benchmark(name: string, f: micro()) {
    let start = Instant::now()
    f()
    let duration = Instant::now() - start
    print("{} 耗时: {:.2}ms", name, duration.as_millis())
}

micro main() {
    let size = (1024, 1024)
    let a = Array::random(size)
    let b = Array::random(size)
    
    # 测试 CPU 性能
    let cpu_time = benchmark("CPU Matrix Mult") {
        compute(Backend::CPU) { a * b }
    }
    
    # 测试 GPU 性能
    let gpu_time = benchmark("GPU Matrix Mult") {
        compute(Backend::GPU) { a * b }
    }
    
    print("GPU 加速比: {:.2}x", cpu_time.as_millis() as f64 / gpu_time.as_millis() as f64)
}
```
