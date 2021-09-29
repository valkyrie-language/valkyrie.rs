# 嵌入式開發

Valkyrie 提供了完整的嵌入式開發解決案，支持 WebAssembly (WASM) 目標、微控制器編程、實時系統開發等。通過零成本抽象和內存安全保證，Valkyrie 為嵌入式系統提供了現代化的開發體验。

## 核心特性

### 內存管理

```valkyrie
# 栈分配的固定大小數組
type FixedBuffer<T, const N: usize> = array<T, N>

# 無堆分配的向量實現
structure HeaplessVec<T, const N: usize> {
    data: array<MaybeUninit<T>, N>,
    len: usize
}

impl<T, const N: usize> HeaplessVec<T, N> {
    micro new() -> Self {
        HeaplessVec {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0
        }
    }
    
    micro push(mut self, item: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len] = MaybeUninit::new(item)
            self.len += 1
            Ok(())
        } else {
            Err(item)
        }
    }
    
    micro pop(mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1
            Some(unsafe { self.data[self.len].assume_init_read() })
        } else {
            None
        }
    }
    
    micro get(self, index: usize) -> Option<T> {
        if index < self.len {
            Some(unsafe { self.data[index].assume_init_ref() })
        } else {
            None
        }
    }
}
```

### 實時系統支持

Valkyrie 的零成本抽象和對裸指针的精细控制，使其成為编寫實時操作系統（RTOS）的理想選擇。

- [操作系統與 RTOS 開發](./operator-system.md)
- [數字電路設計](./digital-circuits.md)

## WebAssembly 集成

### WASM 模組開發

```valkyrie
# WASM 導出函數
@wasm_export
micro add(a: i32, b: i32) -> i32 {
    a + b
}

@wasm_export
micro process_buffer(ptr: ◆u8, len: usize) -> i32 {
    unsafe {
        let mut sum = 0
        loop i in 0..<len {
            sum += i32(ptr[i])
        }
        sum
    }
}

# WASM 內存管理
structure WasmAllocator {
    heap_start: ◆u8,
    heap_size: usize,
    free_blocks: HeaplessVec<MemoryBlock, 64>
}

structure MemoryBlock {
    ptr: ◆u8,
    size: usize
}

imply WasmAllocator {
    micro new(heap_start: ◆u8, heap_size: usize) -> WasmAllocator {
        let mut allocator = WasmAllocator {
            heap_start,
            heap_size,
            free_blocks: HeaplessVec::new()
        }
        
        # 初始化一個大的空闲塊
        allocator.free_blocks.push(MemoryBlock {
            ptr: heap_start,
            size: heap_size
        }).unwrap()
        
        allocator
    }
    
    micro allocate(self, layout: Layout) -> ◆u8? {
        for (i, block) in self.free_blocks.iter().enumerate() {
            let aligned_ptr = ◆u8(align_up(usize(block.ptr), layout.align))
            let aligned_size = (usize(block.ptr) + block.size) - (usize(aligned_ptr))
            
            if aligned_size >= layout.size {
                # 分割塊
                let remaining_size = aligned_size - layout.size
                
                if remaining_size > 0 {
                    let remaining_block = MemoryBlock {
                        ptr: unsafe { aligned_ptr.add(layout.size) },
                        size: remaining_size
                    }
                    
                    self.free_blocks[i] = remaining_block
                } else {
                    self.free_blocks.remove(i)
                }
                
                return Some(aligned_ptr)
            }
        }
        
        None
    }
    
    micro delocate(self, ptr: ◆u8, layout: Layout) {
        let block = MemoryBlock { ptr, size: layout.size }
        
        # 簡單的釋放實現，实际應該合並相邻塊
        self.free_blocks.push(block).ok()
    }
}

micro align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
```

### WASI 接口

```valkyrie
# WASI 系統調用封裝
mod wasi {
    @import(wasm32, "wasi_snapshot_preview1", "fd_write")
    micro fd_write(fd: i32, iovs_ptr: ◇IoVec, iovs_len: usize, nwritten: ◆usize) -> i32

    @import(wasm32, "wasi_snapshot_preview1", "fd_read")
    micro fd_read(fd: i32, iovs_ptr: ◇IoVec, iovs_len: usize, nread: ◆usize) -> i32

    @import(wasm32, "wasi_snapshot_preview1", "clock_time_get")
    micro clock_time_get(id: i32, precision: i64, time: ◆i64) -> i32

    @import(wasm32, "wasi_snapshot_preview1", "random_get")
    micro random_get(buf: ◆u8, buf_len: usize) -> i32

    structure IoVec {
        buf: ◇u8,
        buf_len: usize
    }
    
    micro print(msg: str) {
        let iov = IoVec {
            buf: msg.as_ptr(),
            buf_len: msg.length
        }
        
        let mut nwritten = 0
        unsafe {
            fd_write(1, ref iov, 1, mut nwritten)
        }
    }
    
    micro read_input(mut buffer: [u8]) -> usize {
        let iov = IoVec {
            buf: buffer.as_mut_ptr(),
            buf_len: buffer.length
        }
        
        let mut nread = 0
        unsafe {
            fd_read(0, ref iov, 1, mut nread)
        }
        nread
    }
}
```
