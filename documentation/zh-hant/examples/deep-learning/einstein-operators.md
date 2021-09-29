# Einstein 操作符 (Einops)

Valkyrie 提供了強大的 Einstein 操作符，靈感來自于 einops 庫，用於優雅地處理多維數組的重排、重塑和約簡操作。通過直觀的字符串表示法，可以輕鬆表达複雜的張量操作。

## 基本概念

Einstein 操作符使用字符串模式來描述張量操作，其中：
- 字母表示維度
- 空格分隔不同的維度組
- `()` 表示新增維度
- `...` 表示省略的維度

## 重排操作 (Rearrange)

```valkyrie
use valkyrie::tensor::einops::*

# 圖像數據格式轉換
let 𝐗 = ArrayND::random([32, 224, 224, 3])  # NHWC格式

# 轉換為NCHW格式
let 𝐘 = rearrange(𝐗, "n h w c -> n c h w")

# 將batch展平為序列
let 𝐬 = rearrange(𝐗, "n h w c -> (n h w) c")

# 創建圖像塊 (patches)
let 𝐏 = rearrange(𝐗, "n (h p1) (w p2) c -> n (h w) (p1 p2 c)", 
                       p1=16, p2=16)  # 16x16 patches

# 多頭注意力的頭部重排
let 𝐖 = ArrayND::random([8, 12, 64, 64])  # batch, heads, seq, seq
let 𝐑 = rearrange(𝐖, "b h s1 s2 -> (b h) s1 s2")

# 時間序列重塑
let 𝐓 = ArrayND::random([100, 24, 7])  # days, hours, features
let 𝐖 = rearrange(𝐓, "(w d) h f -> w (d h) f", w=14, d=7)
```

## 約簡操作 (Reduce)

```valkyrie
# 全局平均池化
let 𝐅 = ArrayND::random([32, 512, 7, 7])  # batch, channels, height, width
let 𝐆 = reduce(𝐅, "n c h w -> n c", "mean")

# 沿特定軸求和
let 𝐒 = reduce(𝐅, "n c h w -> c h w", "sum")

# 多軸約簡
let μ = reduce(𝐅, "n c h w -> c", "mean")  # 每個通道的均值
let 𝐌 = reduce(𝐅, "n c h w -> n c", "max")   # 空間最大值

# 注意力權重歸一化
let 𝐋 = ArrayND::random([8, 12, 64, 64])
let 𝐀 = reduce(𝐋, "b h i j -> b h i j", "softmax")

# 時間序列統計
let μ_d = reduce(𝐓, "d h f -> d f", "mean")     # 每日平均
let 𝐌_h = reduce(𝐓, "d h f -> h f", "max")    # 每小時最大值
```

## 重複操作 (Repeat)

```valkyrie
# 廣播操作
let 𝐛 = ArrayND::random([512])  # 偏置向量
let 𝐁 = repeat(𝐛, "c -> n c h w", n=32, h=7, w=7)

# 數據增強 - 重複樣本
let 𝐬 = ArrayND::random([224, 224, 3])
let 𝐀 = repeat(𝐬, "h w c -> n h w c", n=8)  # 創建8個副本

# 位置編碼重複
let 𝐄 = ArrayND::random([64, 512])  # seq_len, d_model
let 𝐁 = repeat(𝐄, "s d -> n s d", n=32)  # 為整個batch重複

# 卷積核重複
let 𝐊 = ArrayND::random([3, 3])  # 2D卷積核
let 𝐂 = repeat(𝐊, "h w -> c_out c_in h w", c_out=64, c_in=3)
```

## 複雜操作組合

```valkyrie
# Vision Transformer patch embedding
class PatchEmbedding {
    patch_size: Integer
    embed_dim: Integer
    
    forward(self, 𝐱: ArrayND) -> ArrayND {
        # 將圖像分割為patches
        let patches = rearrange(𝐱, 
            "n (h p1) (w p2) c -> n (h w) (p1 p2 c)",
            p1=self.patch_size, p2=self.patch_size)
        
        # 線性投影到embedding維度
        let embedded = self.linear(patches)  # n (h w) embed_dim
        return embedded
    }
}

# 多尺度特徵融合
micro multi_scale_fusion(features: [ArrayND]) -> ArrayND {
    let unified_features = []
    
    for (i, feat) in features.enumerate() {
        # 統一空間尺寸
        let resized = if i == 0 {
            feat
        } else {
            # 上採樣到最大尺寸
            interpolate(feat, size=[features[0].shape()[2], features[0].shape()[3]])
        }
        
        # 重排為統一格式
        let rearranged = rearrange(resized, "n c h w -> n (h w) c")
        unified_features.push(rearranged)
    }
    
    # 沿通道維度拼接
    let fused = concatenate(unified_features, axis=2)
    return rearrange(fused, "n (h w) c -> n c h w", 
                    h=features[0].shape()[2], w=features[0].shape()[3])
}

# 自注意力機制
class MultiHeadAttention {
    num_heads: Integer
    head_dim: Integer
    
    forward(self, x: ArrayND) -> ArrayND {
        let n, s, d = x.shape()
        
        # 計算Q, K, V
        let qkv = self.qkv_proj(x)  # n s (3 * num_heads * head_dim)
        let qkv_reshaped = rearrange(qkv, 
            "n s (three h d) -> three n h s d", 
            three=3, h=self.num_heads, d=self.head_dim)
        
        let q, k, v = qkv_reshaped[0], qkv_reshaped[1], qkv_reshaped[2]
        
        # 計算注意力分數
        let scores = einsum("n h i d, n h j d -> n h i j", q, k) / sqrt(self.head_dim)
        let attention = softmax(scores, axis=-1)
        
        # 應用注意力
        let out = einsum("n h i j, n h j d -> n h i d", attention, v)
        
        # 重新組合多頭輸出
        let combined = rearrange(out, "n h s d -> n s (h d)")
        return self.out_proj(combined)
    }
}
```

