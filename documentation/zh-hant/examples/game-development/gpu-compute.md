# GPU 計算與並行編程

Valkyrie 提供強大的 GPU 計算能力，支持通用 GPU 編程 (GPGPU)，可以利用現代显卡的並行計算能力進行高性能計算任務。

## 計算着色器基礎

### 工作組和線程模型

```valkyrie
@compute(workgroup_size = [256, 1, 1])
micro add_arrays(id: ComputeInput, input_a: [f32], input_b: [f32], output: [mut f32]) {
    let global_id = id.global_invocation_id.x
    if global_id < input_a.length {
        output[global_id] = input_a[global_id] + input_b[global_id]
    }
}

# 2D 工作組範例
@compute(workgroup_size = [8, 8, 1])  # 8x8 = 64個線程
micro image_process_compute(id: ComputeInput) {
    let coords = id.global_invocation_id.xy
    let local_coords = id.local_invocation_id.xy
    
    # 處理圖像像素
    process_pixel(coords)
}

# 3D 工作組範例
@compute(workgroup_size = [4, 4, 4])  # 4x4x4 = 64個線程
micro volume_compute(id: ComputeInput) {
    let coords = id.global_invocation_id.xyz
    
    # 處理體素數據
    process_voxel(coords)
}
```

### 存儲緩衝區

```valkyrie
# 只讀存儲緩衝區
@group(0) @binding(0)
let input_data: storage⟨[f32], read⟩

# 可讀寫存儲緩衝區
@group(0) @binding(1)
let output_data: storage⟨[f32], read_write⟩

# 結構化數據
structure ParticleData {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    life: f32,
}

@group(0) @binding(2)
let particles: storage⟨[ParticleData], read_write⟩

# Uniform 緩衝區（常量數據）
@group(1) @binding(0)
structure ComputeParams {
    delta_time: f32,
    particle_count: u32,
    gravity: Vec3,
    damping: f32,
}
```

## 並行算法實現

### 並行归约 (Parallel Reduction)

```valkyrie
# 共享內存用於工作組內通信
@compute(workgroup_size = [256, 1, 1])
micro parallel_sum(id: ComputeInput) {
    # 共享內存聲明
    @workgroup let shared_data: array<f32, 256>
    
    let thread_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    # 加載數據到共享內存
    if global_id < input_data.length {
        shared_data[thread_id] = input_data[global_id]
    } else {
        shared_data[thread_id] = 0.0
    }
    
    # 同步工作組內所有線程
    workgroup_barrier()
    
    # 並行归约
    let mut stride = 128u32
    while stride > 0 {
        if thread_id < stride {
            shared_data[thread_id] += shared_data[thread_id + stride]
        }
        workgroup_barrier()
        stride >>= 1
    }
    
    # 將結果寫入輸出緩衝區
    if thread_id == 0 {
        output_data[id.workgroup_id.x] = shared_data[0]
    }
}

# 完整的归约實現
structure ParallelReduction {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    input_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer
    
    micro new(device: wgpu::Device, queue: wgpu::Queue) -> ParallelReduction {
        let shader = compile_valkyrie_shader(include_str!("parallel_sum.val"))
        let compute_pipeline = create_compute_pipeline(device, shader)
        
        ParallelReduction {
            device,
            queue,
            compute_pipeline,
            input_buffer: create_storage_buffer(device, 1024 × 1024 × 4), # 1M floats
            output_buffer: create_storage_buffer(device, 4096 × 4), # 4K floats
            staging_buffer: create_staging_buffer(device, 4096 × 4),
        }
    }
    
    micro compute_sum(mut self, data: [f32]) -> f32 {
        # 上传數據
        self.queue.write_buffer(self.input_buffer, 0, bytemuck::cast_slice(data))
        
        let mut encoder = self.device.create_command_encoder()
        
        # 第一階段：並行归约
        {
            let mut compute_pass = encoder.begin_compute_pass()
            compute_pass.set_pipeline(self.compute_pipeline)
            compute_pass.set_bind_group(0, bind_group, [])
            
            let workgroups = (data.length + 255) / 256
            compute_pass.dispatch_workgroups(workgroups, 1, 1)
        }
        
        # 複製結果到 staging buffer
        encoder.copy_buffer_to_buffer(
            self.output_buffer, 0,
            self.staging_buffer, 0,
            workgroups × 4,
        )
        
        self.queue.submit([encoder.finish()])
        
        # 讀取結果
        let slice = self.staging_buffer.slice(..(workgroups × 4))
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel()
        slice.map_async(wgpu::MapMode::Read) {
            sender.send($).unwrap()
        }
        
        self.device.poll(wgpu::Maintain::Wait)
        receiver.receive().await.unwrap().unwrap()
        
        let data = slice.get_mapped_range()
        let result: [f32] = bytemuck::cast_slice(data)
        
        # CPU 端完成最終归约
        result.iter().sum()
    }
}
```

