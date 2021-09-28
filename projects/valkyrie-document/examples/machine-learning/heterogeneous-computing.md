# 异构计算

在 Valkyrie 中，异构计算通过统一的数组抽象实现跨设备的高性能计算。

## 数组类型体系

- `Array` 是一段内存中的数据
- `Array<T, N>` 是 `[T; N]` 的语法糖，表示固定大小数组
- `ArrayND` 是异构计算、机器学习、深度学习的基础多维数组类型
- `Array1D` 是 `ArrayND` 的 type alias，专门用于一维数组操作

ArrayND 可以选择不同的设备 (device) 和布局 (layout)，系统会自动进行优化。

## 基本用法

### 创建数组

```valkyrie
# 创建不同维度的数组
let vector = Array1D::zeros(1000)           # 一维数组
let matrix = ArrayND::zeros([1000, 1000])   # 二维数组
let tensor = ArrayND::zeros([32, 3, 224, 224])  # 四维张量

# 从数据创建
let data = [1.0, 2.0, 3.0, 4.0]
let arr = Array1D::from(data)
let reshaped = arr.reshape([2, 2])  # 重塑为2x2矩阵
```

### 设备选择

```valkyrie
# 在不同设备上创建数组
let cpu_array = ArrayND::zeros([1024, 1024]).on_device(Device::CPU)
let gpu_array = ArrayND::zeros([1024, 1024]).on_device(Device::GPU)

# 设备间转换
let gpu_result = cpu_array.to_device(Device::GPU)
let cpu_result = gpu_array.to_cpu()
```

## 布局优化

### 自动布局选择

```valkyrie
# 系统会根据操作自动选择最优布局
let a = ArrayND::random([1024, 1024])
let b = ArrayND::random([1024, 1024])

# 矩阵乘法会自动选择最优的内存布局
let result = a.matmul(&b)  # 自动优化

# 手动指定布局（通常不需要）
let row_major = a.with_layout(Layout::RowMajor)
let col_major = a.with_layout(Layout::ColMajor)
```

### 异步数据传输

```valkyrie
# 异步设备间传输
async fn process_large_data() {
    let data = ArrayND::load("dataset.bin")
    let gpu_data = data.to_device_async(Device::GPU).await
    
    # GPU计算
    let result = gpu_data.matmul(&gpu_data.transpose())
    
    # 传回CPU保存
    let cpu_result = result.to_cpu_async().await
    cpu_result.save("result.bin")
}
```

## 常用操作

### 数学运算

```valkyrie
# 基本运算（自动选择最优设备和布局）
let a = ArrayND::random([1000, 1000])
let b = ArrayND::random([1000, 1000])

# 矩阵运算
let sum = a + b
let product = a * b  # 逐元素乘法
let matmul = a.matmul(&b)  # 矩阵乘法
let transpose = a.transpose()

# 统计运算
let mean = a.mean()
let sum_all = a.sum()
let max_val = a.max()
let std_dev = a.std()
```

## 机器学习应用

### 神经网络训练

```valkyrie
# 简单的线性层
struct LinearLayer {
    weights: ArrayND,
    bias: ArrayND,
}

impl LinearLayer {
    fn new(input_size: usize, output_size: usize) -> Self {
        Self {
            weights: ArrayND::random([input_size, output_size]).on_device(Device::GPU),
            bias: ArrayND::zeros([output_size]).on_device(Device::GPU),
        }
    }
    
    fn forward(&self, input: &ArrayND) -> ArrayND {
        input.matmul(&self.weights) + &self.bias
    }
    
    fn backward(&mut self, grad_output: &ArrayND, input: &ArrayND, learning_rate: f32) {
        # 计算梯度
        let grad_weights = input.transpose().matmul(grad_output)
        let grad_bias = grad_output.sum_axis(0)
        
        # 更新参数
        self.weights = &self.weights - learning_rate * &grad_weights
        self.bias = &self.bias - learning_rate * &grad_bias
    }
}
```

### 批处理训练

