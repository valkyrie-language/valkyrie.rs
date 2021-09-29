# 分配器 (Allocator)

`Allocator` 是 Valkyrie 手动内存管理机制的核心抽象，专门负责 AIFD 模型中的 **A (Allocate, 分配)** 和 **D (Delocate, 释放)** 两个关键物理阶段。

## Allocator 接口定义

`Allocator` 接口直接抽象了底层内存的申请与回收行为，为开发者提供了对内存布局的精细控制权。

```valkyrie
trait Allocator {
    # A: 分配内存
    # 根据指定的布局 (Layout) 申请原始内存空间。
    # 返回：指向分配内存起始位置的原始指针，若分配失败则返回 None。
    micro allocate(self, layout: Layout) -> Option⟨◆u8⟩

    # D: 归还内存
    # 将之前分配的内存空间释放并归还给分配器。
    # 参数：ptr 必须是该分配器之前通过 allocate 返回的指针，且 layout 必须与之匹配。
    micro delocate(self, ptr: ◆u8, layout: Layout)
}
```

### 运行时实现：虚表胖指针 (Fat Pointer)

在 Valkyrie 运行时中，当 `Allocator` 接口作为参数或变量进行传递时，它通常被实现为一个**胖指针 (Fat Pointer)**。该结构由两部分组成：
1.  **数据指针**：指向分配器实例的私有状态数据。
2.  **虚函数表 (vtable)**：包含 `allocate` 和 `delocate` 的具体实现地址。

这种设计确保了即便在复杂的动态分发场景下，内存操作的开销依然能保持在极低且确定的水平。

## 显式资源编排 (RAII)

在需要规避垃圾回收 (GC) 不确定性损耗的高性能场景中，开发者可以利用 `Scoped` 或 `Box` 等显式容器，结合自定义分配器来手动编排对象的 AIFD 全生命周期。

### 核心应用：Arena 分配器 (ArenaAllocator)

`ArenaAllocator`（区域分配器）非常适合处理具有高度一致生命周期的批处理任务。它允许在任务执行期间连续快速分配内存，并在任务结束时通过单次操作释放整个内存池，从而极大地提升吞吐量。

```valkyrie
micro heavy_task(arena: ArenaAllocator) {
    # 在指定的 arena 中分配内存并就地初始化对象 (触发 A 与 I 阶段)
    let buffer = Scoped::new_in⟨BigData⟩(args, arena)
    
    # ... 执行高强度业务计算 ...
    
}
# 离开作用域时自动触发销毁：
# 1. 自动调用对象的终结逻辑 (F)
# 2. 通过 arena 接口将物理内存统一归还给系统 (D)
```

## 最佳实践指南

1.  **传递优于持有**：尽量避免在长生命周期的结构体中存储分配器的引用。推荐在具体执行内存操作的方法中，将分配器作为上下文参数传入。
2.  **面向接口设计**：在编写通用的高性能库时，应接受通用的 `Allocator` 接口。这允许库的用户根据实际环境（如嵌入式、高性能服务器等）自由注入最合适的内存分配策略。
3.  **对齐与布局**：在自定义分配逻辑时，务必严格遵守 `Layout` 要求的对齐规则，以避免在某些硬件架构上出现性能下降或非法访问。

---

## 下一步

掌握了分配器后，你已经具备了构建高性能系统的核心能力。最后，让我们看看如何处理那些不受 Valkyrie 直接管理的 **[外部对象 (Foreign Objects)](foreign-objects.md)**，完成跨语言内存管理的最后一块拼图。