### 並行前綴和 (Parallel Prefix Sum)

```valkyrie
# Blelloch 扫描算法實現
@compute(workgroup_size = [256, 1, 1])
micro prefix_sum_up_sweep(id: ComputeInput, params: ScanParams) {
    @workgroup let shared_data: array<u32, 512>  # 雙倍大小用於 padding
    
    let thread_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    # 加載數據
    let ai = thread_id
    let bi = thread_id + 256
    
    shared_data[ai] = if global_id < input_data.length { input_data[global_id] } else { 0 }
    shared_data[bi] = if global_id + 256 < input_data.length { input_data[global_id + 256] } else { 0 }
    
    # Up-sweep 阶段
    let mut offset = 1u32
    let mut d = 256u32
    while d > 0 {
        workgroup_barrier()
        
        if thread_id < d {
            let ai = offset × (2 × thread_id + 1) - 1
            let bi = offset × (2 × thread_id + 2) - 1
            shared_data[bi] += shared_data[ai]
        }
        
        offset ×= 2
        d >>= 1
    }
    
    # 清零最後一個元素
    if thread_id == 0 {
        shared_data[511] = 0
    }
    
    # Down-sweep 阶段
    d = 1
    while d < 512 {
        offset /= 2
        workgroup_barrier()
        
        if thread_id < d {
            let ai = offset × (2 × thread_id + 1) - 1
            let bi = offset × (2 × thread_id + 2) - 1
            
            let temp = shared_data[ai]
            shared_data[ai] = shared_data[bi]
            shared_data[bi] += temp
        }
        
        d ×= 2
    }
    
    workgroup_barrier()
    
    # 寫回結果
    if global_id < output_data.length {
        output_data[global_id] = shared_data[ai]
    }
    if global_id + 256 < output_data.length {
        output_data[global_id + 256] = shared_data[bi]
    }
}
```

### 並行排序

```valkyrie
# 雙调排序 (Bitonic Sort)
@compute(workgroup_size = [256, 1, 1])
micro bitonic_sort(id: ComputeInput, params: SortParams) {
    var<workgroup> shared_data: array<u32, 256>
    
    let thread_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    # 加載數據
    shared_data[thread_id] = if global_id < input_data.length {
        input_data[global_id]
    } else {
        0xFFFFFFFF  # 最大值作為填充
    }
    
    workgroup_barrier()
    
    # 雙调排序主循環
    let mut k = 2u32
    while k <= 256 {
        let mut j = k / 2
        while j > 0 {
            let ixj = thread_id ^ j
            
            if ixj > thread_id {
                let ascending = (thread_id & k) == 0
                
                if (shared_data[thread_id] > shared_data[ixj]) == ascending {
                    # 交換元素
                    let temp = shared_data[thread_id]
                    shared_data[thread_id] = shared_data[ixj]
                    shared_data[ixj] = temp
                }
            }
            
            workgroup_barrier()
            j /= 2
        }
        k ×= 2
    }
    
    # 寫回排序後的數據
    if global_id < output_data.length {
        output_data[global_id] = shared_data[thread_id]
    }
}

# 基數排序實現
@compute(workgroup_size = [256, 1, 1])
micro radix_sort_count(id: ComputeInput, params: RadixSortParams) {
    var<workgroup> local_histogram: array<u32, 16>  # 4-bit 基數
    
    let thread_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    # 初始化本地直方圖
    if thread_id < 16 {
        local_histogram[thread_id] = 0
    }
    
    workgroup_barrier()
    
    # 計算本地直方圖
    if global_id < input_data.length {
        let value = input_data[global_id]
        let digit = (value >> (params.bit_shift × 4)) & 0xF
        atomicAdd(local_histogram[digit], 1u)
    }
    
    workgroup_barrier()
    
    # 將本地直方圖寫入全局直方圖
    if thread_id < 16 {
        let workgroup_id = id.workgroup_id.x
        global_histogram[workgroup_id × 16 + thread_id] = local_histogram[thread_id]
    }
}
```