```valkyrie
# 批量处理数据
fn train_batch(model: &mut LinearLayer, batch_x: &ArrayND, batch_y: &ArrayND, lr: f32) {
    # 前向传播
    let predictions = model.forward(batch_x)
    
    # 计算损失梯度
    let loss_grad = predictions - batch_y
    
    # 反向传播
    model.backward(&loss_grad, batch_x, lr)
}

# 完整训练循环
fn train_model(train_data: &[(ArrayND, ArrayND)], epochs: usize) {
    let mut model = LinearLayer::new(784, 10)  # MNIST分类
    
    for epoch in 0..epochs {
        for (batch_x, batch_y) in train_data {
            train_batch(&mut model, batch_x, batch_y, 0.01)
        }
        println!("Epoch {} completed", epoch)
    }
}
```

## 多设备并行

### 数据并行训练

```valkyrie
# 在多个GPU上并行训练
fn parallel_training(data: &[ArrayND], num_gpus: usize) {
    let batch_size = data.len() / num_gpus
    let mut models = Vec::new()
    
    # 在每个GPU上创建模型副本
    for i in 0..num_gpus {
        let model = LinearLayer::new(784, 10).on_device(Device::GPU(i))
        models.push(model)
    }
    
    # 并行处理数据批次
    for chunk in data.chunks(batch_size) {
        let results = models.par_iter_mut()
            .zip(chunk.par_iter())
            .map(|(model, batch)| {
                model.forward(batch)
            })
            .collect::<Vec<_>>()
        
        # 聚合结果
        let averaged = results.iter().fold(ArrayND::zeros([10]), |acc, x| acc + x) / num_gpus as f32
    }
}
```

### 模型并行

```valkyrie
# 将大模型分布到多个设备
struct DistributedModel {
    layers: Vec<LinearLayer>,
    devices: Vec<Device>,
}

impl DistributedModel {
    fn forward(&self, input: ArrayND) -> ArrayND {
        let mut x = input
        
        for (layer, device) in self.layers.iter().zip(&self.devices) {
            # 传输到对应设备
            x = x.to_device(device)
            
            # 执行计算
            x = layer.forward(&x)
        }
        
        x
    }
}
```

## 内存优化策略

### 梯度检查点

```valkyrie
struct GradientCheckpointing {
    checkpoint_layers: HashSet<LayerId>,
    recompute_cache: HashMap<LayerId, Tensor>,
}

impl GradientCheckpointing {
    # 前向传播时选择性保存中间结果
    fn forward_with_checkpointing(&mut self, model: &Model, input: Tensor) -> Tensor {
        let mut activations = HashMap::new()
        let mut current = input
        
        for (i, layer) in model.layers.iter().enumerate() {
            current = layer.forward(current)
            
            # 只在检查点层保存激活值
            if self.checkpoint_layers.contains(&LayerId(i)) {
                activations.insert(LayerId(i), current.clone())
            }
        }
        
        self.recompute_cache = activations
        current
    }
    
    # 反向传播时重新计算中间结果
    fn backward_with_recomputation(&mut self, model: &Model, grad_output: Tensor) -> Tensor {
        let mut grad = grad_output
        
        for (i, layer) in model.layers.iter().enumerate().rev() {
            let layer_id = LayerId(i)
            
            # 如果需要激活值但没有保存，则重新计算
            let activation = if let Some(cached) = self.recompute_cache.get(&layer_id) {
                cached.clone()
            } else {
                self.recompute_activation(model, layer_id)
            }
            
            grad = layer.backward(grad, &activation)
        }
        
        grad
    }
}
```

### 动态内存管理

