# 操作系统开发

Valkyrie 提供了从零开始构建操作系统内核的完整能力，支持裸机（Bare Metal）编程和实时操作系统（RTOS）的开发。

## 操作系统 (OS)

在 Valkyrie 中编写操作系统内核时，可以利用语言提供的底层特性来直接操纵硬件。

### 裸机环境配置

通过禁用标准库并定义入口点，Valkyrie 可以直接在硬件上运行。

```valkyrie
# 声明入口点
@lang(entry)
micro kernel_main() {
    # 初始化 UART 串口
    uart_init()
    
    # 初始化内存管理单元 (MMU)
    mmu_init()
    
    # 打印欢迎信息
    print("Valkyrie OS is starting...")
    
    loop {}
}
```

### 手动内存管理

在 OS 内核开发中，精确的内存控制至关重要。Valkyrie 允许在不使用 `class`（以及与之相关的垃圾回收和运行时元数据）的情况下，仅使用纯 `structure` 来实现完全的手动内存管理。

- **零运行时开销**：纯结构体不包含任何隐藏的指针或类型信息，其布局与 C 语言结构体完全兼容。
- **显式分配与释放**：开发者可以利用裸指针 (`◆T`, `◇T`) 直接操作物理内存或实现自定义的分配器。

```valkyrie
# 纯结构体，无 class 元数据
structure PhysicalPage {
    address: u64,
    flags: u32
}

# 简单的页分配器
structure PageAllocator {
    current_free: ◆PhysicalPage,
    capacity: usize
}

imply PageAllocator {
    # 手动分配示例
    micro allocate_page(mut self) -> ◆PhysicalPage {
        let ptr = self.current_free
        # 假设 PhysicalPage 数组是连续的
        self.current_free = (ptr as usize + sizeof(PhysicalPage)) as ◆PhysicalPage
        ptr
    }
}
```

### 内存布局与链接

使用 `structure` 和特定的内存布局注解，可以精确控制内核在物理内存中的位置。

```valkyrie
@repr(C)
structure PageTableEntry {
    present: bool,
    writable: bool,
    user: bool,
    pfn: u64
}
```

### 中断处理与代数效应

Valkyrie 的代数效应系统可以完美建模硬件中断和系统调用。通过定义异常类型，我们可以将底层的硬件事件抽象为高层的逻辑操作。

```valkyrie
# 时钟中断处理
# @repr(interrupt) 注解指示编译器生成中断返回指令
@repr(interrupt)
micro timer_handler() {
    # 更新系统节拍
    tick()
    # 触发调度异常，由内核调度器处理
    raise SchedulingYield
}
```

---

## 实时操作系统 (RTOS)

对于需要高度确定性的实时系统，Valkyrie 的零成本抽象和内存安全特性提供了极大的优势。

### 任务管理与调度

利用 `HeaplessVec` 等无堆分配的数据结构，可以实现完全确定性的任务调度。

```valkyrie
# 任务控制块 (TCB)
structure TaskControlBlock {
    stack_ptr: ◆void,
    priority: u8,
    state: TaskState
}

# 优先级调度逻辑
imply Scheduler {
    micro schedule(mut self) {
        let next_task = self.find_highest_priority_ready_task()
        self.switch_to(next_task)
    }
}
```

### 快速上下文切换

Valkyrie 的执行流切换机制天然支持高效的任务切换，这可以被用来实现极速的 RTOS 上下文切换。

```valkyrie
# 利用异常实现非阻塞任务挂起
micro async_task() {
    # 执行一些操作
    # ...
    # 主动放弃 CPU，将控制权交还给调度器
    raise SchedulingYield
    # 恢复执行
}
```

### 确定性保证

- **无 GC 干扰**：Valkyrie 允许在内核和实时任务中禁用垃圾回收，消除不可预测的停顿。
- **内存安全**：在编写底层驱动和内核代码时，Valkyrie 的所有权模型能有效防止常见的内存溢出和竞态条件。
