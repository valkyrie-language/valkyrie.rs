# 面向对象编程 (Object-Oriented Programming)

Valkyrie 支持基于类的面向对象编程，但对象系统并不是语言里唯一的抽象层。阅读本章时，建议先区分：

- `class`：对象类型与继承层级
- `sealed class`：显式封闭的类层级
- `unite`：抽象类及其封闭 variant 集合的紧凑写法
- 具名 `trait`：协议与 witness
- 匿名 row：轻量方法约束，不属于对象系统本体

## 核心概念

### 类定义 (Class Definitions)

使用 `class` 关键字定义结构化数据类型：

```valkyrie
class Person {
    public name: utf8
    private age: i32
    
    initiate(mut self, name: utf8, age: i32) {
        self.name = name
        self.age = age
    }
    
    micro greet(self) {
        print("Hello, I'm {self.name}")
    }
}
```

### 特殊类类型

| 类型 | 关键字 | 说明 | 文档 |
|:---|:---|:---|:---|
| 抽象类 | `abstract class` | 不能实例化，可包含抽象成员 | [abstract-class.md](./abstract-class.md) |
| 密封类 | `sealed class` | 显式定义封闭类层级 | [sealed-class.md](./sealed-class.md) |
| 最终类 | `final class` | 禁止继承 | [final-class.md](./final-class.md) |
| 单例对象 | `singleton` | 全局唯一实例 | [singleton.md](./singleton.md) |
| 值类型 | `structure` | 栈分配，复制语义 | [value-class.md](./value-class.md) |
| 组件类 | `widget` | UI 组件 | [widget.md](./widget.md) |

### 对象系统与其他抽象的边界

- 继承和子类关系属于 `class` 这一层。
- 具名协议满足属于 `trait / imply` 这一层。
- 匿名 `{ draw() -> unit }` 这类写法应理解为 row requirement，而不是“匿名类”或“匿名 trait 对象”。
- 如果一个抽象需要关联类型、默认实现或 trait object，它应写成具名 `trait`，而不是继续停留在匿名 row。

## 文档导航

### 类与继承

- [继承系统](./inheritance.md) - 多继承、C3 线性化、重命名继承
- [抽象类](./abstract-class.md) - 抽象方法、模板方法模式
- [密封类](./sealed-class.md) - 封闭类层级与 `unite` 的关系
- [最终类与方法](./final-class.md) - 禁止继承和重写
- [单例对象](./singleton.md) - 全局唯一实例

### Trait 系统

- [Trait 系统](./trait-system.md) - Trait 定义、实现、见证表、动态分发，以及它与匿名 row 的边界

### 封装与访问控制

- [访问控制](./access-control.md) - public/protected/internal/private
- [属性系统](./property.md) - getter/setter、计算属性、验证

### 特殊类型

- [值类型](./value-class.md) - structure 定义、复制语义
- [匿名类](./anonymous-classes.md) - 即时定义、变量捕获；不要与匿名 row 约束混淆
- [组件类](./widget.md) - UI 组件系统

## 对象生命周期

在 Valkyrie 中，一个对象的生命周期包含四个关键阶段：

| 阶段 | 职责 | Trait / 方法 | 调用者 |
|:---|:---|:---|:---|
| **1. Allocate** | 内存获取 | `Allocator::allocate` | 容器 (Box/GC) |
| **2. Initiate** | 状态初始化 | `Initiate::initiate` | 编译器 |
| **3. Finalize** | 状态清理 | `Finalize::finalize` | 容器 (Box/GC) |
| **4. Delocate** | 内存归还 | `Allocator::delocate` | 容器 (Box/GC) |

### 终结器

`Finalize` Trait 用于定义对象在被销毁前的清理逻辑：

```valkyrie
trait Finalize {
    micro finalize(mut self)
}

class FileWrapper {
    handle: i32
    
    initiate(mut self, path: utf8) {
        self.handle = open_file(path)
    }
}

imply FileWrapper: Finalize {
    micro finalize(mut self) {
        close_file(self.handle)
    }
}
```

## 快速参考

### 类成员默认可见性

| 成员类型 | 默认可见性 |
|:---|:---|
| 字段 | `private` |
| 方法 | `public` |
| 属性 | getter/setter 各自独立 |

### 方法接收者模式

```valkyrie
class Example {
    # 不可变借用
    micro read(self) { }
    
    # 可变借用
    micro modify(mut self) { }
    
    # 消费
    micro consume(self) { }
    
    # 静态方法
    micro static create() -> Self { }
}
```

### 继承语法

```valkyrie
# 单继承
class Child(Parent) { }

# 多继承
class Child(Parent1, Parent2) { }

# 重命名继承
class Child(primary: Parent1, secondary: Parent2) { }

# 实现 Trait
class Child: Trait1 + Trait2 { }

# 继承 + Trait
class Child(Parent): Trait { }
```

## 一句话分工

- 想显式写出封闭类层级，用 `sealed class`。
- 想用紧凑语法声明抽象类与互斥 variant，用 `unite`。
- 想表达普通对象类型本体与继承层级，用 `class`。
- 想表达具名协议、默认实现与关联类型，用 `trait / imply`。
- 想表达“只要会这些方法就行”，用匿名 row。
