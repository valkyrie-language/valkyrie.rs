# 线性类型 (Linear Types)

线性类型系统（Linear Type System）是 Valkyrie 处理资源管理和并发安全的核心。它要求一个变量**必须且只能被使用一次**。这比 Rust 的仿射类型（Affine Types，即变量最多被使用一次）更加严格。

## 线性 vs. 仿射

- **仿射类型 (Rust/Valkyrie 默认)**：资源**最多**使用一次。如果不手动销毁，编译器会自动插入销毁代码（析构）。
- **线性类型 (Valkyrie 显式声明)**：资源**必须**使用一次。你不能忽略它，也不能重复使用它。

## 在 Valkyrie 中使用

在 Valkyrie 中，大多数类型是仿射的（基于 AIFD 模型自动管理）。当你需要更严苛的保证时，可以使用 `@linear` 注解：

- **`@linear` 结构体**：其实例必须被手动处理。
- **`@linear` 参数**：函数承诺会完全消耗该参数。

---

## 核心语义

### 1. 资源不灭
线性类型定义的资源不能被随意丢弃（Drop），必须显式地交给某个函数处理或销毁。

```valkyrie
@linear
structure FileHandle {
    fd: i32
}

micro close_file(f: FileHandle) {
    # 物理销毁资源
    @syscall_close(f.fd)
}

micro example() {
    let f = open_file("data.txt")
    # 如果这里不调用 close_file(f)，编译器将报错
    # 报错信息：Linear resource 'f' must be consumed.
}
```

### 2. 状态协议 (Protocol Tracking)
线性类型非常适合建模必须按顺序执行的状态机。

```valkyrie
@linear
unite Connection {
    Disconnected,
    Connected { socket: Socket },
}

micro connect(c: Disconnected) -> Connected { ... }
micro send(c: Connected, msg: utf8) -> Connected { ... }
micro close(c: Connected) { ... }

# 使用示例
let c1: Disconnected = ...
let c2 = connect(c1)    # c1 被消耗
let c3 = send(c2, "hi") # c2 被消耗
close(c3)               # c3 被消耗
```

---

## 线性类型的应用

1. **协议验证**: 确保网络协议（如握手 -> 数据 -> 挥手）按顺序执行且没有步骤被跳过。
2. **内存手动管理**: 在不使用 GC 的场景下，确保分配的每一块内存都被精确地释放。
3. **并发无锁化**: 证明资源的所有权在线程间线性传递，从而消除竞态条件。

---

## 进阶应用：类型安全状态机 (Type-Safe FSM)

通过将 ADT 与线性类型结合，我们可以构建在编译时强制执行状态转换规则的协议，确保开发者不会在错误的状态下调用方法。

### 场景：文件传输握手
```valkyrie
@linear
unite Connection {
    Disconnected,
    Handshaking { progress: i32 },
    Established { socket: Socket },
}

# 状态转换函数：消耗旧状态并返回新状态
micro start_handshake(c: Disconnected) -> Handshaking {
    Handshaking { progress: 0 }
}

micro complete_handshake(c: Handshaking) -> Established {
    Established { socket: Socket::new() }
}

micro close(c: Established) -> Disconnected {
    Disconnected
}

micro example() {
    let c1 = Disconnected
    let c2 = start_handshake(c1)
    let c3 = complete_handshake(c2)
    close(c3)
}

---

## 底层实现 (Implementation Details)

Valkyrie 的线性类型并不是依靠复杂的运行时检查实现的，而是通过 **Valkyrie 编译器** 与 **Nyar VM/GC** 的深度协同。

### 1. 编译期静态分析
线性类型的“必须且只能使用一次”规则完全由 Valkyrie 编译器在编译期保证。
- **所有权跟踪**：编译器会追踪每个线性资源的生命周期。如果资源在路径结束时未被消耗，或者被重复使用，编译器将直接报错。
- **零成本抽象**：在生成的字节码层面，线性资源与普通资源没有区别。VM 并不需要维护额外的引用计数或标记位。

### 2. Nyar VM 中的所有权转移 (Move)
在 Nyar VM 这一栈式虚拟机中，线性资源的“移动”通过原生的栈操作实现：
- **移动 (Move)**：当线性资源作为参数传递或赋值给新变量时，编译器生成 `PUSH` 和 `STORE` 指令。编译器确保在 `STORE` 之后，旧的局部变量槽位（Local Slot）不再被访问，或者显式地写入 `null` 以切断引用。
- **禁止拷贝**：对于标记为 `@linear` 的类型，编译器严禁生成 `DUP` 等会导致指针副本的指令。

### 3. 代数效应与资源封装
当代数效应捕获 `Continuation` 时，底层机制如下：
- **状态快照**：Nyar VM 在处理效应时会将当前的执行栈和局部变量克隆到一个 `Continuation` 对象中。
- **所有权转移到 Continuation**：此时，栈上的线性资源所有权转移到了该 `Continuation` 对象内部。
- **线性 Resume**：如果 `Continuation` 包含线性资源，Valkyrie 编译器会要求该 `Continuation` 自身也必须作为线性资源处理（只能被 `resume` 一次）。

### 4. Nyar GC 提供的确定性回收 (Safety Net)
如果一个包含线性资源的 `Continuation` 因为逻辑分支原因从未被 `resume`，Nyar GC 将提供语义保障：
- **销毁钩子**：当 Nyar GC 回收不再可达的 `Continuation` 对象时，会调用其 VTable 中的 `drop_and_dealloc` 函数。
- **级联清理**：该函数会递归地调用其内部持有的所有线性资源的析构函数。这确保了即使在复杂的代数效应跳转中，文件句柄、网络连接等外部资源也能被确定性地关闭，其语义与 Rust 的 `Drop` 机制高度一致。
```

---

**上一页**: [依赖类型](./dependent-types.md) | **下一页**: [效应类型](./effect-type.md)