## 物理模擬

### N-Body 粒子模擬

```valkyrie
# N-Body 重力模擬
structure Body {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    _padding: f32  # 對齊到 16 字節
}

@group(0) @binding(0)
let bodies: storage⟨[Body], read_write⟩

@group(0) @binding(1)
structure NBodyParams {
    body_count: u32,
    Δt: f32,
    ε: f32,  # 软化參數避免奇點
    damping: f32
}

@compute(workgroup_size = [64, 1, 1])
micro n_body_simulation(id: ComputeInput, params: NBodyParams) {
    let index = id.global_invocation_id.x
    if index >= params.body_count {
        return
    }
    
    let body = bodies[index]
    let mut 𝐅 = Vec3(0.0, 0.0, 0.0)
    
    # 計算所有其他物體的引力
    loop i in 0..params.body_count {
        if i != index {
            let other = bodies[i]
            let r = other.position - body.position
            let dist_sq = dot(r, r) + params.ε ^ 2
            let dist = sqrt(dist_sq)
            let force_magnitude = other.mass / (dist_sq × dist)
            𝐅 += r × force_magnitude
        }
    }
    
    # 更新速度和位置
    let 𝐚 = 𝐅 / body.mass
    let new_𝐯 = body.velocity + 𝐚 × params.Δt
    let new_𝐩 = body.position + new_𝐯 × params.Δt
    
    # 應用阻尼
    let damped_𝐯 = new_𝐯 × params.damping
    
    # 寫回結果
    bodies[index].velocity = damped_𝐯
    bodies[index].position = new_𝐩
}

# 優化版本：使用共享內存
@compute(workgroup_size = [64, 1, 1])
micro n_body_optimized(id: ComputeInput, params: NBodyParams) {
    @workgroup let shared_bodies: array<Body, 64>
    
    let thread_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    if global_id >= params.body_count {
        return
    }
    
    let body = bodies[global_id]
    let mut force = Vec3(0.0, 0.0, 0.0)
    
    # 分塊處理
    let num_tiles = (params.body_count + 63) / 64
    
    loop tile in 0..num_tiles {
        let tile_start = tile × 64
        let shared_index = tile_start + thread_id
        
        # 加載一個 tile 的數據到共享內存
        if shared_index < params.body_count {
            shared_bodies[thread_id] = bodies[shared_index]
        }
        
        workgroup_barrier()
        
        # 計算與當前 tile 中所有物體的相互作用
        let tile_size = min(64u32, params.body_count - tile_start)
        loop i in 0..tile_size {
            if tile_start + i != global_id {
                let other = shared_bodies[i]
                let r = other.position - body.position
                let dist_sq = dot(r, r) + params.softening ^ 2
                let dist = sqrt(dist_sq)
                let force_magnitude = other.mass / (dist_sq × dist)
                force += r × force_magnitude
            }
        }
        
        workgroup_barrier()
    }
    
    # 更新物體狀態
    let acceleration = force / body.mass
    bodies[global_id].velocity += acceleration × params.delta_time
    bodies[global_id].position += bodies[global_id].velocity × params.delta_time
    bodies[global_id].velocity ×= params.damping
}
```

