# 生命周期与内存管理 (Lifetime & Memory Management)

Valkyrie 采用分层设计的内存管理系统，旨在平衡开发效率（UX）与对底层的精细控制。Valkyrie 默认提供垃圾回收 (GC) 机制，同时也支持在高性能或嵌入式场景下进行显式的内存管理。

## 核心支柱

Valkyrie 的内存管理体系建立在以下核心概念之上：

1.  **[AIFD 生命周期模型](lifecycle.md)**：将对象的生命周期严谨地划分为内存获取 (Allocate)、状态准备 (Initiate)、逻辑清理 (Finalize) 和内存归还 (Delocate) 四个清晰阶段。
2.  **[作用域与静态分析](scope.md)**：阐述 Valkyrie 如何利用确定性作用域与深度控制流分析，实现生命周期函数的全自动、智能化注入。
3.  **[引用类型 (Class)](class.md)**：作为应用层开发的默认选择，提供零心智负担的内存安全保障，底层由高性能垃圾回收 (GC) 引擎驱动。
4.  **[值类型 (Structure)](structure.md)**：为性能敏感型场景提供极致的内存控制能力，支持数据的内联存储。
5.  **[分配器 (Allocator)](allocator.md)**：为底层开发提供精细的内存控制，允许精确编排 AIFD 模型中的物理分配与释放。
6.  **[外部对象 (Foreign Objects)](foreign-objects.md)**：定义了在与 C/C++/Rust 等外部语言进行互操作时，如何严谨地管理跨语言边界的对象生命周期。

---

## 快速导航

- **探索对象从诞生到消亡的全过程？** 请参阅 [AIFD 生命周期](lifecycle.md)。
- **探究编译器如何确定销毁的时机？** 请参阅 [作用域与自动插入](scope.md)。
- **深入理解引用类型的运行机制？** 请参阅 [引用类型 (Class)](class.md)。
- **追求极致性能或进行底层系统开发？** 请参阅 [值类型 (Structure)](structure.md)。
- **需要自定义内存分配行为？** 请参阅 [分配器 (Allocator)](allocator.md)。
- **需要与现有的 C/Rust 库进行无缝集成？** 请参阅 [外部对象互操作](foreign-objects.md)。
