# Trait 系统 (Trait System)

## 概述

Valkyrie 的具名 `trait` 系统提供协议抽象、默认实现、关联类型和具名 witness。它与匿名 row 约束不是同一层语义：

- 具名 `trait` 回答“你是否满足这个被命名的协议”
- 匿名 row 回答“你当前是否提供这组方法”
- `class` 回答“你是不是这个名义类型层级里的成员”
- `unite` 回答“你当前属于哪一个已声明 variant”

理解这三层区别，是正确使用 `trait` 的前提。

## 基础定义与实现

在 Valkyrie 中，Trait 定义了类型的行为，而 `imply` 关键字用于为特定类型实现这些行为。

### 简单 Trait 定义

```valkyrie
trait Display {
    micro fmt(self) -> utf8
}

trait Clone {
    micro clone(self) -> Self
}

trait Debug {
    micro debug_fmt(self) -> utf8 {
        # 默认实现
        f"{}@{:p}".format(self.type_name(), self)
    }
}
```

### 基本实现 (imply)

```valkyrie
class Point {
    x: f64,
    y: f64,
}

imply Point: Display {
    micro fmt(self) -> utf8 {
        f"({}, {})".format(self.x, self.y)
    }
}

imply Point: Clone {
    micro clone(self) -> Self {
        Point { x: self.x, y: self.y }
    }
}
```

## 进阶定义

随着抽象需求的增加，具名 `trait` 可以包含关联类型、继承其他 Trait 或添加约束。这些能力都依赖 trait 的协议身份，因此不属于匿名 row。

### 带关联类型的 Trait

```valkyrie
trait Iterator {
    type Item
    
    micro next(self) -> Self::Item?
    
    micro collect⟨C: FromIterator⟨Self::Item⟩⟩(self) -> C {
        C::from_iter(self)
    }
}

trait FromIterator⟨T⟩ {
    micro from_iter⟨I: Iterator⟨Item = T⟩⟩(iter: I) -> Self
}
```

### Trait 继承与约束

```valkyrie
trait PartialEq⟨Rhs = Self⟩ {
    micro eq(self, other: Rhs) -> bool
    
    micro ne(self, other: Rhs) -> bool {
        !self.eq(other)
    }
}

# Ord 继承了 PartialEq 和 PartialOrd
trait Ord: PartialEq + PartialOrd {
    micro cmp(self, other: Self) -> Ordering
}
```

## 泛型与条件实现

泛型允许 Trait 处理多种类型，而条件实现则允许根据类型满足的约束来提供实现。

### 泛型实现

泛型使用数学角括号 `⟨ ⟩`。

```valkyrie
imply⟨T: Display⟩ [T]: Display {
    micro fmt(self) -> utf8 {
        let items = self.iter()
            .map { %.fmt() }
            .collect::⟨[utf8]⟩()
            .join(", ")
        f"[{}]".format(items)
    }
}

imply⟨T: Clone⟩ [T]: Clone {
    micro clone(self) -> Self {
        self.iter().map { %.clone() }.collect()
    }
}
```

### 条件实现

```valkyrie
imply⟨T: PartialEq⟩ [T]: PartialEq {
    micro eq(self, other: Self) -> bool {
        self.length == other.length && 
        self.iter().zip(other.iter()).all { %1 == %2 }
    }
}
```

## Trait 对象与动态分发

在某些情况下，我们希望处理一组实现了相同 Trait 的不同类型。这时可以使用 **Trait 对象**。

### Trait 对象示例

```valkyrie
trait Animal {
    micro make_sound(self)
    micro name(self) -> utf8
}

class Dog {
    name: utf8,
}

imply Dog: Animal {
    micro make_sound(self) {
        print("Woof!")
    }
    
    micro name(self) -> utf8 {
        self.name.clone()
    }
}

class Cat {
    name: utf8,
}

imply Cat: Animal {
    micro make_sound(self) {
        print("Meow!")
    }
    
    micro name(self) -> utf8 {
        self.name.clone()
    }
}

# 使用 trait 对象：将不同类型放入同一个数组
let animals: [Animal] = [
    Dog { name: "Buddy" },
    Cat { name: "Whiskers" },
]

loop animal in animals {
    print("{} says:", animal.name())
    animal.make_sound() # 动态分发
}
```

## 底层原理：见证表 (Witness Table)

Valkyrie 的动态分发机制依赖于 **见证表 (Witness Table)**。这使得 Valkyrie 能够突破传统静态多态的限制。

这里的“见证”只属于具名 `trait/imply` 体系，不属于匿名 row 约束。

### 什么是“见证”？ (Witnessing Existence)