### 流體模擬 (SPH)

```valkyrie
# 光滑粒子流體动力學 (Smoothed Particle Hydrodynamics)
structure FluidParticle {
    position: Vec3,
    velocity: Vec3,
    density: f32,
    pressure: f32,
    force: Vec3,
    _padding: f32
}

@group(0) @binding(0)
let particles: storage⟨[FluidParticle], read_write⟩

@group(0) @binding(1)
structure SPHParams {
    particle_count: u32,
    rest_density: f32,
    gas_constant: f32,
    viscosity: f32,
    smoothing_radius: f32,
    particle_mass: f32,
    delta_time: f32,
    gravity: Vec3
}

# SPH 核函數
micro poly6_kernel(r: f32, h: f32) -> f32 {
    if r >= h {
        return 0.0
    }
    
    let h2 = h ^ 2
    let h6 = h ^ 6
    let h9 = h ^ 9
    let r2 = r ^ 2
    
    # Poly6 Kernel (用於密度計算)
    let factor = 315.0 / (64.0 × PI × h9) × (h2 - r2) ^ 3
    return factor
}

micro spiky_gradient_kernel(r_vec: Vec3, h: f32) -> Vec3 {
    let r = length(r_vec)
    if r >= h || r == 0.0 {
        return Vec3(0.0, 0.0, 0.0)
    }
    
    let h6 = h ^ 6
    # Spiky Kernel (用於压力梯度計算)
    let factor = -45.0 / (PI × h6) × (h - r) ^ 2 / r
    
    return r_vec × factor
}

micro viscosity_laplacian_kernel(r: f32, h: f32) -> f32 {
    if r >= h {
        return 0.0
    }
    
    let h6 = h ^ 6
    # Viscosity Kernel (用於拉普拉斯算子計算)
    let factor = 45.0 / (PI × h6) × (h - r)
    return factor
}

# 密度計算
@compute(workgroup_size = [64, 1, 1])
micro compute_density(id: ComputeInput, params: SPHParams) {
    let index = id.global_invocation_id.x
    if index >= params.particle_count {
        return
    }
    
    let particle = particles[index]
    let mut density = 0.0
    
    # 計算密度
    loop i in 0..params.particle_count {
        let other = particles[i]
        let r = length(particle.position - other.position)
        
        if r < params.smoothing_radius {
            density += params.particle_mass × poly6_kernel(r, params.smoothing_radius)
        }
    }
    
    particles[index].density = density
    particles[index].pressure = params.gas_constant × (density - params.rest_density)
}

# 力計算
@compute(workgroup_size = [64, 1, 1])
micro compute_forces(id: ComputeInput, params: SPHParams) {
    let index = id.global_invocation_id.x
    if index >= params.particle_count {
        return
    }
    
    let particle = particles[index]
    let mut pressure_force = Vec3(0.0, 0.0, 0.0)
    let mut viscosity_force = Vec3(0.0, 0.0, 0.0)
    
    loop i in 0..params.particle_count {
        if i == index {
            continue
        }
        
        let other = particles[i]
        let r_vec = particle.position - other.position
        let r = length(r_vec)
        
        if r < params.smoothing_radius && r > 0.0 {
            # 压力力
            let pressure_gradient = spiky_gradient_kernel(r_vec, params.smoothing_radius)
            pressure_force -= params.particle_mass × 
                (particle.pressure + other.pressure) / (2.0 × other.density) × 
                pressure_gradient
            
            # 粘性力
            let velocity_diff = other.velocity - particle.velocity
            let viscosity_laplacian = viscosity_laplacian_kernel(r, params.smoothing_radius)
            viscosity_force += params.viscosity × params.particle_mass × 
                velocity_diff / other.density × viscosity_laplacian
        }
    }
    
    # 總力 = 压力力 + 粘性力 + 重力
    let total_force = pressure_force + viscosity_force + params.gravity × particle.density
    particles[index].force = total_force
}

# 積分更新
@compute(workgroup_size = [64, 1, 1])
micro integrate_particles(id: ComputeInput, params: SPHParams) {
    let index = id.global_invocation_id.x
    if index >= params.particle_count {
        return
    }
    
    let mut particle = particles[index]
    
    # Leapfrog 積分
    let acceleration = particle.force / particle.density
    particle.velocity += acceleration × params.delta_time
    particle.position += particle.velocity × params.delta_time
    
    # 邊界條件（簡單反弹）
    let boundary = 10.0
    if particle.position.x < -boundary || particle.position.x > boundary {
        particle.velocity.x ×= -0.8
        particle.position.x = clamp(particle.position.x, -boundary, boundary)
    }
    if particle.position.y < -boundary || particle.position.y > boundary {
        particle.velocity.y ×= -0.8
        particle.position.y = clamp(particle.position.y, -boundary, boundary)
    }
    if particle.position.z < -boundary || particle.position.z > boundary {
        particle.velocity.z ×= -0.8
        particle.position.z = clamp(particle.position.z, -boundary, boundary)
    }
    
    particles[index] = particle
}
```