## 高級模式匹配

```valkyrie
# 動態形狀處理
micro adaptive_pooling(x: ArrayND, target_size: Tuple⟨Integer, Integer⟩) -> ArrayND {
    let n, c, h, w = x.shape()
    let th, tw = target_size
    
    # 自適應池化窗口大小
    let pool_h = h / th
    let pool_w = w / tw
    
    # 使用einops進行自適應池化
    let pooled = reduce(x, 
        "n c (th ph) (tw pw) -> n c th tw", 
        "mean", th=th, tw=tw, ph=pool_h, pw=pool_w)
    
    return pooled
}

# 序列到序列的注意力
micro seq2seq_attention(encoder_out: ArrayND, decoder_hidden: ArrayND) -> ArrayND {
    # encoder_out: [batch, enc_seq, hidden]
    # decoder_hidden: [batch, dec_seq, hidden]
    
    # 計算注意力權重
    let attention_scores = einsum("b i h, b j h -> b i j", decoder_hidden, encoder_out)
    let attention_weights = softmax(attention_scores, axis=-1)
    
    # 應用注意力
    let context = einsum("b i j, b j h -> b i h", attention_weights, encoder_out)
    
    return context
}

# 圖卷積網絡的鄰接矩陣操作
micro graph_convolution(node_features: ArrayND, adjacency: ArrayND) -> ArrayND {
    # node_features: [batch, nodes, features]
    # adjacency: [batch, nodes, nodes]
    
    # 聚合鄰居特徵
    let aggregated = einsum("b i j, b j f -> b i f", adjacency, node_features)
    
    # 歸一化
    let degree = reduce(adjacency, "b i j -> b i", "sum")
    let degree_expanded = repeat(degree, "b i -> b i f", f=node_features.shape()[2])
    
    let normalized = aggregated / (degree_expanded + 1e-8)
    return normalized
}
```

## 性能優化

```valkyrie
# 內存高效的操作
micro memory_efficient_attention(q: ArrayND, k: ArrayND, v: ArrayND, 
                                 chunk_size: Integer = 1024) -> ArrayND {
    let b, h, s, d = q.shape()
    let output = ArrayND::zeros([b, h, s, d])
    
    # 分塊計算避免大矩陣乘法
    loop i in 0..s step chunk_size {
        let end_i = min(i + chunk_size, s)
        let q_chunk = q.slice(2, i..end_i)  # [b, h, chunk, d]
        
        # 計算當前chunk的注意力
        let scores = einsum("b h i d, b h j d -> b h i j", q_chunk, k)
        let attention = softmax(scores, axis=-1)
        let out_chunk = einsum("b h i j, b h j d -> b h i d", attention, v)
        
        output.slice_mut(2, i..end_i).copy_from(out_chunk)
    }
    
    return output
}

# GPU優化的批量操作
micro batch_matrix_multiply(a: ArrayND, b: ArrayND) -> ArrayND {
    # 使用einops確保正確的批量維度對齊
    let a_reshaped = rearrange(a, "... i j -> (...) i j")
    let b_reshaped = rearrange(b, "... j k -> (...) j k")
    
    # 批量矩陣乘法
    let result = einsum("b i j, b j k -> b i k", a_reshaped, b_reshaped)
    
    # 恢復原始形狀
    let original_shape = a.shape()[:-2] + [a.shape()[-2], b.shape()[-1]]
    return result.reshape(original_shape)
}
```

## 最佳實踐

1. **清晰的維度命名**：使用有意義的字母表示不同維度
2. **一致的約定**：在整個項目中保持維度命名的一致性
3. **性能考慮**：對于大張量操作，考慮內存使用和計算效率
4. **類型安全**：利用 Valkyrie 的類型系統確保操作的正確性
5. **文檔化**：為複雜的 einops 操作添加注释說明

Einstein 操作符讓複雜的張量操作变得直觀和可讀，是深度學習和科學計算中不可或缺的工具。