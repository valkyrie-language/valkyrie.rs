# 數組操作統一指南

Valkyrie 提供了完整的多維數組操作體系，這是所有數值計算、機器學習和深度學習應用的基礎。本指南涵盖了從基礎數組操作到高級數值計算的完整內容。

## 核心數組類型

- `Array1D⟨T⟩` - 一維數組，向量運算的基礎
- `Array2D⟨T⟩` - 二維數組，矩陣運算和圖像處理
- `ArrayND⟨T⟩` - N維數組，張量運算和深度學習

所有數組類型都支持泛型、高效內存管理和豐富的數學運算API。

## 數組創建

### 基礎創建方法

```valkyrie
# 一維數組創建
let v = Array1D::new([1, 2, 3, 4, 5])
let zeros_1d = Array1D::zeros(100)
let ones_1d = Array1D::ones(50)
let range_1d = Array1D::range(0, 10, 1)

# 二維數組創建
let M = Array2D::new([[1, 2, 3], [4, 5, 6]])
let zeros_2d = Array2D::zeros([3, 4])
let identity = Array2D::eye(3)

# N維數組創建
let T = ArrayND::zeros([2, 3, 4, 5])
let from_data = ArrayND::from_vec(data, [2, 3, 4])

# 隨機數組
let r = Array1D::random(1000)
let normal_2d = Array2D::normal([100, 50], 0.0, 1.0)
let uniform_nd = ArrayND::uniform([2, 3, 4], -1.0, 1.0)
```

## 數組操作

### 基本操作

```valkyrie
# 形狀和屬性
let shape = A.shape()        # 獲取形狀
let ndim = A.ndim()          # 獲取維數
let size = A.size()          # 總元素數
let dtype = A.dtype()        # 數據類型

# 索引和切片
let element = v[2]           # 一維索引
let element_2d = M[[1, 2]]  # 二維索引
let slice = v[1..4]          # 切片
let subarray = T[[0..2, 1..3, :]]  # 多維切片

# 統計運算
let sum = A.sum()           # 求和
let mean = A.mean()          # 均值
let std = A.std()           # 標準差
let max_val = max(A)     # 最大值
let min_val = min(A)     # 最小值

# 沿軸運算
let sum_axis0 = A.sum(axis: 0)    # 沿軸0求和
let mean_axis1 = A.mean(axis: 1)   # 沿軸1求均值
```

### 形狀操作

```valkyrie
# 形狀變換
let flattened = A.flatten()           # 展平為一維
let reshaped = A.reshape([3, 2, 4])   # 重塑形狀
let transposed = M.transpose()       # 轉置
let swapped = T.swap_axes(0, 2)      # 交換軸

# 維度操作
let expanded = A.expand_dims(1)       # 在指定位置增加維度
let squeezed = A.squeeze()            # 移除長度為1的維度
let unsqueezed = A.unsqueeze(0)       # 在指定位置增加長度為1的維度

# 命名軸操作（高級特性）
let X = ArrayND::zeros([32, 3, 224, 224])
    .with_axis_names(["batch", "channel", "height", "width"])

let first_image = X.select("batch", 0)
let red_channel = X.select("channel", 0)
let batch_mean = X.mean_along("batch")
```

## 數學運算

### 基礎運算

```valkyrie
# 元素級運算
let add = a + b              # 加法
let sub = a - b              # 減法
let mul = a * b              # 元素級乘法
let div = a / b              # 除法
let pow = a ^ 2               # 幂運算

# 數學函數
let sin_a = sin(a)
let cos_a = cos(a)
let exp_a = exp(a)
let log_a = ln(a)
let sqrt_a = sqrt(a)
let abs_a = abs(a)

# 矩陣運算
let matmul = A @ B              # 矩陣乘法
let outer = a.outer(b)              # 外積
```

### 線性代數

```valkyrie
# 矩陣分解
let lu = LU(A)               # LU分解
let qr = QR(A)               # QR分解
let svd = SVD(A)             # 奇异值分解
let eigen = A.eigen()             # 特徵值分解

# 求解線性方程組
let b = Array1D::new([1.0, 2.0, 3.0])
let x = solve(A, b)         # 求解 Ax = b

# 矩陣性質
let det = det(A)             # 行列式
let rank = rank(A)           # 矩陣的秩
let inv = A.inv()                # 逆矩陣
let norm = A.norm()               # 矩陣範數
```

## 高級操作

### 廣播機制

```valkyrie
# 自動廣播
let A = Array2D::ones([3, 4])
let v = Array1D::new([1, 2, 3, 4])
let result = A + v          # v自動廣播到每一行

# 显式廣播
let broadcasted = broadcast(v, [3, 4])
let manual_result = A + broadcasted

# 廣播規則檢查
let compatible = Array::broadcast_compatible(shape_a, shape_b)
```

### 索引和條件操作

