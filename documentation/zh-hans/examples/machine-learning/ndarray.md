# 数组操作统一指南

Valkyrie 提供了完整的多维数组操作体系，这是所有数值计算、机器学习和深度学习应用的基础。本指南涵盖了从基础数组操作到高级数值计算的完整内容。

## 核心数组类型

- `Array1D⟨T⟩` - 一维数组，向量运算的基础
- `Array2D⟨T⟩` - 二维数组，矩阵运算和图像处理
- `ArrayND⟨T⟩` - N维数组，张量运算和深度学习

所有数组类型都支持泛型、高效内存管理和丰富的数学运算API。

## 数组创建

### 基础创建方法

```valkyrie
# 一维数组创建
let v = Array1D::new([1, 2, 3, 4, 5])
let zeros_1d = Array1D::zeros(100)
let ones_1d = Array1D::ones(50)
let range_1d = Array1D::range(0, 10, 1)

# 二维数组创建
let M = Array2D::new([[1, 2, 3], [4, 5, 6]])
let zeros_2d = Array2D::zeros([3, 4])
let identity = Array2D::eye(3)

# N维数组创建
let T = ArrayND::zeros([2, 3, 4, 5])
let from_data = ArrayND::from_vec(data, [2, 3, 4])

# 随机数组
let r = Array1D::random(1000)
let normal_2d = Array2D::normal([100, 50], 0.0, 1.0)
let uniform_nd = ArrayND::uniform([2, 3, 4], -1.0, 1.0)
```

## 数组操作

### 基本操作

```valkyrie
# 形状和属性
let shape = A.shape()        # 获取形状
let ndim = A.ndim()          # 获取维数
let size = A.size()          # 总元素数
let dtype = A.dtype()        # 数据类型

## 索引语义

Valkyrie 提供两种不同的索引语法，分别对应不同的语义：

### 序数索引 `[i]`

使用方括号 `[` `]` 进行序数索引，表示"第几个"：

- 从 **1** 开始计数
- 正数表示正序第 n 个元素
- 负数表示倒数第 n 个元素
- `0` 是非法索引

```valkyrie
let arr = [10, 20, 30, 40, 50]

arr[1]   # 10 - 第一个元素
arr[3]   # 30 - 第三个元素
arr[-1]  # 50 - 最后一个元素
arr[-2]  # 40 - 倒数第二个元素
```

### 基数索引 `⁅i⁆`

使用 `⁅` `⁆` (U+2045, U+2046) 进行基数索引，表示"偏移量"：

- 从 **0** 开始计数
- 符合指针代数语义
- `ptr⁅0⁆` 等价于 `*ptr`
- `ptr⁅n⁆` 等价于 `*(ptr + n)`

```valkyrie
let arr = [10, 20, 30, 40, 50]

arr⁅0⁆  # 10 - 偏移量为 0 的元素
arr⁅2⁆  # 30 - 偏移量为 2 的元素
arr⁅4⁆  # 50 - 偏移量为 4 的元素
```

### 选择建议

- **人类友好场景**：使用序数索引 `[i]`，更符合自然语言习惯
- **底层指针运算**：使用基数索引 `⁅i⁆`，与 C/Rust 等语言习惯一致
- **科学计算**：推荐使用基数索引，与 NumPy/Julia 等工具一致

# 索引和切片
let element = v⁅2⁆           # 一维索引
let element_2d = M[[1, 2]]  # 二维索引
let slice = v[1..4]          # 切片
let subarray = T[[0..2, 1..3, :]]  # 多维切片

# 统计运算
let sum = A.sum()           # 求和
let mean = A.mean()          # 均值
let std = A.std()           # 标准差
let max_val = max(A)     # 最大值
let min_val = min(A)     # 最小值

# 沿轴运算
let sum_axis0 = A.sum(axis: 0)    # 沿轴0求和
let mean_axis1 = A.mean(axis: 1)   # 沿轴1求均值
```

### 形状操作

```valkyrie
# 形状变换
let flattened = A.flatten()           # 展平为一维
let reshaped = A.reshape([3, 2, 4])   # 重塑形状
let transposed = M.transpose()       # 转置
let swapped = T.swap_axes(0, 2)      # 交换轴

# 维度操作
let expanded = A.expand_dims(1)       # 在指定位置增加维度
let squeezed = A.squeeze()            # 移除长度为1的维度
let unsqueezed = A.unsqueeze(0)       # 在指定位置增加长度为1的维度

# 命名轴操作（高级特性）
let X = ArrayND::zeros([32, 3, 224, 224])
    .with_axis_names(["batch", "channel", "height", "width"])

let first_image = X.select("batch", 0)
let red_channel = X.select("channel", 0)
let batch_mean = X.mean_along("batch")
```

## 数学运算

### 基础运算

```valkyrie
# 元素级运算
let add = a + b              # 加法
let sub = a - b              # 减法
let mul = a * b              # 元素级乘法
let div = a / b              # 除法
let pow = a ^ 2               # 幂运算

# 数学函数
let sin_a = sin(a)
let cos_a = cos(a)
let exp_a = exp(a)
let log_a = ln(a)
let sqrt_a = sqrt(a)
let abs_a = abs(a)

# 矩阵运算
let matmul = A @ B              # 矩阵乘法
let outer = a.outer(b)              # 外积
```

### 线性代数

```valkyrie
# 矩阵分解
let lu = LU(A)               # LU分解
let qr = QR(A)               # QR分解
let svd = SVD(A)             # 奇异值分解
let eigen = A.eigen()             # 特征值分解

# 求解线性方程组
let b = Array1D::new([1.0, 2.0, 3.0])
let x = solve(A, b)         # 求解 Ax = b

# 矩阵性质
let det = det(A)             # 行列式
let rank = rank(A)           # 矩阵的秩
let inv = A.inv()                # 逆矩阵
let norm = A.norm()               # 矩阵范数
```

