# Graphics Programming and Shader Development

Valkyrie natively supports graphics programming, allowing direct shader code writing as an alternative to GLSL/HLSL, with complete wgpu integration. This enables developers to write high-performance rendering code in a unified language environment.

## Core Concepts

### Rendering Pipeline

```valkyrie
# Define rendering pipeline
class RenderPipeline {
    device: Device,
    pipeline: wgpu::RenderPipeline,
    bind_groups: [BindGroup],
}

# Vertex data structure
class Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    color: Color,
}

# Uniform buffer
class UniformBuffer {
    view_matrix: Mat4,
    projection_matrix: Mat4,
    model_matrix: Mat4,
    time: f32,
}
```

### Shader Basics

```valkyrie
# Valkyrie can directly write shaders
shader basic_vertex {
    struct VertexInput {
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) color: vec4<f32>,
    }
    
    struct VertexOutput {
        @builtin(position) clip_position: vec4<f32>,
        @location(0) world_position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) color: vec4<f32>,
    }
    
    @group(0) @binding(0) var<uniform> uniforms: UniformBuffer;
    
    @vertex
    micro main(input: VertexInput) -> VertexOutput {
        var output: VertexOutput;
        let world_pos = uniforms.model_matrix * vec4<f32>(input.position, 1.0);
        output.world_position = world_pos.xyz;
        output.clip_position = uniforms.projection_matrix * uniforms.view_matrix * world_pos;
        output.normal = (uniforms.model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
        output.uv = input.uv;
        output.color = input.color;
        return output;
    }
}

shader basic_fragment {
    @group(0) @binding(0) var<uniform> uniforms: UniformBuffer;
    @group(0) @binding(1) var<uniform> light: Light;
    @group(1) @binding(0) var texture: texture_2d<f32>;
    @group(1) @binding(1) var sampler: sampler;
    
    @fragment
    micro main(
        @location(0) world_position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) color: vec4<f32>
    ) -> @location(0) vec4<f32> {
        let n = normalize(normal);
        let l = normalize(light.position - world_position);
        let diffuse = max(dot(n, l), 0.0) * light.color;
        
        let ambient = vec3<f32>(0.1, 0.1, 0.1);
        let texture_color = textureSample(texture, sampler, uv);
        
        let final_color = (ambient + diffuse) * texture_color.rgb * color.rgb;
        return vec4<f32>(final_color, color.a);
    }
}
```

## Advanced Rendering Techniques

### Physically Based Rendering (PBR)

```valkyrie
shader pbr_fragment {
    struct PBRMaterial {
        albedo: vec3<f32>,
        metallic: f32,
        roughness: f32,
        ao: f32,
    }
    
    @group(0) @binding(0) var<uniform> camera: Camera;
    @group(1) @binding(0) var albedo_map: texture_2d<f32>;
    @group(1) @binding(1) var normal_map: texture_2d<f32>;
    @group(1) @binding(2) var metallic_map: texture_2d<f32>;
    @group(1) @binding(3) var roughness_map: texture_2d<f32>;
    @group(1) @binding(4) var ao_map: texture_2d<f32>;
    @group(1) @binding(5) var sampler: sampler;
    
    micro distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
        let a = roughness * roughness;
        let a2 = a * a;
        let ndh = max(dot(n, h), 0.0);
        let ndh2 = ndh * ndh;
        
        let nom = a2;
        let denom = ndh2 * (a2 - 1.0) + 1.0;
        return nom / (PI * denom * denom);
    }
    
    micro geometry_schlick_ggx(ndv: f32, roughness: f32) -> f32 {
        let r = roughness + 1.0;
        let k = (r * r) / 8.0;
        return ndv / (ndv * (1.0 - k) + k);
    }
    
    micro geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
        let ndv = max(dot(n, v), 0.0);
        let ndl = max(dot(n, l), 0.0);
        let ggx1 = geometry_schlick_ggx(ndv, roughness);
        let ggx2 = geometry_schlick_ggx(ndl, roughness);
        return ggx1 * ggx2;
    }
    
    micro fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
        return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
    }
    
    @fragment
    micro main(
        @location(0) world_position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>
    ) -> @location(0) vec4<f32> {
        let n = normalize(normal);
        let v = normalize(camera.position - world_position);
        
        # Sample material properties
        let albedo = textureSample(albedo_map, sampler, uv).rgb;
        let metallic = textureSample(metallic_map, sampler, uv).r;
        let roughness = textureSample(roughness_map, sampler, uv).r;
        let ao = textureSample(ao_map, sampler, uv).r;
        
        let f0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);
        
        var lo = vec3<f32>(0.0, 0.0, 0.0);
        
        # Calculate lighting for each light source
        for (var i = 0u; i < light_count; i++) {
            let l = normalize(lights[i].position - world_position);
            let h = normalize(v + l);
            let distance = length(lights[i].position - world_position);
            let attenuation = 1.0 / (distance * distance);
            let radiance = lights[i].color * attenuation;
            
            # Cook-Torrance BRDF
            let ndf = distribution_ggx(n, h, roughness);
            let g = geometry_smith(n, v, l, roughness);
            let f = fresnel_schlick(max(dot(h, v), 0.0), f0);
            
            let numerator = ndf * g * f;
            let denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0);
            let specular = numerator / max(denominator, 0.001);
            
            let ks = f;
            let kd = vec3<f32>(1.0, 1.0, 1.0) - ks;
            kd *= 1.0 - metallic;
            
            let ndl = max(dot(n, l), 0.0);
            lo += (kd * albedo / PI + specular) * radiance * ndl;
        }
        
        let ambient = vec3<f32>(0.03, 0.03, 0.03) * albedo * ao;
        let color = ambient + lo;
        
        # HDR tonemapping
        let mapped = color / (color + vec3<f32>(1.0, 1.0, 1.0));
        
        return vec4<f32>(mapped, 1.0);
    }
}
```

