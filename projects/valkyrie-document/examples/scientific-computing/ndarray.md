# N维数组 (NDArray)

Valkyrie 提供了强大的N维数组类型，支持高效的数值计算和科学计算操作。数组类型包括一维、二维和N维数组，并支持多种内存布局包括GPU布局。

## 基本数组类型

### Array1D - 一维数组

```valkyrie
# 创建一维数组
let arr1d = Array1D::new([1, 2, 3, 4, 5])
let zeros = Array1D::zeros(100)
let ones = Array1D::ones(50)
let range = Array1D::range(0, 10, 1)  # 从0到10，步长为1

# 随机数组
let random = Array1D::random(1000)  # 1000个随机数
let normal = Array1D::normal(500, 0.0, 1.0)  # 正态分布

# 基本操作
let length = arr1d.len()
let sum = arr1d.sum()
let mean = arr1d.mean()
let max_val = arr1d.max()
let min_val = arr1d.min()

# 索引和切片
let element = arr1d[2]
let slice = arr1d[1..4]  # 切片操作
let filtered = arr1d.filter({ $x -> $x > 2 })  # 条件过滤
```

### Array2D - 二维数组

```valkyrie
# 创建二维数组
let arr2d = Array2D::new([
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
])

let zeros_2d = Array2D::zeros([3, 4])  # 3行4列的零矩阵
let ones_2d = Array2D::ones([2, 5])    # 2行5列的单位矩阵
let identity = Array2D::eye(4)         # 4x4单位矩阵

# 形状信息
let [rows, cols] = arr2d.shape()
let total_elements = arr2d.size()

# 索引操作
let element = arr2d[[1, 2]]  # 第1行第2列
let row = arr2d.row(0)       # 第0行
let col = arr2d.col(1)       # 第1列
let submatrix = arr2d[[0..2, 1..3]]  # 子矩阵

# 矩阵运算
let transposed = arr2d.transpose()
let dot_product = arr2d.dot(other_matrix)
let element_wise = arr2d * scalar_value
```

### ArrayND - N维数组

```valkyrie
# 创建N维数组
let arr3d = ArrayND::zeros([2, 3, 4])  # 2x3x4的三维数组
let arr4d = ArrayND::ones([2, 3, 4, 5])  # 四维数组

# 从现有数据创建
let data = vec![1, 2, 3, 4, 5, 6, 7, 8]
let reshaped = ArrayND::from_vec(data, [2, 2, 2])  # 重塑为2x2x2

# 形状操作
let shape = arr3d.shape()  # 获取形状
let ndim = arr3d.ndim()    # 获取维数
let flattened = arr3d.flatten()  # 展平为一维
let reshaped = arr3d.reshape([3, 2, 4])  # 重塑形状

# 轴操作
let sum_axis0 = arr3d.sum_axis(0)  # 沿第0轴求和
let mean_axis1 = arr3d.mean_axis(1)  # 沿第1轴求均值
let max_axis2 = arr3d.max_axis(2)   # 沿第2轴求最大值
```

## 数组布局 (Array Layout)

### CPU布局

```valkyrie
# 行主序布局 (Row-major, C-style)
let row_major = Array2D::with_layout(data, [3, 4], Layout::RowMajor)

# 列主序布局 (Column-major, Fortran-style)
let col_major = Array2D::with_layout(data, [3, 4], Layout::ColMajor)

# 自定义步长
let custom_layout = Array2D::with_strides(data, [3, 4], [4, 1])
```

### GPU布局

```valkyrie
use gpu::*

# GPU内存布局
let gpu_array = Array2D::with_layout(data, [1000, 1000], Layout::GPU {
    device: GPUDevice::cuda(0),  # CUDA设备0
    memory_type: GPUMemory::Global,
    alignment: 256  # 内存对齐
})

# GPU计算
let gpu_result = gpu_array.matmul(other_gpu_array)
let cpu_result = gpu_result.to_cpu()  # 传回CPU

# 异步GPU操作
let future_result = gpu_array.async_matmul(other_gpu_array)
let result = future_result.await
```

### 混合布局

```valkyrie
# 分块布局，适合大型矩阵
let blocked_layout = Layout::Blocked {
    block_size: [64, 64],
    storage: StorageType::CPU
}

let large_matrix = Array2D::with_layout(data, [4096, 4096], blocked_layout)

# 分布式布局
let distributed_layout = Layout::Distributed {
    nodes: vec!["node1", "node2", "node3"],
    partition_strategy: PartitionStrategy::ByRows
}
```

## 数学运算

### 基本运算

```valkyrie
let a = Array2D::new([[1, 2], [3, 4]])
let b = Array2D::new([[5, 6], [7, 8]])

# 元素级运算
let add = a + b
let sub = a - b
let mul = a * b  # 元素级乘法
let div = a / b

# 矩阵运算
let matmul = a.dot(b)  # 矩阵乘法
let power = a.pow(2)   # 矩阵幂

# 数学函数
let sin_a = a.sin()
let exp_a = a.exp()
let log_a = a.log()
let sqrt_a = a.sqrt()
```

### 线性代数

```valkyrie
use linalg::*

let matrix = Array2D::new([
    [4.0, 2.0, 1.0],
    [2.0, 5.0, 3.0],
    [1.0, 3.0, 6.0]
])

# 分解
let lu = matrix.lu_decomposition()
let qr = matrix.qr_decomposition()
let svd = matrix.svd()
let eigen = matrix.eigendecomposition()

# 求解线性方程组
let b = Array1D::new([1.0, 2.0, 3.0])
let x = matrix.solve(b)  # 求解 Ax = b

# 矩阵性质
let det = matrix.determinant()
let rank = matrix.rank()
let cond = matrix.condition_number()
let norm = matrix.norm(NormType::Frobenius)
```

