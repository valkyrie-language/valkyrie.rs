# AIFD 生命周期模型

在 Valkyrie 中，一个对象的完整生命周期被严谨地划分为四个独立且衔接的阶段。这种精细的拆解不仅保证了内存安全，更赋予了开发者在不同场景下灵活组合内存管理策略的能力。

## 模型定义

1.  **A (Allocate)**：**内存分配**。从指定分配器 (Allocator) 中申请符合对象对齐与大小要求的原始内存空间。
2.  **I (Initiate)**：**状态初始化**。在已分配的原始内存上进行“就地构造”，确立对象的初始逻辑状态。
3.  **F (Finalize)**：**逻辑终结**。负责清理对象持有的非内存资源（如关闭文件描述符、断开网络连接、释放互斥锁或递减外部引用计数）。
4.  **D (Delocate)**：**内存释放**。将不再使用的物理内存空间归还给原分配器，使其可被后续操作重用。

这种职责分离的设计（尤其是 F 与 D 的分离）是 Valkyrie 能够同时支持 GC 与确定性析构的关键。

---

## 核心接口：Initiate 与 Finalize

开发者可以通过实现特定的 Trait 来介入对象的生命周期。

### 1. 状态初始化：`Initiate`

`Initiate` 定义了如何将原始内存转化为有效的对象状态。

```valkyrie
trait Initiate⟨Args⟩ {
    # 安全性说明：调用者必须确保 ptr 指向的内存已通过 A 阶段成功分配。
    unsafe micro initiate(ptr: ◆Self, args: Args)
}
```

### 2. 逻辑清理：`Finalize`

`Finalize` 专注于资源清理，**严禁**涉及物理内存释放。

```valkyrie
trait Finalize {
    # 允许在对象被物理销毁前，执行最后的资源释放工作。
    micro finalize(mut self)
}
```

---

## 声明式语法与自动化

为了提升开发体验，Valkyrie 允许在类型定义中直接声明 `initiate` 和 `finalize` 方法，编译器会自动将其解构为标准 Trait 实现。

### 自动生成的实现

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,

    # 映射为 Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
    }

    # 映射为 Finalize
    finalize(mut self) {
        intrinsic::close_file(self.handle)
    }
}
```

### 伪重载机制 (Pseudo-overloading)

虽然 Valkyrie 核心语法不支持传统函数重载，但通过 `Initiate⟨Args⟩` 的泛型设计，类可以拥有多个“构造函数”。

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,
    is_temp: bool,

    # 映射为 Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = false
    }

    # 映射为 Initiate⟨Path, bool⟩
    initiate(mut self, path: Path, is_temp: bool) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = is_temp
    }

    # 映射为 Initiate⟨◆u8⟩
    initiate(mut self, handle: ◆u8) {
        self.path = Path::empty()
        self.handle = handle
        self.is_temp = false
    }

    finalize(mut self) {
        intrinsic::close_file(self.handle)
        if self.is_temp {
            intrinsic::delete_file(self.path)
        }
    }
}
```

**黑魔法原理**：当执行 `FileBuffer(...)` 时，编译器并非在寻找同名函数，而是在寻找满足 `Self: Initiate⟨T⟩` 约束的 Trait 实例化。这实现了静态分派的灵活性与高性能。

---

## 类型判定与协议一致性

由于生命周期被抽象为 Trait，所有的元编程与类型判定依然统一：

- **Trait 判定**：`obj is Finalize` 可用于动态检查对象是否需要清理逻辑。
- **缺省行为**：若未显式定义 `initiate`，编译器将生成默认的零初始化构造函数；若省略 `finalize`，则该对象被视为“平凡对象 (Trivial)”，在销毁时无需执行额外操作。
- **一致性协议**：`is` 和 `as` 等关键字始终通过 Trait 协议进行判定，确保了泛型约束（例如 `T: Finalize`）能完美兼容手动实现的结构体与自动生成的类。

---

## 下一步

你已经掌握了 AIFD 生命周期模型的基础理论。接下来，我们将探讨编译器如何利用 **[作用域与自动插入](scope.md)** 技术，在代码执行过程中自动、精准地编排这些生命周期阶段。