## 高级操作

### 广播机制

```valkyrie
# 自动广播
let A = Array2D::ones([3, 4])
let v = Array1D::new([1, 2, 3, 4])
let result = A + v          # v自动广播到每一行

# 显式广播
let broadcasted = broadcast(v, [3, 4])
let manual_result = A + broadcasted

# 广播规则检查
let compatible = Array::broadcast_compatible(shape_a, shape_b)
```

### 索引和条件操作

```valkyrie
# 布尔索引
let m = A > 5.0
let filtered = A.where(m)  # 获取满足条件的元素
let masked_array = A.mask(m)  # 应用掩码

# 花式索引
let I = Array1D::new([0, 2, 4])
let selected = A.take(I)  # 按索引选择元素
let selected_rows = M.take_rows(I)

# 条件赋值
A.where_assign(m, 0.0)  # 将满足条件的元素设为0
A.clip(0.0, 1.0)  # 将值限制在[0, 1]范围内

# 查找操作
let i_max = A.argmax()  # 最大值索引
let i_min = A.argmin()  # 最小值索引
let nonzero = A.nonzero()     # 非零元素索引
```

### 数据处理

```valkyrie
# 排序操作
let sorted = A.sort()           # 排序
let argsorted = A.argsort()     # 排序索引
let sorted_axis = M.sort_axis(0)  # 沿轴排序

# 唯一值操作
let unique = array.unique()         # 唯一值
let counts = array.value_counts()   # 值计数

# 数据变换
let normalized = array.normalize()  # 归一化到[0,1]
let standardized = array.standardize()  # 标准化(零均值单位方差)
let centered = array - array.mean()  # 中心化

# 缺失值处理
let filled = array.fill_nan(0.0)   # 填充NaN值
let dropped = array.drop_nan()      # 删除NaN值
let interpolated = array.interpolate()  # 插值填充
```

## 内存管理

### 高效内存操作

```valkyrie
# 预分配内存
let mut result = Array2D::uninitialized([1000, 1000])
result.fill_with_fn { i, j -> i + j }

# 就地操作
let mut array = Array2D::ones([500, 500])
array.add_assign(other_array)  # 就地加法，避免分配新内存
array.mul_assign(2.0)          # 就地标量乘法

# 视图操作
let view = array.view([100..400, 200..300])  # 创建视图，不复制数据
let mut_view = array.view_mut([0..100, 0..100])  # 可变视图

# 内存布局控制
let row_major = Array2D::with_layout(data, shape, Layout::RowMajor)
let col_major = ArrayColMajor::with_layout(data, shape, Layout::ColMajor)
```

## 数据导入导出

### 文件操作

```valkyrie
# CSV文件操作
let csv_data = Array2D::from_csv("data.csv")
array.to_csv("output.csv")

# 二进制格式
let binary_data = array.to_bytes()
let restored = Array2D::from_bytes(binary_data, shape)

# NumPy兼容格式
let numpy_array = Array2D::from_npy("data.npy")
array.to_npy("output.npy")

# JSON格式
let json_data = array.to_json()
let from_json = Array2D::from_json(json_data)
```

### 数据转换

```valkyrie
# 与其他类型转换
let vec_data: [f64] = array.to_vec()
let from_vec = Array1D::from_vec(vec_data)

# 类型转换
let float_array = int_array.cast⟨f64⟩()
let int_array = float_array.cast⟨i32⟩()

# 与标准库集成
let slice: [f64] = array.as_slice()
let mut_slice: mut [f64] = array.as_mut_slice()
```

## 性能优化

### 高效编程模式

```valkyrie
# 选择合适的数据类型
let high_precision = Array2D⟨f64⟩::zeros([1000, 1000])  # 双精度
let fast_computation = Array2D⟨f32⟩::zeros([1000, 1000])  # 单精度，更快
let integer_data = Array2D⟨i32⟩::zeros([1000, 1000])     # 整数

# 避免不必要的复制
let view = array.view([100..900, 100..900])  # 使用视图
let mut result = view.to_owned()  # 仅在需要时复制

# 批量操作
let arrays = [array1, array2, array3]
let batch_sum = arrays.iter().fold(Array2D::zeros(shape)) { acc, x -> acc + x }

# 数值稳定性
let max_val = array.max()
let stable_result = (array - max_val).exp()  # 防止溢出
```

## 应用场景

### 常见用例

```valkyrie
# 图像处理
let image = Array2D::from_image("photo.jpg")
let resized = image.resize([224, 224])
let normalized = (resized.cast⟨f32⟩() / 255.0 - 0.5) / 0.5

# 数据分析
let data = Array2D::from_csv("dataset.csv")
let correlation = data.correlation_matrix()
let pca_result = data.pca(n_components: 10)

# 科学计算
let signal = Array1D::linspace(0.0, 10.0, 1000)
let fft_result = signal.fft()
let filtered = fft_result.filter_frequencies(cutoff: 5.0).ifft()
```

## 总结

Valkyrie 的数组系统提供了完整的数值计算基础设施：

- **统一的API设计** - 一维、二维和N维数组使用一致的接口
- **高效的内存管理** - 支持视图、就地操作和自定义布局
- **丰富的数学运算** - 从基础运算到高级线性代数
- **灵活的数据处理** - 索引、条件操作、数据变换
- **广泛的兼容性** - 支持多种文件格式和数据转换

这些特性使得 Valkyrie 数组成为机器学习、深度学习、科学计算和数据分析的理想选择。