## 機器學習加速

### 矩陣乘法

```valkyrie
# 高效的矩陣乘法實現
@compute(workgroup_size = [16, 16, 1])
micro matrix_multiply(id: ComputeInput, params: MatMulParams) {
    var⟨workgroup⟩ tile_a: array<array<f32, 16>, 16>
    var⟨workgroup⟩ tile_b: array<array<f32, 16>, 16>
    
    let row = id.global_invocation_id.y
    let col = id.global_invocation_id.x
    let local_row = id.local_invocation_id.y
    let local_col = id.local_invocation_id.x
    
    if row >= params.m || col >= params.n {
        return
    }
    
    let mut sum = 0.0
    let num_tiles = (params.k + 15) / 16
    
    loop k in 0..num_tiles {
        # 加載 tile 數據
        tile_a[id.local_invocation_id.y][id.local_invocation_id.x] = 
            matrix_a[row × params.k + (k × 16 + id.local_invocation_id.x)]
        tile_b[id.local_invocation_id.y][id.local_invocation_id.x] = 
            matrix_b[(k × 16 + id.local_invocation_id.y) × params.n + col]
        
        workgroup_barrier()
        
        # 計算 tile 乘法
        loop i in 0..16 {
            sum += tile_a[local_row][i] × tile_b[i][local_col]
        }
        
        workgroup_barrier()
    }
    
    matrix_c[row × params.n + col] = sum
}

# 批量矩陣乘法
@compute(workgroup_size = [8, 8, 4])
micro batch_matrix_multiply(id: ComputeInput, params: BatchMatMulParams) {
    let batch = id.global_invocation_id.z
    let row = id.global_invocation_id.y
    let col = id.global_invocation_id.x
    
    if batch >= params.batch_size || row >= params.m || col >= params.n {
        return
    }
    
    let mut sum = 0.0
    let a_offset = batch × params.m × params.k
    let b_offset = batch × params.k × params.n
    let c_offset = batch × params.m × params.n
    
    loop k in 0..params.k {
        let a_val = matrix_a[a_offset + row × params.k + k]
        let b_val = matrix_b[b_offset + k × params.n + col]
        sum += a_val × b_val
    }
    
    matrix_c[c_offset + row × params.n + col] = sum
}
```

### 卷積神經網絡

