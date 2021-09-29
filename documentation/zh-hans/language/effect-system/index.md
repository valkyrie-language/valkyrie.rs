# 效应系统 (Effect System)

Valkyrie 的效应系统提供了强大的副作用管理和控制流机制，包括异常处理、协程、生成器、反应式编程、面向切面编程和依赖注入等功能。

## 效应系统组件

- **[异常处理](./error-handler.md)** - 灵活的错误处理和异常传播机制
- **[协程](./coroutine.md)** - 协作式多任务处理的基础
- **[异步效应](./asynchronous.md)** - 基于代数效应的异步编程模型
- **[生成器](./generator.md)** - 惰性计算和值序列生成
- **[反应式编程](./reactive.md)** - 数据流和变化传播的编程范式
- **[面向切面编程](./aop.md)** - 横切关注点的分离和管理
- **[依赖注入](./ioc.md)** - 控制反转和依赖管理

## 核心概念

### 代数效应

代数效应是 Valkyrie 效应系统的核心抽象，允许将副作用的描述与实现分离。任何 Valkyrie 对象都可以作为效应载体被 `raise` 或 `resume`：

```valkyrie
# 定义效应载体（普通结构体）
structure Read {
    prompt: utf8
}

# 触发效应
micro process() -> utf8 {
    let data = raise Read { prompt: "请输入数据" }
    parse(data)
}

# 处理效应
micro main() {
    catch process() {
        case Read { prompt }: resume("input data")
    }
}
```

### 效应与控制流

效应系统统一了多种控制流机制：

| 机制 | 效应类型 | 用途 |
|:---|:---|:---|
| 异常 | `raise` | 错误处理 |
| 异步 | `await` | 并发编程 |
| 生成器 | `yield` | 惰性计算 |
| 状态 | `get/put` | 可变状态 |

---

更多详情请参阅各子章节文档。
