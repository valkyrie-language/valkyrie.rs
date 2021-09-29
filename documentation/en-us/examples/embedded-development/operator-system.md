# Operating System Development

Valkyrie provides complete capabilities for building operating system kernels from scratch, supporting bare metal programming and real-time operating system (RTOS) development.

## Operating System (OS)

When writing operating system kernels in Valkyrie, you can leverage the language's low-level features to directly manipulate hardware.

### Bare Metal Environment Configuration

By disabling the standard library and defining an entry point, Valkyrie can run directly on hardware.

```valkyrie
# Declare entry point
@lang(entry)
micro kernel_main() {
    # Initialize UART serial port
    uart_init()
    
    # Initialize Memory Management Unit (MMU)
    mmu_init()
    
    # Print welcome message
    print("Valkyrie OS is starting...")
    
    loop {}
}
```

### Manual Memory Management

In OS kernel development, precise memory control is crucial. Valkyrie allows using only pure `structure` without `class` (and associated garbage collection and runtime metadata) to achieve complete manual memory management.

- **Zero Runtime Overhead**: Pure structures contain no hidden pointers or type information; their layout is fully compatible with C structures.
- **Explicit Allocation and Deallocation**: Developers can use raw pointers (`◆T`, `◇T`) to directly manipulate physical memory or implement custom allocators.

```valkyrie
# Pure structure, no class metadata
structure PhysicalPage {
    address: u64,
    flags: u32
}

# Simple page allocator
structure PageAllocator {
    current_free: ◆PhysicalPage,
    capacity: usize
}

imply PageAllocator {
    # Manual allocation example
    micro allocate_page(mut self) -> ◆PhysicalPage {
        let ptr = self.current_free
        # Assume PhysicalPage array is contiguous
        self.current_free = (ptr as usize + sizeof(PhysicalPage)) as ◆PhysicalPage
        ptr
    }
}
```

### Memory Layout and Linking

Using `structure` with specific memory layout annotations allows precise control over the kernel's position in physical memory.

```valkyrie
@repr(C)
structure PageTableEntry {
    present: bool,
    writable: bool,
    user: bool,
    pfn: u64
}
```

### Interrupt Handling and Algebraic Effects

Valkyrie's algebraic effect system can perfectly model hardware interrupts and system calls. By defining `effect`, we can abstract low-level hardware events into high-level logical operations.

```valkyrie
# Define scheduling effect
effect Scheduling {
    # Actively yield CPU
    yield(): void
    # Sleep for a duration
    sleep(ms: u64): void
}

# Timer interrupt handler
# @repr(interrupt) annotation instructs compiler to generate interrupt return instruction
@repr(interrupt)
micro timer_handler() {
    # Update system tick
    tick()
    # Trigger scheduling effect, handled by kernel scheduler
    perform Scheduling.yield()
}
```

---

## Real-Time Operating System (RTOS)

For real-time systems requiring high determinism, Valkyrie's zero-cost abstractions and memory safety features provide significant advantages.

### Task Management and Scheduling

Using heapless data structures like `HeaplessVec`, completely deterministic task scheduling can be implemented.

```valkyrie
# Task Control Block (TCB)
structure TaskControlBlock {
    stack_ptr: ◆void,
    priority: u8,
    state: TaskState
}

# Priority scheduling logic
imply Scheduler {
    micro schedule(mut self) {
        let next_task = self.find_highest_priority_ready_task()
        self.switch_to(next_task)
    }
}
```

### Fast Context Switching

Valkyrie's execution flow switching mechanism naturally supports efficient task switching, which can be used to implement extremely fast RTOS context switches.

```valkyrie
# Use effects to implement non-blocking task suspension
micro async_task() {
    # Execute some operations
    # ...
    # Actively yield CPU, return control to scheduler
    perform Scheduling.yield()
    # Resume execution
}
```

### Determinism Guarantees

- **No GC Interference**: Valkyrie allows disabling garbage collection in kernel and real-time tasks, eliminating unpredictable pauses.
- **Memory Safety**: When writing low-level drivers and kernel code, Valkyrie's ownership model effectively prevents common memory overflows and race conditions.