```valkyrie
# 2D 卷積層
@compute(workgroup_size = [8, 8, 1])
micro conv2d(id: ComputeInput, params: Conv2DParams) {
    let out_y = id.global_invocation_id.y
    let out_x = id.global_invocation_id.x
    let out_c = id.global_invocation_id.z
    
    if out_y >= params.output_height || out_x >= params.output_width || out_c >= params.output_channels {
        return
    }
    
    let mut sum = 0.0
    
    # 卷積計算
    loop in_c in 0..params.input_channels {
        loop ky in 0..params.kernel_size {
            loop kx in 0..params.kernel_size {
                let in_y = out_y × params.stride + ky - params.padding
                let in_x = out_x × params.stride + kx - params.padding
                
                if in_y >= 0 && in_y < params.input_height && 
                   in_x >= 0 && in_x < params.input_width {
                    
                    let input_idx = in_c × params.input_height × params.input_width + 
                                   in_y × params.input_width + in_x
                    let weight_idx = out_c × params.input_channels × params.kernel_size × params.kernel_size +
                                    in_c × params.kernel_size × params.kernel_size +
                                    ky × params.kernel_size + kx
                    
                    sum += input_data[input_idx] × weights[weight_idx]
                }
            }
        }
    }
    
    # 添加偏置並應用激活函數
    sum += biases[out_c]
    let activated = relu(sum)  # ReLU 激活
    
    let output_idx = out_c × params.output_height × params.output_width + 
                    out_y × params.output_width + out_x
    output_data[output_idx] = activated
}

# 激活函數
micro relu(x: f32) -> f32 {
    max(0.0, x)
}

micro sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + exp(-x))
}

micro tanh_activation(x: f32) -> f32 {
    tanh(x)
}

micro gelu(x: f32) -> f32 {
    0.5 × x × (1.0 + tanh(sqrt(2.0 / PI) × (x + 0.044715 × x ^ 3)))
}

# 批量歸一化
@compute(workgroup_size = [256, 1, 1])
micro batch_norm(id: ComputeInput, params: BatchNormParams) {
    let index = id.global_invocation_id.x
    if index >= params.size {
        return
    }
    
    let channel = index % params.channels
    let mean = running_mean[channel]
    let var = running_var[channel]
    let gamma = scale[channel]
    let beta = shift[channel]
    
    let normalized = (input_data[index] - mean) / sqrt(var + params.epsilon)
    output_data[index] = gamma × normalized + beta
}
```

## 圖像處理

### 高級滤波器