```valkyrie
# 布爾索引
let m = A > 5.0
let filtered = A.where(m)  # 獲取满足條件的元素
let masked_array = A.mask(m)  # 應用掩碼

# 花式索引
let I = Array1D::new([0, 2, 4])
let selected = A.take(I)  # 按索引選擇元素
let selected_rows = M.take_rows(I)

# 條件賦值
A.where_assign(m, 0.0)  # 將满足條件的元素设為0
A.clip(0.0, 1.0)  # 將值限制在[0, 1]範圍內

# 查找操作
let i_max = A.argmax()  # 最大值索引
let i_min = A.argmin()  # 最小值索引
let nonzero = A.nonzero()     # 非零元素索引
```

### 數據處理

```valkyrie
# 排序操作
let sorted = A.sort()           # 排序
let argsorted = A.argsort()     # 排序索引
let sorted_axis = M.sort_axis(0)  # 沿軸排序

# 唯一值操作
let unique = array.unique()         # 唯一值
let counts = array.value_counts()   # 值计數

# 數據變換
let normalized = array.normalize()  # 歸一化到[0,1]
let standardized = array.standardize()  # 標準化(零均值單位方差)
let centered = array - array.mean()  # 中心化

# 缺失值處理
let filled = array.fill_nan(0.0)   # 填充NaN值
let dropped = array.drop_nan()      # 删除NaN值
let interpolated = array.interpolate()  # 插值填充
```

## 內存管理

### 高效內存操作

```valkyrie
# 預分配內存
let mut result = Array2D::uninitialized([1000, 1000])
result.fill_with_fn { i, j -> i + j }

# 就地操作
let mut array = Array2D::ones([500, 500])
array.add_assign(other_array)  # 就地加法，避免分配新內存
array.mul_assign(2.0)          # 就地标量乘法

# 视圖操作
let view = array.view([100..400, 200..300])  # 創建视圖，不複製數據
let mut_view = array.view_mut([0..100, 0..100])  # 可变视圖

# 內存佈局控制
let row_major = Array2D::with_layout(data, shape, Layout::RowMajor)
let col_major = ArrayColMajor::with_layout(data, shape, Layout::ColMajor)
```

## 數據导入導出

### 文件操作

```valkyrie
# CSV文件操作
let csv_data = Array2D::from_csv("data.csv")
array.to_csv("output.csv")

# 二進制格式
let binary_data = array.to_bytes()
let restored = Array2D::from_bytes(binary_data, shape)

# NumPy兼容格式
let numpy_array = Array2D::from_npy("data.npy")
array.to_npy("output.npy")

# JSON格式
let json_data = array.to_json()
let from_json = Array2D::from_json(json_data)
```

### 數據轉換

```valkyrie
# 與其他類型轉換
let vec_data: [f64] = array.to_vec()
let from_vec = Array1D::from_vec(vec_data)

# 類型轉換
let float_array = int_array.cast⟨f64⟩()
let int_array = float_array.cast⟨i32⟩()

# 與標準庫集成
let slice: [f64] = array.as_slice()
let mut_slice: mut [f64] = array.as_mut_slice()
```

## 性能優化

### 高效編程模式

```valkyrie
# 選擇合適的數據類型
let high_precision = Array2D⟨f64⟩::zeros([1000, 1000])  # 雙精度
let fast_computation = Array2D⟨f32⟩::zeros([1000, 1000])  # 單精度，更快
let integer_data = Array2D⟨i32⟩::zeros([1000, 1000])     # 整數

# 避免不必要的複製
let view = array.view([100..900, 100..900])  # 使用视圖
let mut result = view.to_owned()  # 仅在需要時複製

# 批量操作
let arrays = [array1, array2, array3]
let batch_sum = arrays.iter().fold(Array2D::zeros(shape)) { acc, x -> acc + x }

# 數值穩定性
let max_val = array.max()
let stable_result = (array - max_val).exp()  # 防止溢出
```

## 應用場景

### 常见用例

```valkyrie
# 圖像處理
let image = Array2D::from_image("photo.jpg")
let resized = image.resize([224, 224])
let normalized = (resized.cast⟨f32⟩() / 255.0 - 0.5) / 0.5

# 數據分析
let data = Array2D::from_csv("dataset.csv")
let correlation = data.correlation_matrix()
let pca_result = data.pca(n_components: 10)

# 科學計算
let signal = Array1D::linspace(0.0, 10.0, 1000)
let fft_result = signal.fft()
let filtered = fft_result.filter_frequencies(cutoff: 5.0).ifft()
```

## 總結

Valkyrie 的數組系統提供了完整的數值計算基礎设施：

- **統一的API設計** - 一維、二維和N維數組使用一致的接口
- **高效的內存管理** - 支持视圖、就地操作和自定義佈局
- **豐富的數學運算** - 從基礎運算到高級線性代數
- **靈活的數據處理** - 索引、條件操作、數據變換
- **广泛的兼容性** - 支持多種文件格式和數據轉換

這些特性使得 Valkyrie 數組成為機器學習、深度學習、科學計算和數據分析的理想選擇。