### 统计运算

```valkyrie
let data = Array2D::random([100, 50])

# 描述性统计
let mean = data.mean()
let std = data.std()
let var = data.var()
let median = data.median()

# 沿轴统计
let row_means = data.mean_axis(1)  # 每行的均值
let col_stds = data.std_axis(0)    # 每列的标准差

# 相关性分析
let corr_matrix = data.correlation()
let cov_matrix = data.covariance()
```

## 高级操作

### 广播 (Broadcasting)

```valkyrie
let matrix = Array2D::ones([3, 4])
let vector = Array1D::new([1, 2, 3, 4])

# 自动广播
let result = matrix + vector  # vector自动广播到每一行

# 显式广播
let broadcasted = vector.broadcast_to([3, 4])
let manual_result = matrix + broadcasted
```

### 索引和掩码

```valkyrie
let arr = Array2D::new([
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
])

# 布尔索引
let mask = arr > 5
let filtered = arr.where(mask)  # 获取大于5的元素

# 花式索引
let indices = Array1D::new([0, 2])
let selected_rows = arr.take_rows(indices)

# 条件赋值
arr.where_assign(mask, 0)  # 将大于5的元素设为0
```

### 窗口操作

```valkyrie
let signal = Array1D::random(1000)

# 滑动窗口
let windowed = signal.sliding_window(10)  # 窗口大小为10
let window_means = windowed.map({ $window -> $window.mean() })

# 卷积
let kernel = Array1D::new([0.25, 0.5, 0.25])  # 平滑核
let smoothed = signal.convolve(kernel)

# 二维卷积
let image = Array2D::random([256, 256])
let edge_kernel = Array2D::new([
    [-1, -1, -1],
    [-1,  8, -1],
    [-1, -1, -1]
])
let edges = image.convolve2d(edge_kernel)
```

## 性能优化

### 内存管理

```valkyrie
# 预分配内存
let mut result = Array2D::uninitialized([1000, 1000])
result.fill_with({ Array2D::random([1000, 1000]) })

# 就地操作
let mut arr = Array2D::ones([500, 500])
arr.add_assign(other_array)  # 就地加法，避免分配新内存
arr.mul_assign(2.0)          # 就地标量乘法

# 视图操作
let view = arr.view([100..400, 200..300])  # 创建视图，不复制数据
let mut_view = arr.view_mut([0..100, 0..100])  # 可变视图
```

### 并行计算

```valkyrie
use parallel::*

# 并行元素操作
let large_array = Array2D::random([10000, 10000])
let parallel_result = large_array.par_map({ $x -> $x.sin() })

# 并行归约
let parallel_sum = large_array.par_sum()
let parallel_max = large_array.par_max()

# 并行矩阵乘法
let a = Array2D::random([2000, 1500])
let b = Array2D::random([1500, 1000])
let parallel_matmul = a.par_dot(b)
```

### SIMD优化

```valkyrie
# 启用SIMD优化
let arr = Array1D::random(10000).with_simd(true)

# SIMD友好的操作
let vectorized_add = arr.simd_add(other_array)
let vectorized_mul = arr.simd_mul(scalar)

# 自动向量化
#[simd_optimize]
fn custom_operation(arr: Array1D<f32>) -> Array1D<f32> {
    arr.map({ $x -> $x * $x + 2.0 * $x + 1.0 })
}
```

## 互操作性

### 与其他库的集成

```valkyrie
# 从NumPy数组导入
let numpy_array = import_numpy("data.npy")
let valkyrie_array = Array2D::from_numpy(numpy_array)

# 导出到NumPy
let exported = valkyrie_array.to_numpy()
export_numpy(exported, "result.npy")

# 与Rust ndarray互操作
let rust_ndarray = ndarray::Array2::zeros((100, 100))
let valkyrie_array = Array2D::from_ndarray(rust_ndarray)
```

### 序列化

```valkyrie
# 二进制序列化
let arr = Array2D::random([1000, 1000])
let serialized = arr.serialize_binary()
let deserialized = Array2D::deserialize_binary(serialized)

# HDF5格式
let hdf5_file = HDF5File::create("data.h5")
hdf5_file.write_array("dataset", arr)
let loaded = hdf5_file.read_array::<Array2D<f64>>("dataset")
```

## 最佳实践

### 1. 选择合适的布局

```valkyrie
# 根据访问模式选择布局
let row_access_matrix = Array2D::with_layout(data, shape, Layout::RowMajor)
let col_access_matrix = Array2D::with_layout(data, shape, Layout::ColMajor)

# GPU密集计算使用GPU布局
let gpu_matrix = Array2D::with_layout(data, shape, Layout::GPU {
    device: GPUDevice::best_available(),
    memory_type: GPUMemory::Global,
    alignment: 256
})
```

### 2. 内存效率

```valkyrie
# 避免不必要的复制
fn efficient_computation(arr: &Array2D<f64>) -> Array2D<f64> {
    # 使用视图而不是复制
    let view = arr.view([100..900, 100..900])
    
    # 就地操作
    let mut result = view.to_owned()
    result.add_assign(1.0)
    result
}
```

### 3. 数值稳定性

```valkyrie
# 使用数值稳定的算法
fn stable_matrix_inverse(matrix: Array2D<f64>) -> Array2D<f64> {
    # 使用SVD而不是直接求逆
    let svd = matrix.svd()
    svd.pseudo_inverse()
}
```

N维数组为 Valkyrie 提供了强大的数值计算基础，支持从简单的向量运算到复杂的张量操作，并通过多种布局优化满足不同场景的性能需求。