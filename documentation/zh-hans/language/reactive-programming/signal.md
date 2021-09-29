# 信号 (Signal)

`Signal` 是 Valkyrie 中用于状态同步与响应式传播的核心原语。它代表一个随时间变化的“当前值”，并能自动追踪其在响应式上下文（如 UI 渲染、副作用块）中的依赖关系。

## 核心原语：状态 vs 事件

在 Valkyrie 的反应式版图中，`Signal` 与 `Observable` 承担着不同的职责：

| 特性 | Signal (信号) | Observable (观察对象) |
| :--- | :--- | :--- |
| **语义** | **状态** (State)：它是什么 | **事件** (Events)：发生了什么 |
| **时效性** | 持续的数值，始终有当前值 | 离散的序列，代表瞬时行为 |
| **驱动方式** | 细粒度同步更新 | 异步流式传播 |
| **典型应用** | UI 数据绑定、业务状态、配置项 | 点击流、WebSocket、定时器 |

---

## 初始化与代数效应

Valkyrie 使用 `raise` 关键字初始化响应式容器。这表明该操作是一个**挂钩 (Hooking)** 行为，它会向当前的执行上下文请求一个受托管的响应式状态。

```valkyrie
# 初始化一个基础信号 (State)
let count = raise Signal(0)

# 初始化一个异步资源 (Resource)
let user = raise Resource { fetch_user(id) }
```

通过 `raise` 声明，编译器可以将该变量绑定到最近的响应式作用域（如 Widget），并确保在作用域销毁时自动调度清理逻辑。

---

## 抽象接口：Accessor 与 Settler

为了实现严格的读写分离与多态性，Signal 体系基于两个核心 Trait 构建：

### Accessor⟨T⟩ (访问器)
代表对状态的**观察权限**。任何接受 `Accessor` 的函数或组件都可以读取该值，并自动将其注册为响应式依赖。
- `property value: T { get }`

### Settler⟨T⟩ (建立器)
继承自 `Accessor`，代表对状态的**修改权限**。
- `property value: T { get, set }`

---

## 状态容器矩阵

| 容器类型 | 实现接口 | 生命周期 | 说明 |
| :--- | :--- | :--- | :--- |
| **Signal** | `Settler` | 作用域持久 | 基础的可变状态源。 |
| **Memo** | `Accessor` | 随依赖自动销毁 | 基于其他信号生成的衍生计算值。 |
| **Resource** | `Accessor` | 受托管异步 | 封装了异步操作及其加载/错误状态。 |
| **Bridge** | `Settler` | 跨端同步 | 由框架托管，实现前后端状态的自动一致性。 |

---

## 基本用法

### 1. 自动依赖追踪
得益于编译器的深度集成，用户可以直接操作信号变量，而无需显式调用解包方法。

```valkyrie
let count = raise Signal(0)
let doubled = Memo { count * 2 }

# 在 Effect 块中读取，自动建立订阅
Effect {
    print("Value: {count}, Double: {doubled}")
}

# 像普通变量一样赋值，触发细粒度更新
count = count + 1
```

### 2. 衍生状态 (Memo)
`Memo` 用于创建只读的计算属性，它仅在依赖项发生变化时才会重新计算。

```valkyrie
let list = raise Signal([1, 2, 3])
let sum = Memo { list.iter().sum() }
```

---

## 进阶特性

### 批处理 (Batching)
在高性能场景下，可以使用批处理合并多次修改，使订阅者仅在最后触发一次更新。

```valkyrie
std::reactive::batch {
    # 无论循环多少次，依赖该信号的 Effect 只执行一次
    loop i in 1..100 {
        count = i
    }
}
```

### 跨端同步 (Bridge)

`Bridge` 是 Valkyrie 专为分布式环境（如前后端分离、多进程协作）设计的增强型信号。它打破了内存边界，使得状态可以在不同的执行环境之间自动保持同步。

#### 核心原理
`Bridge` 本质上是一个具备**透明传输能力**的 `Settler`。当状态在一端发生变更时，框架会执行以下流程：
1. **变更捕获**：利用编译器生成的追踪信息，识别受影响的数据片段。
2. **差分序列化**：仅将变更的部分（Delta）序列化为紧凑的二进制格式（如 ProtoBuf 或内部格式）。
3. **协议传输**：通过底层抽象的 `Transport` 接口（支持 WebSocket, gRPC, 或 SharedMemory）发送到对端。
4. **状态对齐**：对端接收到变更后，原子化地更新本地副本，并触发本地的响应式依赖。

#### 使用场景
- **全栈实时同步**：在后端修改 `user_score`，前端 UI 实时跳动。
- **协同编辑**：多用户共享同一个 `Bridge` 容器，实现类似 Google Docs 的实时反馈。
- **分布式配置**：在配置中心修改信号，所有微服务实例自动感知。

#### 代码示例
```valkyrie
# 在后端定义并导出
export let server_status = raise Bridge("status", "Initializing")

# 在前端通过标识符挂钩
let status = raise Bridge("status")

Effect {
    print("服务器状态实时感知: {status}")
}
```

#### 冲突处理与一致性
`Bridge` 默认遵循**最终一致性 (Eventual Consistency)**。在极高性能要求的场景下，可以配置不同的同步策略：
- `SyncStrategy::Eager`：立即发送，适用于实时 UI。
- `SyncStrategy::Debounced(ms)`：防抖发送，合并频繁修改，节省带宽。
- `SyncStrategy::Reliable`：确保送达，适用于关键业务逻辑。

---

## 性能与生命周期

- **细粒度更新**：Signal 系统构建了精确的拓扑依赖图，更新时仅触达真正受影响的节点，避免了昂贵的全局比对（Diff）。
- **确定性终结**：通过 AIFD 模型，当响应式作用域结束时，所有相关的 Signal 节点及其订阅关系都会被编译器插入的代码自动清理，确保无内存泄漏。