从类型论的角度看，Trait 系统实际上是 **存在量化 (Existential Quantification)** 的一种体现：

- **泛型是全称量化 ($\forall T$)**: 调用者拥有选择 $T$ 的权力，被调用的函数必须能够处理调用者给出的任何 $T$。
- **接口是存在量化 ($\exists T$)**: 构造者拥有选择 $T$ 的权力，而调用者只知道“存在一个类型 $T$ 满足某些约束”，但并不知道 $T$ 具体是谁。

在存在命题 $\exists T. P(T)$ 中，那个使得命题为真的具体类型 $T$ 就被称为 **见证类型 (Witness Type)**。

### 存在类型的传统困境

在许多静态语言（例如 Rust）中，当你将一个对象向上转型为 Trait 对象时，具体的见证类型信息会被完全擦除。这通常会导致：

1.  **内存布局丢失**: 编译器不再知道原始类型的 Size 和 Alignment，导致无法在栈上直接处理这些对象。
2.  **方法调用限制**: 诸如返回 `Self` 或带有泛型参数的方法通常无法在动态分发中使用，因为调用者失去了“恢复”这些信息的能力。

### Valkyrie 的解决方案：Open Existential Types

Valkyrie 通过 **见证表 (Witness Table)** 彻底解决了上述困境。见证表不仅是方法的索引，它还包含了类型的 **全量元数据**。

- **解包 (Open)**: 见证表指导运行时如何“打开”这个不透明的存在类型，重新获得其内存布局和类型信息。
- **全动态支持**: 这种设计使得 Valkyrie 的 Trait 对象**没有所谓的“对象安全”限制**。即使是返回 `Self` 的方法，也可以在动态分发中安全使用，因为运行时可以通过见证表动态处理返回值。

### 转化原理

在编译阶段，Valkyrie 编译器会将 Trait 定义和其实现转化为底层的见证表结构：

1. **Trait 定义 -> 布局模板**: 编译器为每个 Trait 定义一个方法列表模板。
2. **imply 实现 -> 见证表实例**: 对于每一个 `imply Class: Trait` 块，编译器会生成一个具体的见证表实例。该表包含了指向该类具体方法实现的指针（或字节码索引）。
3. **Trait 对象 -> 胖指针**: 运行时的 Trait 对象实际上是一个“胖指针”，由两部分组成：
    - 指向具体数据实例的指针。
    - 指向对应见证表的指针。

### 见证表 (Witness Table) 与 vtable 的区别

虽然见证表和传统的虚函数表 (vtable) 都用于实现多态，但它们在设计哲学和内存布局上有显著区别：

| 特性 | vtable (传统 OOP) | wtable (见证表) |
| :--- | :--- | :--- |
| **绑定关系** | **强绑定**：嵌入在类定义的继承层级中，通常要求类型在定义时就声明接口。 | **弱绑定**：独立于类定义，在 `imply` 块中定义，支持后期绑定。 |
| **内存布局** | **侵入式**：每个对象实例内部通常包含一个指向其类 vtable 的指针。 | **非侵入式**：对象本身保持原始布局。多态由外部的“胖指针”携带见证表实现。 |
| **扩展性** | **闭合**：通常无法为已有的类在外部添加新的虚函数或接口实现。 | **开放**：可以随时为任何已存在的类型（包括基本类型）实现新的 Trait。 |
| **多继承处理** | **复杂**：需要处理复杂的偏移量计算或维护多个 vtable 指针。 | **扁平**：每个 Trait 对应一个独立的见证表，组合逻辑清晰且性能一致。 |

### 见证表的优势

1.  **非侵入式扩展 (Retrofitting)**: 
    你可以为第三方库中的类型实现自己的 Trait。这打破了传统 OOP 中“先有父类再有子类”的层级束缚。
2.  **二进制兼容性 (ABI Stability)**: 
    为类添加新的 Trait 实现不会改变类的内存布局。这意味着即使库升级添加了新功能，旧的二进制代码依然可以安全地访问该对象。
3.  **零开销潜力**: 
    在泛型实例化阶段，编译器可以通过单态化 (Monomorphization) 直接消除见证表，实现与静态调用一致的性能。
4.  **按需分发**: 
    只有在代码真正需要将对象作为 `Trait 对象` 处理时，才会生成胖指针。在普通调用中，完全没有虚表带来的间接开销。

### 运行时示例 (Bytecode Level)

```valkyrie
# 对于 Dog 的实现，编译器生成一个 Animal_for_Dog 的见证表实例
imply Dog: Animal { ... }

# 当执行此函数时：
micro shout(animal: Animal) {
    # 1. 从 animal 胖指针中获取见证表
    # 2. 从见证表中查找 make_sound 的索引
    # 3. 调用该索引对应的具体实现
    animal.make_sound()
}
```

