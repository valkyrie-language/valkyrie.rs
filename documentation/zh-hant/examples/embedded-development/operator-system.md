# 操作系統開發

Valkyrie 提供了從零開始構建操作系統內核的完整能力，支持裸機（Bare Metal）編程和實時操作系統（RTOS）的開發。

## 操作系統 (OS)

在 Valkyrie 中编寫操作系統內核時，可以利用語言提供的底層特性來直接操纵硬件。

### 裸機環境配置

通過禁用標準庫並定義入口點，Valkyrie 可以直接在硬件上運行。

```valkyrie
# 聲明入口點
@lang(entry)
micro kernel_main() {
    # 初始化 UART 串口
    uart_init()
    
    # 初始化內存管理單元 (MMU)
    mmu_init()
    
    # 打印歡迎信息
    print("Valkyrie OS is starting...")
    
    loop {}
}
```

### 手动內存管理

在 OS 內核開發中，精確的內存控制至关重要。Valkyrie 允許在不使用 `class`（以及與之相關的垃圾回收和運行時元數據）的情況下，仅使用纯 `structure` 來實現完全的手动內存管理。

- **零運行時開銷**：纯結構體不包含任何隐藏的指针或類型信息，其佈局與 C 語言結構體完全兼容。
- **显式分配與釋放**：開發者可以利用裸指针 (`◆T`, `◇T`) 直接操作物理內存或實現自定義的分配器。

```valkyrie
# 纯結構體，無 class 元數據
structure PhysicalPage {
    address: u64,
    flags: u32
}

# 簡單的页分配器
structure PageAllocator {
    current_free: ◆PhysicalPage,
    capacity: usize
}

imply PageAllocator {
    # 手动分配範例
    micro allocate_page(mut self) -> ◆PhysicalPage {
        let ptr = self.current_free
        # 假設 PhysicalPage 數組是連續的
        self.current_free = (ptr as usize + sizeof(PhysicalPage)) as ◆PhysicalPage
        ptr
    }
}
```

### 內存佈局與鏈接

使用 `structure` 和特定的內存佈局注解，可以精確控制內核在物理內存中的位置。

```valkyrie
@repr(C)
structure PageTableEntry {
    present: bool,
    writable: bool,
    user: bool,
    pfn: u64
}
```

### 中斷處理與代數效应

Valkyrie 的代數效应系統可以完美建模硬件中斷和系統調用。通過定義 `effect`，我们可以將底層的硬件事件抽象為高層的邏輯操作。

```valkyrie
# 定義系統調度效应
effect Scheduling {
    # 主动放弃 CPU
    yield(): void
    # 睡眠一段時間
    sleep(ms: u64): void
}

# 時钟中斷處理
# @repr(interrupt) 注解指示編譯器生成中斷返回指令
@repr(interrupt)
micro timer_handler() {
    # 更新系統节拍
    tick()
    # 觸發調度效应，由內核調度器處理
    perform Scheduling.yield()
}
```

---

## 實時操作系統 (RTOS)

對于需要高度确定性的實時系統，Valkyrie 的零成本抽象和內存安全特性提供了極大的优势。

### 任務管理與調度

利用 `HeaplessVec` 等無堆分配的數據結構，可以實現完全确定性的任務調度。

```valkyrie
# 任務控制塊 (TCB)
structure TaskControlBlock {
    stack_ptr: ◆void,
    priority: u8,
    state: TaskState
}

# 優先級調度邏輯
imply Scheduler {
    micro schedule(mut self) {
        let next_task = self.find_highest_priority_ready_task()
        self.switch_to(next_task)
    }
}
```

### 快速上下文切换

Valkyrie 的執行流切换機制天然支持高效的任務切换，这可以被用來實現極速的 RTOS 上下文切换。

```valkyrie
# 利用效应實現非阻塞任務挂起
micro async_task() {
    # 執行一些操作
    # ...
    # 主动放弃 CPU，將控制权交还给調度器
    perform Scheduling.yield()
    # 恢復執行
}
```

### 确定性保證

- **無 GC 干扰**：Valkyrie 允許在內核和實時任務中禁用垃圾回收，消除不可預測的停顿。
- **內存安全**：在编寫底層驅動和內核代碼時，Valkyrie 的所有权模型能有效防止常见的內存溢出和竞態條件。
