# NDArray

Valkyrie provides a powerful N-dimensional array (NDArray) type optimized for numerical computing and machine learning.

## Basic Usage

### Creating Arrays

```valkyrie
using std::ndarray

# Create from data
let a = NDArray.from([[1, 2, 3], [4, 5, 6]])  # Shape: (2, 3)

# Create special arrays
let zeros = NDArray.zeros([3, 4])           # 3x4 matrix of zeros
let ones = NDArray.ones([2, 3, 4])          # 2x3x4 tensor of ones
let eye = NDArray.eye(3)                     # 3x3 identity matrix
let random = NDArray.random([5, 5])          # Random values

# Create with range
let range = NDArray.arange(0, 10, 1)         # [0, 1, 2, ..., 9]
let linspace = NDArray.linspace(0, 1, 11)    # [0, 0.1, 0.2, ..., 1.0]
```

### Array Properties

```valkyrie
let a = NDArray.from([[1, 2, 3], [4, 5, 6]])

print("Shape: {a.shape}")        # (2, 3)
print("Dimensions: {a.ndim}")     # 2
print("Size: {a.size}")           # 6
print("Dtype: {a.dtype}")         # f64
```

## Indexing and Slicing

### Basic Indexing

```valkyrie
let a = NDArray.arange(0, 12).reshape([3, 4])

# Single element
let elem = a[1, 2]          # Row 1, column 2

# Row
let row = a[1]              # Entire row 1
let row2 = a[1, ..]         # Same as above

# Column
let col = a[.., 2]          # Entire column 2
```

### Slicing

```valkyrie
# Slice rows 0-1, columns 1-3
let slice = a[0..2, 1..3]

# Step slicing
let stepped = a[.., ..2]    # Every other column

# Negative indices
let last_row = a[-1]        # Last row
let last_col = a[.., -1]    # Last column
```

### Advanced Indexing

```valkyrie
# Boolean indexing
let mask = a > 5
let filtered = a[mask]      # Elements greater than 5

# Integer array indexing
let indices = NDArray.from([0, 2])
let selected = a[.., indices]  # Select columns 0 and 2
```

## Operations

### Arithmetic

```valkyrie
let a = NDArray.ones([3, 3])
let b = NDArray.ones([3, 3]) * 2

# Element-wise operations
let sum = a + b
let diff = a - b
let prod = a * b        # Element-wise multiplication
let div = a / b
let pow = a ** 2

# In-place operations
a += b
a *= 2
```

### Matrix Operations

```valkyrie
let a = NDArray.random([3, 4])
let b = NDArray.random([4, 5])

# Matrix multiplication
let c = a @ b           # Or a.matmul(b)

# Transpose
let t = a.T             # Or a.transpose()

# Dot product (for vectors)
let v1 = NDArray.from([1, 2, 3])
let v2 = NDArray.from([4, 5, 6])
let dot = v1.dot(v2)    # 32
```

### Reductions

```valkyrie
let a = NDArray.random([3, 4, 5])

# Basic reductions
let sum = a.sum()               # Sum of all elements
let mean = a.mean()             # Mean of all elements
let max = a.max()               # Maximum value
let min = a.min()               # Minimum value
let std = a.std()               # Standard deviation

# Along axis
let sum_axis0 = a.sum(axis: 0)  # Shape: (4, 5)
let sum_axis1 = a.sum(axis: 1)  # Shape: (3, 5)
let sum_axis2 = a.sum(axis: 2)  # Shape: (3, 4)

# Keep dimensions
let sum_keep = a.sum(axis: 0, keepdims: true)  # Shape: (1, 4, 5)
```

### Broadcasting

```valkyrie
let a = NDArray.ones([3, 4])
let b = NDArray.from([1, 2, 3, 4])

# b is broadcast from (4,) to (3, 4)
let result = a + b

# Broadcasting rules
let c = NDArray.ones([3, 1])
let d = NDArray.ones([1, 4])
let e = c + d  # Result shape: (3, 4)
```

## Shape Manipulation

```valkyrie
let a = NDArray.arange(12)

# Reshape
let b = a.reshape([3, 4])
let c = a.reshape([2, 2, 3])

# Flatten
let flat = b.flatten()    # Shape: (12,)

# Squeeze (remove dimensions of size 1)
let squeezed = a.reshape([1, 3, 4, 1]).squeeze()  # Shape: (3, 4)

# Expand dimensions
let expanded = a.expand_dims(0)  # Add dimension at position 0

# Transpose
let t = b.transpose()           # Shape: (4, 3)
let t2 = b.transpose([1, 0])    # Same as above
```

## Mathematical Functions

```valkyrie
let a = NDArray.random([3, 3])

# Trigonometric
let sin_a = a.sin()
let cos_a = a.cos()
let tan_a = a.tan()

# Exponential and logarithm
let exp_a = a.exp()
let log_a = a.log()
let log10_a = a.log10()

# Rounding
let floor_a = a.floor()
let ceil_a = a.ceil()
let round_a = a.round()

# Other
let sqrt_a = a.sqrt()
let abs_a = a.abs()
let sign_a = a.sign()
```

## Linear Algebra

```valkyrie
using std::linalg

let a = NDArray.random([3, 3])

# Determinant
let det = linalg.det(a)

# Inverse
let inv = linalg.inv(a)

# Eigenvalues and eigenvectors
let (eigenvalues, eigenvectors) = linalg.eig(a)

# Singular Value Decomposition
let (u, s, vh) = linalg.svd(a)

# QR decomposition
let (q, r) = linalg.qr(a)

# Solve linear system Ax = b
let b = NDArray.from([1, 2, 3])
let x = linalg.solve(a, b)

# Least squares
let (solution, residuals, rank, s) = linalg.lstsq(a, b)
```

## GPU Acceleration

```valkyrie
# Move array to GPU
let gpu_array = a.to_device(Device.gpu())

# Operations automatically run on GPU
let result = gpu_array @ gpu_array.T

# Move back to CPU
let cpu_result = result.to_device(Device.cpu())
```

## Memory Efficiency

### Views vs Copies

```valkyrie
# Views share memory
let a = NDArray.arange(12).reshape([3, 4])
let view = a[.., 0..2]   # View, no copy
let copy = a.copy()       # Explicit copy

# Check if view
print(view.is_view())     # true
```

### Memory Mapping

```valkyrie
# Memory-map large file
let mmap = NDArray.memmap("large_array.bin", shape: [10000, 10000])

# Access without loading entire file
let slice = mmap[0..100, 0..100]
```

## Integration with ML

```valkyrie
# Convert to/from ML tensors
let tensor = a.to_tensor()          # To ML framework tensor
let array = tensor.to_ndarray()     # Back to NDArray

# Batch operations
let batch = NDArray.stack([a, b, c], axis: 0)  # Stack along new axis
let unstacked = batch.unstack(axis: 0)          # Split along axis
```

## Best Practices

1. **Use views when possible**: Avoid unnecessary copies
2. **Prefer built-in operations**: Faster than Python-style loops
3. **Use appropriate dtype**: f32 for most ML, f64 for precision
4. **Leverage broadcasting**: Write cleaner, more efficient code
5. **Profile memory usage**: Large arrays can consume significant memory