```valkyrie
struct DynamicMemoryManager {
    memory_pools: HashMap<ComputeDevice, MemoryPool>,
    allocation_strategy: AllocationStrategy,
}

enum AllocationStrategy {
    FirstFit,
    BestFit,
    WorstFit,
    Buddy,
}

impl DynamicMemoryManager {
    # 智能内存分配
    fn allocate_tensor<T>(&mut self, shape: &[usize], device: &ComputeDevice) -> Result<Tensor<T>, MemoryError> {
        let size = shape.iter().product::<usize>() * std::mem::size_of::<T>()
        let pool = self.memory_pools.get_mut(device).unwrap()
        
        match self.allocation_strategy {
            AllocationStrategy::FirstFit => {
                pool.allocate_first_fit(size)
            },
            AllocationStrategy::BestFit => {
                pool.allocate_best_fit(size)
            },
            AllocationStrategy::Buddy => {
                pool.allocate_buddy(size)
            },
            _ => pool.allocate_first_fit(size)
        }
    }
    
    # 内存压缩和整理
    fn compact_memory(&mut self, device: &ComputeDevice) {
        if let Some(pool) = self.memory_pools.get_mut(device) {
            pool.defragment()
            pool.compact()
        }
    }
    
    # 自适应策略调整
    fn adapt_strategy(&mut self, device: &ComputeDevice, allocation_pattern: &AllocationPattern) {
        let current_fragmentation = self.measure_fragmentation(device)
        
        self.allocation_strategy = match allocation_pattern {
            AllocationPattern::ManySmallAllocations if current_fragmentation > 0.3 => {
                AllocationStrategy::Buddy
            },
            AllocationPattern::FewLargeAllocations => {
                AllocationStrategy::FirstFit
            },
            AllocationPattern::MixedSizes => {
                AllocationStrategy::BestFit
            },
            _ => self.allocation_strategy
        }
    }
}
```

## 性能监控和调优

### 性能分析器

```valkyrie
struct PerformanceProfiler {
    kernel_times: HashMap<String, Vec<Duration>>,
    memory_usage: HashMap<ComputeDevice, Vec<usize>>,
    bandwidth_usage: HashMap<(ComputeDevice, ComputeDevice), Vec<f64>>,
}

impl PerformanceProfiler {
    # 记录内核执行时间
    fn profile_kernel<F, R>(&mut self, kernel_name: &str, device: &ComputeDevice, f: F) -> R 
    where F: FnOnce() -> R {
        let start = Instant::now()
        let result = f()
        let duration = start.elapsed()
        
        self.kernel_times.entry(kernel_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration)
        
        result
    }
    
    # 生成性能报告
    fn generate_report(&self) -> PerformanceReport {
        let mut report = PerformanceReport::new()
        
        # 内核性能统计
        for (kernel, times) in &self.kernel_times {
            let avg_time = times.iter().sum::<Duration>() / times.len() as u32
            let max_time = times.iter().max().unwrap()
            let min_time = times.iter().min().unwrap()
            
            report.add_kernel_stats(kernel, KernelStats {
                average_time: avg_time,
                max_time: *max_time,
                min_time: *min_time,
                call_count: times.len(),
            })
        }
        
        # 内存使用统计
        for (device, usage) in &self.memory_usage {
            let peak_usage = usage.iter().max().unwrap_or(&0)
            let avg_usage = usage.iter().sum::<usize>() / usage.len().max(1)
            
            report.add_memory_stats(device, MemoryStats {
                peak_usage: *peak_usage,
                average_usage: avg_usage,
            })
        }
        
        report
    }
    
    # 自动调优建议
    fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new()
        
        # 检查内核性能
        for (kernel, times) in &self.kernel_times {
            let avg_time = times.iter().sum::<Duration>() / times.len() as u32
            
            if avg_time > Duration::from_millis(100) {
                suggestions.push(OptimizationSuggestion::SlowKernel {
                    kernel_name: kernel.clone(),
                    average_time: avg_time,
                    recommendation: "考虑使用更高效的算法或增加并行度".to_string(),
                })
            }
        }
        
        # 检查内存使用
        for (device, usage) in &self.memory_usage {
            let peak = usage.iter().max().unwrap_or(&0)
            let device_memory = self.get_device_memory(device)
            
            if *peak as f64 / device_memory as f64 > 0.9 {
                suggestions.push(OptimizationSuggestion::HighMemoryUsage {
                    device: device.clone(),
                    usage_ratio: *peak as f64 / device_memory as f64,
                    recommendation: "考虑使用梯度检查点或模型并行来减少内存使用".to_string(),
                })
            }
        }
        
        suggestions
    }
}
```