```valkyrie
# 雙邊滤波器
@compute(workgroup_size = [8, 8, 1])
micro bilateral_filter(id: ComputeInput, params: BilateralParams) {
    let coords = id.global_invocation_id.xy
    let dimensions = textureDimensions(input_texture)
    
    if coords.x >= dimensions.x || coords.y >= dimensions.y {
        return
    }
    
    let center_color = textureLoad(input_texture, coords, 0)
    let mut weighted_sum = Vec3(0.0, 0.0, 0.0)
    let mut weight_sum = 0.0
    
    let radius = params.radius
    let sigma_space = params.sigma_space
    let sigma_color = params.sigma_color
    
    loop dy in -radius..=radius {
        loop dx in -radius..=radius {
            let sample_coords = coords + Vec2(dx, dy)
            
            if sample_coords.x >= 0 && sample_coords.x < dimensions.x &&
               sample_coords.y >= 0 && sample_coords.y < dimensions.y {
                
                let sample_color = textureLoad(input_texture, sample_coords, 0)
                
                # 空間權重
                let spatial_dist_sq = f32(dx × dx + dy × dy)
                let spatial_weight = exp(-spatial_dist_sq / (2.0 × sigma_space ^ 2))
                
                # 顏色權重
                let color_dist_sq = length(sample_color.rgb - center_color.rgb) ^ 2
                let color_weight = exp(-color_dist_sq / (2.0 × sigma_color ^ 2))
                
                let weight = spatial_weight × color_weight
                weighted_sum += sample_color.rgb × weight
                weight_sum += weight
            }
        }
    }
    
    let filtered_color = if weight_sum > 0.0 {
        weighted_sum / weight_sum
    } else {
        center_color.rgb
    }
    
    textureStore(output_texture, coords, Vec4(filtered_color, center_color.a))
}

# 非局部均值去噪
@compute(workgroup_size = [8, 8, 1])
micro non_local_means(id: ComputeInput, params: NLMParams) {
    let coords = id.global_invocation_id.xy
    let dimensions = textureDimensions(input_texture)
    
    if coords.x >= dimensions.x || coords.y >= dimensions.y {
        return
    }
    
    let patch_size = params.patch_size
    let search_window = params.search_window
    let h = params.filtering_parameter
    
    let mut weighted_sum = Vec3(0.0, 0.0, 0.0)
    let mut weight_sum = 0.0
    
    # 搜索窗口
    loop sy in -search_window..=search_window {
        loop sx in -search_window..=search_window {
            let search_coords = coords + Vec2(sx, sy)
            
            if search_coords.x >= patch_size && search_coords.x < dimensions.x - patch_size &&
               search_coords.y >= patch_size && search_coords.y < dimensions.y - patch_size {
                
                # 計算补丁相似度
                let mut patch_distance = 0.0
                let mut patch_count = 0
                
                loop py in -patch_size..=patch_size {
                    loop px in -patch_size..=patch_size {
                        let p1 = textureLoad(input_texture, coords + Vec2(px, py), 0)
                        let p2 = textureLoad(input_texture, search_coords + Vec2(px, py), 0)
                        
                        let diff = p1.rgb - p2.rgb
                        patch_distance += dot(diff, diff)
                        patch_count += 1
                    }
                }
                
                patch_distance /= f32(patch_count)
                
                # 計算權重
                let weight = exp(-max(patch_distance - 2.0 × params.noise_variance, 0.0) / (h × h))
                
                let sample_color = textureLoad(input_texture, search_coords, 0)
                weighted_sum += sample_color.rgb × weight
                weight_sum += weight
            }
        }
    }
    
    let denoised_color = if weight_sum > 0.0 {
        weighted_sum / weight_sum
    } else {
        textureLoad(input_texture, coords, 0).rgb
    }
    
    textureStore(output_texture, coords, Vec4(denoised_color, 1.0))
}
```

## 性能優化技巧

### 內存訪問優化

```valkyrie
# 合並內存訪問
@compute(workgroup_size = [32, 1, 1])  # warp size
micro coalesced_access(id: ComputeInput) {
    let thread_id = id.global_invocation_id.x
    
    # 好的訪問模式：連續訪問
    let value = input_data[thread_id]  # 線程 i 訪問元素 i
    
    # 避免的訪問模式：跨步訪問
    # let value = input_data[thread_id × stride]  # 可能导致內存帶宽浪费
    
    output_data[thread_id] = process(value)
}

# 使用共享內存減少全局內存訪問
@compute(workgroup_size = [256, 1, 1])
micro shared_memory_optimization(id: ComputeInput) {
    let local_id = id.local_invocation_id.x
    let global_id = id.global_invocation_id.x
    
    # 聲明共享內存
    @workgroup let shared_cache: array<f32, 256>
    
    # 從全局內存加載到共享內存
    shared_cache[local_id] = input_data[global_id]
    
    workgroup_barrier()
    
    # 现在可以快速訪問共享內存中的數據
    let mut result = 0.0
    loop i in 0..256 {
        result += shared_cache[i] × coefficients[i]
    }
    
    output_data[global_id] = result
}
```

### 分支優化

```valkyrie
# 避免分支分歧
@compute(workgroup_size = [32, 1, 1])
micro branch_optimization(id: ComputeInput) {
    let thread_id = id.global_invocation_id.x
    let value = input_data[thread_id]
    
    # 不好的分支：可能导致 warp 分歧
    # if thread_id % 2 == 0 {
    #     result = expensive_computation_a(value)
    # } else {
    #     result = expensive_computation_b(value)
    # }
    
    # 更好的方法：使用條件表達式
    let condition = thread_id % 2 == 0
    let result_a = expensive_computation_a(value)
    let result_b = expensive_computation_b(value)
    let result = if condition { result_a } else { result_b }
    
    # 或者重新組织算法避免分支
    output_data[thread_id] = result
}
```

