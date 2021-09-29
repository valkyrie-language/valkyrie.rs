# Heterogeneous Computing

Valkyrie provides seamless support for heterogeneous computing, allowing code to run efficiently on CPUs, GPUs, and other accelerators.

## Overview

Heterogeneous computing in Valkyrie is built on:
- **Unified Memory Model**: Transparent data sharing between devices
- **Kernel Abstraction**: Write once, run anywhere
- **Automatic Offload**: Intelligent workload distribution

## Device Management

### Query Available Devices

```valkyrie
using std::compute

micro list_devices() {
    let devices = compute::devices()
    
    for device in devices {
        print("Device: ${device.name}")
        print("  Type: ${device.type}")  # CPU, GPU, TPU, etc.
        print("  Memory: ${device.memory / 1024^3} GB")
        print("  Compute Units: ${device.compute_units}")
    }
}
```

### Select Device

```valkyrie
# Use specific device
let gpu = compute::devices()
    .find { $.type == DeviceType.GPU }
    .expect("No GPU found")

# Or use default device selection
let device = compute::default_device()
```

## Kernel Definition

### Basic Kernel

```valkyrie
# Define a kernel that runs on GPU
kernel vector_add(a: [f32], b: [f32], c: [mut f32]) {
    let i = global_id(0)
    if i < a.length {
        c[i] = a[i] + b[i]
    }
}

# Launch kernel
micro run_vector_add() {
    let n = 1000000
    let a = [f32].random(n)
    let b = [f32].random(n)
    let c = [f32].zeros(n)
    
    # Execute on GPU
    let queue = device.create_queue()
    queue.execute(vector_add, (a, b, c), global_size: n)
    
    # Results are automatically synchronized
    print("Result[0]: ${c[0]}")
}
```

### Kernel with Local Memory

```valkyrie
kernel matrix_multiply(
    a: [f32], b: [f32], c: [mut f32],
    M: usize, N: usize, K: usize
) {
    local tile_a: [[f32, 16], 16]
    local tile_b: [[f32, 16], 16]
    
    let row = global_id(0)
    let col = global_id(1)
    let local_row = local_id(0)
    let local_col = local_id(1)
    
    let mut sum = 0.0
    
    for t in 0..(K / 16) {
        # Load tiles into local memory
        tile_a[local_row][local_col] = a[row * K + t * 16 + local_col]
        tile_b[local_row][local_col] = b[(t * 16 + local_row) * N + col]
        
        barrier()
        
        # Compute partial product
        for k in 0..16 {
            sum += tile_a[local_row][k] * tile_b[k][local_col]
        }
        
        barrier()
    }
    
    c[row * N + col] = sum
}
```

## Memory Management

### Unified Memory

```valkyrie
# Data automatically migrates between devices
let data = compute::UnifiedArray⟨f32⟩::zeros(1000000)

# Access on CPU
for i in 0..100 {
    data[i] = i as f32
}

# Process on GPU
queue.execute(process_kernel, (data,), global_size: data.length)

# Results available on CPU automatically
print("Result: ${data[0]}")
```

### Explicit Memory Transfer

```valkyrie
# For performance-critical code
let host_data = [f32].random(1000000)
let device_buffer = device.create_buffer(host_data.length)

# Copy to device
device_buffer.write(host_data)

# Execute kernel
queue.execute(kernel, (device_buffer,), global_size: host_data.length)

# Copy back to host
let result = device_buffer.read()
```

## Parallel Patterns

### Map

```valkyrie
# Apply function to each element
let squared = data.gpu_map { $ * $ }
```

### Reduce

```valkyrie
# Parallel reduction
let sum = data.gpu_reduce(0.0) { $1 + $2 }
let max = data.gpu_reduce(f32::min) { f32::max($1, $2) }
```

### Scan (Prefix Sum)

```valkyrie
# Inclusive scan
let prefix_sums = data.gpu_scan { $1 + $2 }

# Exclusive scan
let exclusive_sums = data.gpu_scan_exclusive(0) { $1 + $2 }
```

### Filter

```valkyrie
# Parallel filter
let positives = data.gpu_filter { $ > 0 }
```

## Automatic Parallelization

Valkyrie can automatically parallelize suitable code:

```valkyrie
# This loop is automatically parallelized on GPU
micro parallel_computation(data: [mut f32]) {
    parallel for i in 0..data.length {
        data[i] = complex_calculation(data[i])
    }
}
```

## Multi-Device Computing

### Data Parallelism

```valkyrie
micro multi_gpu_compute(data: [f32]) -> [f32] {
    let gpus = compute::devices().filter { $.type == DeviceType.GPU }
    
    # Split data across GPUs
    let chunk_size = data.length / gpus.length
    let futures = []
    
    for (i, gpu) in gpus.enumerate() {
        let start = i * chunk_size
        let end = if i == gpus.length - 1 { data.length } else { start + chunk_size }
        let chunk = data[start..end]
        
        futures.push(async {
            let queue = gpu.create_queue()
            let result = queue.execute(process_kernel, (chunk,), global_size: chunk.length)
            result
        })
    }
    
    # Combine results
    let results = Future.join_all(futures).await
    results.concat()
}
```

### Model Parallelism

```valkyrie
# Split model across devices for large neural networks
micro model_parallel_forward(input: Tensor) -> Tensor {
    let gpu0 = compute::devices()[0]
    let gpu1 = compute::devices()[1]
    
    # First half of model on GPU 0
    let hidden = gpu0.execute(layer1_forward, (input,))
    
    # Transfer to GPU 1
    let transferred = hidden.to_device(gpu1)
    
    # Second half on GPU 1
    let output = gpu1.execute(layer2_forward, (transferred,))
    
    output
}
```

## Performance Optimization

### Kernel Optimization

```valkyrie
kernel optimized_kernel(data: [mut f32]) {
    # Use vector types for better memory access
    let vec_data = data.as_vector4()
    
    let i = global_id(0)
    if i < vec_data.length {
        # Process 4 elements at once
        vec_data[i] = vec_data[i] * 2.0
    }
}
```

### Memory Coalescing

```valkyrie
kernel coalesced_access(data: [f32], output: [mut f32]) {
    # Good: consecutive threads access consecutive memory
    let i = global_id(0)
    output[i] = data[i] * 2.0
}

kernel strided_access(data: [f32], output: [mut f32], stride: usize) {
    # Bad: strided access pattern
    let i = global_id(0)
    output[i] = data[i * stride] * 2.0  # Avoid this pattern
}
```

## Integration with ML

```valkyrie
# Use GPU acceleration for machine learning
micro train_neural_network(model: NeuralNetwork, data: Dataset) {
    let gpu = compute::devices().find { $.type == DeviceType.GPU }
    let trainer = Trainer.new(model, device: gpu)
    
    for epoch in 0..100 {
        for batch in data.batches(32) {
            # Forward and backward pass on GPU
            let loss = trainer.train_step(batch)
        }
        
        print("Epoch ${epoch}: loss = ${trainer.current_loss}")
    }
}
```

## Best Practices

1. **Minimize data transfer**: Keep data on device as long as possible
2. **Use appropriate precision**: f16 or bf16 for ML workloads
3. **Batch operations**: Combine multiple operations into single kernel
4. **Profile before optimizing**: Use profiling tools to identify bottlenecks
5. **Consider memory access patterns**: Coalesced access is crucial for performance