## Unicode 希腊字母支持

Valkyrie 原生支持 Unicode 希腊字母，使数学表达式更加直观和符合学术标准：

```valkyrie
# 基础希腊字母变量
let α = 0.01      # alpha - 学习率
let β = 0.9       # beta - 动量参数
let γ = 0.99      # gamma - 折扣因子
let δ = 1e-6      # delta - 数值稳定性
let ε = 1e-8      # epsilon - 小量
let ζ = 0.1       # zeta - 正则化强度
let η = 0.001     # eta - 学习率调度
let θ = Matrix::random([784, 128])  # theta - 模型参数
let λ = 0.01      # lambda - 正则化系数
let μ = Array::zeros([128])         # mu - 均值
let ν = Array::ones([128])          # nu - 方差
let ξ = Random::normal(0.0, 1.0)    # xi - 随机变量
let π = 3.14159265359               # pi - 圆周率
let ρ = 0.95      # rho - 相关系数
let σ = 0.1       # sigma - 标准差
let τ = 1.0       # tau - 时间常数
let φ = 1.618     # phi - 黄金比例
let χ = Array::random([64])         # chi - 卡方分布
let ψ = Matrix::identity(128)       # psi - 波函数
let ω = 2.0 * π   # omega - 角频率

# 带下标的希腊字母
let β₁ = 0.9      # Adam优化器第一动量
let β₂ = 0.999    # Adam优化器第二动量
let σ₁ = 0.1      # 第一层标准差
let σ₂ = 0.05     # 第二层标准差
let θᵢ = Matrix::random([256, 128]) # 第i层参数
let μₜ = Array::zeros([128])        # t时刻均值
let νₜ = Array::zeros([128])        # t时刻方差

# 在神经网络中使用希腊字母
struct NeuralNetwork {
    θ: Vec<Tensor>,  # 参数向量
    ∇θ: Vec<Tensor>, # 梯度向量
    μ: Vec<Tensor>,  # 动量项
    ν: Vec<Tensor>,  # 二阶动量项
}

impl NeuralNetwork {
    # 使用希腊字母的梯度下降
    fn gradient_descent(&mut self, α: f32) {
        for (θᵢ, ∇θᵢ) in self.θ.iter_mut().zip(&self.∇θ) {
            *θᵢ = θᵢ - α * ∇θᵢ
        }
    }
    
    # Adam 优化器
    fn adam_step(&mut self, α: f32, β₁: f32, β₂: f32, ε: f32, t: usize) {
        for i in 0..self.θ.len() {
            let m = β₁ * self.μ[i] + (1.0 - β₁) * self.∇θ[i]
            let v = β₂ * self.ν[i] + (1.0 - β₂) * self.∇θ[i].powi(2)
            
            let m̂ = m / (1.0 - β₁.powi(t as i32))
            let v̂ = v / (1.0 - β₂.powi(t as i32))
            
            self.θ[i] = self.θ[i] - α * m̂ / (v̂.sqrt() + ε)
            self.μ[i] = m
            self.ν[i] = v
        }
    }
    
    # 带正则化的损失函数
    fn regularized_loss(&self, ŷ: &Tensor, y: &Tensor, λ: f32) -> f32 {
        let ℒ = self.cross_entropy_loss(ŷ, y)
        let Ω = self.l2_regularization(λ)
        ℒ + Ω
    }
}

# 数学函数使用希腊字母
fn σ(x: f64) -> f64 {  # Sigmoid激活函数
    1.0 / (1.0 + (-x).exp())
}

fn φ(x: f64) -> f64 {  # 标准正态分布CDF
    0.5 * (1.0 + (x / 2.0_f64.sqrt()).erf())
}

fn ψ(x: &Array, θ: &Matrix) -> Array {  # 神经网络前向传播
    σ(x.matmul(θ))
}

# 损失函数
fn ℒ(ŷ: &Array, y: &Array) -> f64 {  # 交叉熵损失
    let n = y.len() as f64
    -((y * ŷ.log()).sum() + ((1.0 - y) * (1.0 - ŷ).log()).sum()) / n
}

# 正则化项
fn Ω(θ: &Matrix, λ: f64) -> f64 {  # L2正则化
    λ * (θ * θ).sum() / 2.0
}

# 梯度计算
fn ∇ℒ(θ: &Matrix, x: &Array, y: &Array) -> Matrix {  # 损失函数梯度
    let ŷ = ψ(x, θ)
    let δ = ŷ - y
    x.transpose().matmul(&δ)
}

# 物理常数使用希腊字母
const π = 3.14159265358979323846
const φ = 1.618033988749895  # 黄金比例
const γ = 0.5772156649015329  # 欧拉常数
const Δ = 4.669201609102990  # Feigenbaum常数

# 高斯分布
fn gaussian(x: f64, μ: f64, σ: f64) -> f64 {
    let coefficient = 1.0 / (σ * (2.0 * π).sqrt())
    let exponent = -0.5 * ((x - μ) / σ).powi(2)
    coefficient * exponent.exp()
}

# 统计分布参数
struct BetaDistribution {
    α: f64,  # 形状参数1
    β: f64,  # 形状参数2
}

struct GammaDistribution {
    α: f64,  # 形状参数
    β: f64,  # 率参数
}

struct DirichletDistribution {
    α: Vec<f64>,  # 浓度参数向量
}

# 概率密度函数
impl BetaDistribution {
    fn pdf(&self, x: f64) -> f64 {
        let Β = gamma_function(self.α) * gamma_function(self.β) / gamma_function(self.α + self.β)
        x.powf(self.α - 1.0) * (1.0 - x).powf(self.β - 1.0) / Β
    }
}

# 优化算法中的希腊字母
struct SGDOptimizer {
    α: f32,  # 学习率
    μ: f32,  # 动量系数
}

struct AdamOptimizer {
    α: f32,   # 学习率
    β₁: f32,  # 一阶动量衰减率
    β₂: f32,  # 二阶动量衰减率
    ε: f32,   # 数值稳定性参数
}

struct RMSpropOptimizer {
    α: f32,  # 学习率
    ρ: f32,  # 衰减率
    ε: f32,  # 数值稳定性参数
}
```