## 匿名 row 与具名 trait 的区别

Valkyrie 支持匿名方法约束，但它们在语义上应理解为 `row requirement`，而不是“没有名字的 trait”。

### 匿名 row 约束

```valkyrie
micro process_drawable(drawable: {
    draw() -> unit,
    get_bounds() -> Rectangle,
}) {
    let bounds = drawable.get_bounds()
    print("Drawing object with bounds: {}", bounds)
    drawable.draw()
}
```

这类约束的特点是：

- 只检查方法行是否满足
- 不生成具名 witness
- 不支持关联类型
- 不支持默认实现

### 什么时候应该改用具名 trait

如果你需要下面这些能力，就不该继续用匿名 row，而应定义具名 `trait`：

- `associated type`
- 默认方法体
- trait 继承
- trait 对象
- 明确的协议身份与 witness

```valkyrie
trait Drawable {
    micro draw(self)
    micro get_bounds(self) -> Rectangle
}

micro process_drawable⟨T: Drawable⟩(drawable: T) {
    let bounds = drawable.get_bounds()
    print("Drawing object with bounds: {}", bounds)
    drawable.draw()
}
```

## 高级特性

### 关联常量

```valkyrie
trait MathConstants {
    const PI: f64 = 3.14159265359
    const E: f64 = 2.71828182846
    
    micro circle_area(radius: f64) -> f64 {
        Self::PI * radius * radius
    }
}

class Calculator {}

imply Calculator: MathConstants {}

let area = Calculator::circle_area(5.0)
```

### Trait 别名

```valkyrie
# 定义 trait 别名
trait Printable = Display + Debug + Clone

# 使用 trait 别名
micro print_info⟨T: Printable⟩(item: T) {
    print("Display: {}", item.fmt())
    print("Debug: {}", item.debug_fmt())
    let cloned = item.clone()
    print("Cloned: {}", cloned.fmt())
}
```

## 派生宏

Valkyrie 提供了自动派生常用 trait 的宏：

```valkyrie
@derive(Debug, Clone, PartialEq, Eq, Hash)
class User {
    id: u64,
    name: utf8,
    email: utf8,
}

@derive(Display)
class Point {
    x: f64,
    y: f64,
}

# 自定义派生行为
@derive(Debug, Clone)
@derive_display(format = "User({})", field = "name")
class SimpleUser {
    name: utf8,
    internal_id: u64,  # 不会在 Display 中显示
}
```

## 最佳实践

### 1. Trait 设计原则

```valkyrie
# 好的设计：单一职责
trait Readable {
    micro read(mut self, buffer: mut [u8]) -> Result⟨usize, Error⟩
}

trait Writable {
    micro write(self, data: [u8]) -> Result⟨usize, Error⟩
}

# 组合使用
trait ReadWrite: Readable + Writable {}
```

### 2. 使用关联类型 vs 泛型参数

```valkyrie
# 使用关联类型：每个类型只有一个实现
trait Iterator {
    type Item
    micro next(mut self) -> Self::Item?
}

# 使用泛型参数：可以有多个实现
trait From⟨T⟩ {
    micro from(value: T) -> Self
}

# utf8 可以从多种类型转换
imply utf8: From⟨utf8⟩ { ... }
imply utf8: From⟨char⟩ { ... }
imply utf8: From⟨[char]⟩ { ... }
```

### 3. 错误处理

```valkyrie
trait TryFrom⟨T⟩ {
    type Error
    
    micro try_from(value: T) -> Result⟨Self, Self::Error⟩
}

trait TryInto⟨T⟩ {
    type Error
    
    micro try_into(self) -> Result⟨T, Self::Error⟩
}

# 自动实现
imply⟨T, U⟩ T: TryInto⟨U⟩ 
where U: TryFrom⟨T⟩ {
    type Error = U::Error
    
    micro try_into(self) -> Result⟨U, Self::Error⟩ {
        U::try_from(self)
    }
}
```

## 总结

Valkyrie 的 trait 系统提供了：

1. **灵活的抽象**：通过 trait 定义行为接口
2. **代码复用**：通过默认实现和泛型
3. **类型安全**：编译时检查 trait 边界
4. **动态分发**：通过 trait 对象支持运行时多态
5. **协议身份**：通过具名 witness 保留“为何满足该 trait”的事实
6. **与匿名 row 分层**：把轻量方法约束与真正的协议系统分开

正确使用 trait 系统可以编写出既灵活又类型安全的代码。