### Post-Processing Effects

```valkyrie
shader bloom {
    @group(0) @binding(0) var input_texture: texture_2d<f32>;
    @group(0) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
    @group(0) @binding(2) var sampler: sampler;
    
    @fragment
    micro main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
        let color = textureSample(input_texture, sampler, uv);
        
        # Brightness threshold
        let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if (brightness > 1.0) {
            return vec4<f32>(color.rgb, 1.0);
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }
}

shader gaussian_blur {
    layout(local_size_x = 16, local_size_y = 16) in;
    
    @group(0) @binding(0) var input_texture: texture_2d<f32>;
    @group(0) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
    @group(0) @binding(2) var sampler: sampler;
    @group(0) @binding(3) var<uniform> direction: vec2<f32>;
    
    @compute
    micro main(@builtin(global_invocation_id) global_id: vec3<u32>) {
        let texel_size = 1.0 / vec2<f32>(textureDimensions(input_texture));
        let uv = (vec2<f32>(global_id.xy) + 0.5) * texel_size;
        
        let weights = [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216];
        
        var result = textureSample(input_texture, sampler, uv) * weights[0];
        
        for (var i = 1; i < 5; i++) {
            let offset = direction * texel_size * f32(i);
            result += textureSample(input_texture, sampler, uv + offset) * weights[i];
            result += textureSample(input_texture, sampler, uv - offset) * weights[i];
        }
        
        textureStore(output_texture, global_id.xy, result);
    }
}
```

### Shadow Mapping

```valkyrie
shader shadow_map_vertex {
    @group(0) @binding(0) var<uniform> light_view_projection: mat4x4<f32>;
    @group(0) @binding(1) var<uniform> model_matrix: mat4x4<f32>;
    
    @vertex
    micro main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
        return light_view_projection * model_matrix * vec4<f32>(position, 1.0);
    }
}

shader shadow_mapping_fragment {
    @group(0) @binding(0) var shadow_map: texture_depth_2d;
    @group(0) @binding(1) var shadow_sampler: sampler_comparison;
    @group(0) @binding(2) var<uniform> light: Light;
    
    micro calculate_shadow(
        frag_pos_light_space: vec4<f32>,
        normal: vec3<f32>,
        light_dir: vec3<f32>
    ) -> f32 {
        let proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
        let proj_coords = proj_coords * 0.5 + 0.5;
        
        if (proj_coords.z > 1.0) {
            return 0.0;
        }
        
        let current_depth = proj_coords.z;
        let bias = max(0.05 * (1.0 - dot(normal, light_dir)), 0.005);
        
        # PCF (Percentage Closer Filtering)
        var shadow = 0.0;
        let texel_size = 1.0 / vec2<f32>(textureDimensions(shadow_map));
        
        for (var x = -1; x <= 1; x++) {
            for (var y = -1; y <= 1; y++) {
                let pcf_depth = textureSampleCompare(
                    shadow_map,
                    shadow_sampler,
                    proj_coords.xy + vec2<f32>(f32(x), f32(y)) * texel_size,
                    current_depth - bias
                );
                shadow += pcf_depth;
            }
        }
        
        return shadow / 9.0;
    }
    
    @fragment
    micro main(
        @location(0) world_position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) frag_pos_light_space: vec4<f32>
    ) -> @location(0) vec4<f32> {
        let n = normalize(normal);
        let l = normalize(light.position - world_position);
        
        let shadow = calculate_shadow(frag_pos_light_space, n, l);
        let lighting = (1.0 - shadow) * max(dot(n, l), 0.0);
        
        return vec4<f32>(vec3<f32>(lighting), 1.0);
    }
}
```