## 最佳实践

### 1. 设备选择策略

```valkyrie
# 根据计算复杂度自动选择设备
fn auto_device_selection(operation: &Operation, input_size: usize) -> ComputeDevice {
    match operation {
        Operation::MatMul if input_size > 1024 * 1024 => ComputeDevice::GPU,
        Operation::ElementWise => ComputeDevice::CPU,
        Operation::Convolution => ComputeDevice::GPU,
        Operation::FFT if input_size > 4096 => ComputeDevice::GPU,
        _ => ComputeDevice::CPU,
    }
}
```

### 2. 内存布局优化

```valkyrie
# 根据访问模式选择最优布局
fn optimize_layout(access_pattern: AccessPattern) -> MemoryLayout {
    match access_pattern {
        AccessPattern::RowWise => MemoryLayout::RowMajor,
        AccessPattern::ColumnWise => MemoryLayout::ColumnMajor,
        AccessPattern::Random => MemoryLayout::Blocked,
        AccessPattern::Sequential => MemoryLayout::RowMajor,
    }
}
```

### 3. 异步计算流水线

```valkyrie
# 重叠计算和通信
async fn pipelined_computation(data_stream: DataStream) {
    let mut pipeline = ComputePipeline::new()
    
    pipeline
        .stage("load", |batch| batch.to_device_async(GPU))
        .stage("compute", |batch| model.forward_async(batch))
        .stage("store", |result| result.to_device_async(CPU))
    
    pipeline.run(data_stream).await
}
```

通过这种统一的抽象，Valkyrie 能够无缝地在不同的计算设备和内存布局之间切换，为开发者提供了强大而灵活的异构计算能力。