### 占用率優化

```valkyrie
# 優化寄存器使用
@compute(workgroup_size = [128, 1, 1])  # 調整工作組大小以平衡占用率
micro register_optimization(id: ComputeInput) {
    let thread_id = id.global_invocation_id.x
    
    # 避免使用过多局部變量（寄存器）
    # 將大數組移到共享內存或重新計算
    
    let value = input_data[thread_id]
    let result = complex_computation(value)  # 內联小函數
    output_data[thread_id] = result
}

# 內存帶宽優化
@compute(workgroup_size = [64, 1, 1])
micro bandwidth_optimization(id: ComputeInput) {
    let thread_id = id.global_invocation_id.x
    
    # 向量化訪問
    let vec4_index = thread_id / 4
    let vec4_data = input_vec4[vec4_index]  # 一次讀取 4 個 f32
    
    # 處理向量數據
    let processed = process_vec4(vec4_data)
    
    output_vec4[vec4_index] = processed
}
```

## 調試和性能分析

### GPU 調試工具

```valkyrie
# 調試緩衝區
@group(2) @binding(0)
let debug_buffer: storage<array<DebugInfo>, write>

structure DebugInfo {
    thread_id: u32,
    value: f32,
    iteration: u32,
    timestamp: u32
}

@compute(workgroup_size = [64, 1, 1])
micro debug_compute(id: ComputeInput) {
    let thread_id = id.global_invocation_id.x
    
    # 記錄調試信息
    debug_buffer[thread_id] = DebugInfo {
        thread_id: thread_id,
        value: input_data[thread_id],
        iteration: 0,
        timestamp: 0  # GPU 時間戳
    }
    
    # 主要計算邏輯
    let result = compute_something(input_data[thread_id])
    
    # 更新調試信息
    debug_buffer[thread_id].value = result
    debug_buffer[thread_id].iteration = 1
    
    output_data[thread_id] = result
}

# 性能计數器
structure GPUPerformanceCounter {
    query_sets: [wgpu::QuerySet],
    timestamp_period: f32
    
    micro new(device: wgpu::Device) -> GPUPerformanceCounter {
        let timestamp_query = device.create_query_set(wgpu::QuerySetDescriptor {
            label: Some("Timestamp Queries"),
            ty: wgpu::QueryType::Timestamp,
            count: 64
        })
        
        GPUPerformanceCounter {
            query_sets: [timestamp_query],
            timestamp_period: queue.get_timestamp_period()
        }
    }
    
    micro measure_compute_time(self, encoder: wgpu::CommandEncoder, compute_pass: impl FnOnce()) -> f64 {
        encoder.write_timestamp(self.query_sets[0], 0)
        
        compute_pass()
        
        encoder.write_timestamp(self.query_sets[0], 1)
        
        # 解析時間戳並返回毫秒
    let timestamps = self.resolve_timestamps()
    f64(timestamps[1] - timestamps[0]) × f64(self.timestamp_period) / 1_000_000.0
}
}
```

## 總結

Valkyrie 的 GPU 計算能力提供了：

1. **現代並行編程模型** - 支持計算着色器和 GPGPU 編程
2. **高性能算法實現** - 並行归约、排序、前綴和等基礎算法
3. **物理模擬加速** - N-Body、SPH 流體模擬等複雜物理計算
4. **機器學習支持** - 矩陣運算、卷積神經網絡等 AI 計算
5. **圖像處理優化** - 高級滤波器和圖像算法的 GPU 實現
6. **性能優化工具** - 內存訪問優化、分支優化、調試工具等

通過 Valkyrie 的 GPU 計算框架，開發者可以充分利用現代显卡的並行計算能力，實現高性能的科學計算、遊戲物理、機器學習和圖像處理應用。