## wgpu Integration

### Device and Surface Creation

```valkyrie
use valkyrie::graphics::wgpu::*

class GraphicsContext {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface,
    config: SurfaceConfiguration,
}

imply GraphicsContext {
    micro async new(window: Window) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        })
        
        let surface = unsafe { instance.create_surface(&window) }
        
        let adapter = instance.request_adapter(RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap()
        
        let (device, queue) = adapter.request_device(
            DeviceDescriptor {
                label: Some("Valkyrie Device"),
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        ).await.unwrap()
        
        let window_size = window.inner_size()
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: [],
        }
        
        surface.configure(&device, &config)
        
        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
        }
    }
    
    micro resize(mut self, width: u32, height: u32) {
        self.config.width = width
        self.config.height = height
        self.surface.configure(&self.device, &self.config)
    }
    
    micro render(self, render_callback: micro(GraphicsContext, RenderPass)) {
        let output = self.surface.get_current_texture().unwrap()
        let view = output.texture.create_view(&TextureViewDescriptor::default())
        
        let mut encoder = self.device.create_command_encoder(CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
        
        {
            let mut render_pass = encoder.begin_render_pass(RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: [
                    RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            })
            
            render_callback(self, render_pass)
        }
        
        self.queue.submit([encoder.finish()])
        output.present()
    }
}
```

## Performance Optimization

### Instanced Rendering

```valkyrie
shader instanced_vertex {
    struct Instance {
        @location(3) model_matrix: mat4x4<f32>,
        @location(7) color: vec4<f32>,
    }
    
    @vertex
    micro main(
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        instance: Instance
    ) -> VertexOutput {
        var output: VertexOutput;
        output.world_position = (instance.model_matrix * vec4<f32>(position, 1.0)).xyz;
        output.clip_position = uniforms.view_projection * vec4<f32>(output.world_position, 1.0);
        output.normal = (instance.model_matrix * vec4<f32>(normal, 0.0)).xyz;
        output.color = instance.color;
        return output;
    }
}
```

### Level of Detail (LOD)

```valkyrie
class LODSystem {
    lod_levels: [Mesh],
    distances: [f32],
}

imply LODSystem {
    micro select_lod(self, camera_position: Vec3, object_position: Vec3) -> Mesh {
        let distance = (camera_position - object_position).length()
        
        for (i, threshold) in self.distances.iter().enumerate() {
            if distance < threshold {
                return self.lod_levels[i].clone()
            }
        }
        
        self.lod_levels.last().unwrap().clone()
    }
}
```

## Best Practices

1. **Batch Rendering**: Minimize draw calls, use instanced rendering
2. **Texture Atlasing**: Reduce texture bindings, use texture atlases
3. **Frustum Culling**: Avoid rendering objects outside view frustum
4. **Occlusion Culling**: Skip rendering of occluded objects
5. **LOD System**: Use appropriate level of detail based on distance
6. **Memory Management**: Properly manage GPU resources, avoid memory leaks
7. **Profiling**: Use profiling tools to identify rendering bottlenecks

Valkyrie's graphics programming capabilities provide developers with a complete toolchain for building high-performance rendering systems, from basic shaders to advanced PBR materials, from post-processing effects to performance optimization.
