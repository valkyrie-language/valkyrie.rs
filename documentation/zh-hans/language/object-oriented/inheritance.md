# 多继承系统 (Multiple Inheritance)

## 概述 (Overview)

Valkyrie 支持多继承，允许一个类同时继承多个父类。多继承使用 C3 线性化算法来解决方法解析顺序 (Method Resolution Order, MRO) 问题，确保继承的一致性和可预测性。

需要先区分三件事：

- 继承属于 `class` 的名义层级
- `trait / imply` 属于协议层，不等同于父类继承
- 匿名 row 只是方法行约束，不应被当作“临时父类”

## 基本多继承语法 (Basic Multiple Inheritance Syntax)

### 简单多继承 (Simple Multiple Inheritance)

```valkyrie
class A {
    micro method_a(self) {
        print("Method from A")
    }
    
    micro common_method(self) {
        print("A's common method")
    }
}

class B {
    micro method_b(self) {
        print("Method from B")
    }
    
    micro common_method(self) {
        print("B's common method")
    }
}

class C {
    micro method_c(self) {
        print("Method from C")
    }
    
    micro common_method(self) {
        print("C's common method")
    }
}

# 多继承语法：class 子类(父类1, 父类2, ...)
class MultiChild(A, B, C) {
    micro own_method(self) {
        print("MultiChild's own method")
    }
}
```

### 重命名继承 (Renaming Inheritance)

当多个父类有同名方法时，可以使用重命名语法来避免冲突：

```valkyrie
class Display {
    micro show(self) {
        print("Display show")
    }
}

class Printer {
    micro show(self) {
        print("Printer show")
    }
    
    micro print(self) {
        print("Printing...")
    }
}

# 重命名继承语法：class 子类(rename: 父类, 其他父类)
class Document(rename: Display, Printer) {
    micro display_document(self) {
        # 通过重命名访问 Display 的方法
        self.rename.show()  # 调用 Display::show
        self.print()        # 调用 Printer::print
        self.show()         # 调用 Printer::show (C3 线性化的第一个匹配)
    }
}
```

### 复杂重命名场景 (Complex Renaming Scenarios)

```valkyrie
class FileReader {
    micro read(self) -> utf8 {
        "Reading from file"
    }
    
    micro close(self) {
        print("Closing file")
    }
}

class NetworkReader {
    micro read(self) -> utf8 {
        "Reading from network"
    }
    
    micro close(self) {
        print("Closing network")
    }
}

class Logger {
    micro log(self, message: utf8) {
        print("Log: {}", message)
    }
}

# 多重重命名
class HybridReader(file_reader: FileReader, net_reader: NetworkReader, Logger) {
    micro read_from_file(self) -> utf8 {
        let content = self.file_reader.read()
        self.log(f"Read from file: {}").format(content)
        content
    }
    
    micro read_from_network(self) -> utf8 {
        let content = self.net_reader.read()
        self.log(f"Read from network: {}").format(content)
        content
    }
    
    micro cleanup(self) {
        self.file_reader.close()
        self.net_reader.close()
    }
}
```

## C3 线性化算法 (C3 Linearization Algorithm)

Valkyrie 使用 C3 线性化算法来确定方法解析顺序：

```valkyrie
class A {
    micro method(self) { print("A") }
}

class B(A) {
    micro method(self) { print("B") }
}

class C(A) {
    micro method(self) { print("C") }
}

class D(B, C) {
    # 没有重写 method
}

# C3 线性化顺序：D -> B -> C -> A
# 调用 d.method() 会调用 B::method
let d = D {}
d.method()  # 输出："B"
```

### 线性化顺序示例 (Linearization Order Example)

```valkyrie
class Base {
    micro base_method(self) { print("Base") }
}

class Left(Base) {
    micro left_method(self) { print("Left") }
    micro common_method(self) { print("Left common") }
}

class Right(Base) {
    micro right_method(self) { print("Right") }
    micro common_method(self) { print("Right common") }
}

class Middle(Left, Right) {
    micro middle_method(self) { print("Middle") }
}

class Final(Middle, Right) {
    # C3 线性化：Final -> Middle -> Left -> Right -> Base
    micro test_resolution(self) {
        self.common_method()  # 调用 Left::common_method
        self.left_method()    # 调用 Left::left_method
        self.right_method()   # 调用 Right::right_method
        self.base_method()    # 调用 Base::base_method
    }
}
```

## 方法访问模式 (Method Access Patterns)

### 直接访问 (Direct Access)

```valkyrie
class Child(A, B, C) {
    micro test_access(self) {
        # 直接调用，使用 C3 线性化顺序
        self.common_method()  # 调用第一个匹配的方法
        
        # 通过重命名访问特定父类的方法
        # 注意：没有 super 关键字
    }
}
```

### 重命名访问 (Renamed Access)

```valkyrie
class AdvancedChild(primary: A, secondary: B, tertiary: C) {
    micro demonstrate_access(self) {
        # 通过重命名访问特定父类
        self.primary.common_method()    # 调用 A::common_method
        self.secondary.common_method()  # 调用 B::common_method
        self.tertiary.common_method()   # 调用 C::common_method
        
        # 直接访问使用 C3 线性化
        self.common_method()  # 调用 A::common_method (第一个)
    }
}
```

## 匿名类与匿名 row 的区别

Valkyrie 支持匿名类，但匿名类与匿名 row 不是一回事。

- 匿名类仍然属于对象层
- 匿名 row 属于方法约束层
- 如果你只是想表达“参数必须提供这些方法”，应优先写匿名 row，而不是伪装成匿名继承

### 匿名 row 约束

```valkyrie
micro process_shape(shape: {
    draw() -> unit,
    move_to(f64, f64) -> unit,
    area() -> f64,
}) {
    shape.draw()
    shape.move_to(10.0, 20.0)
    print("Area: {}", shape.area())
}
```

### 匿名类示例

当你确实需要即时定义一个类值时，才考虑匿名类：

```valkyrie
let circle = class(Drawable, Movable) {
    radius: f64,
    x: f64,
    y: f64,
    
    micro area(self) -> f64 {
        3.14159 * self.radius * self.radius
    }
}
```

### 匿名类重命名继承 (Renaming Inheritance in Anonymous Classes)

```valkyrie
# 匿名类的重命名继承
micro create_hybrid_processor() -> class(reader: FileReader, writer: FileWriter) {
    micro process(self, filename: utf8) {
        let content = self.reader.read_file(filename)
        let processed = content.to_uppercase()
        self.writer.write_file(filename + ".processed", processed)
    }
}

let processor = create_hybrid_processor() {
    # 匿名类实现
}
```

## 初始化方法 (Initialization Methods)

```valkyrie
class Parent1 {
    value1: i32,
    
    initiate(mut self, v1: i32) {
        self.value1 = v1
    }
}

class Parent2 {
    value2: utf8,
    
    initiate(mut self, v2: utf8) {
        self.value2 = v2
    }
}

class MultiInherit(Parent1, Parent2) {
    own_value: f64,
    
    initiate(mut self, v1: i32, v2: utf8, own: f64) {
        self.value1 = v1
        self.value2 = v2
        self.own_value = own
    }
}
```

## 抽象类和接口 (Abstract Classes and Traits)

```valkyrie
# 抽象基类
abstract class Shape {
    abstract micro area(self) -> f64
    abstract micro perimeter(self) -> f64
    
    # 具体方法
    micro describe(self) {
        print("Area: {}, Perimeter: {}", self.area(), self.perimeter())
    }
}

# 接口定义
trait Drawable {
    micro draw(self)
    micro set_color(self, color: Color)
}

# 多继承：抽象类 + 接口
class Rectangle(Shape): Drawable {
    width: f64,
    height: f64,
    color: Color,
    
    # 实现抽象方法
    micro area(self) -> f64 {
        self.width * self.height
    }
    
    micro perimeter(self) -> f64 {
        2.0 * (self.width + self.height)
    }
    
    # 实现接口方法
    micro draw(self) {
        print("Drawing rectangle {}x{}", self.width, self.height)
    }
    
    micro set_color(self, color: Color) {
        self.color = color
    }
}
```

这里的含义是：

- `Shape` 提供名义继承关系
- `Drawable` 提供具名协议约束
- 这两条边界不能被匿名 row 或结构兼容替代

## 钻石问题解决 (Diamond Problem Resolution)

```valkyrie
class GrandParent {
    micro method(self) { print("GrandParent") }
}

class Parent1(GrandParent) {
    micro method(self) { print("Parent1") }
}

class Parent2(GrandParent) {
    micro method(self) { print("Parent2") }
}

# 钻石继承
class Child(Parent1, Parent2) {
    # C3 线性化自动解决钻石问题
    # 线性化顺序：Child -> Parent1 -> Parent2 -> GrandParent
    
    micro test_diamond(self) {
        self.method()  # 调用 Parent1::method
    }
    
    # 如果需要调用特定父类的方法，使用重命名
}

# 使用重命名解决钻石问题
class ResolvedChild(p1: Parent1, p2: Parent2) {
    micro test_resolved(self) {
        self.p1.method()  # 明确调用 Parent1::method
        self.p2.method()  # 明确调用 Parent2::method
    }
}
```

## 最佳实践 (Best Practices)

### 1. 组合优先于继承 (Prefer Composition over Inheritance)

```valkyrie
# 好的设计：组合
class Document {
    reader: FileReader,
    writer: FileWriter,
    logger: Logger,
    
    micro process(self) {
        let content = self.reader.read()
        let processed = self.transform(content)
        self.writer.write(processed)
        self.logger.log("Document processed")
    }
}

# 避免：过度继承
# class Document(FileReader, FileWriter, Logger) { ... }
```

### 2. 使用重命名避免方法冲突 (Use Renaming to Avoid Method Conflicts)

```valkyrie
# 清晰的重命名
class MediaPlayer(audio: AudioPlayer, video: VideoPlayer) {
    micro play_audio(self, file: utf8) {
        self.audio.play(file)
    }
    
    micro play_video(self, file: utf8) {
        self.video.play(file)
    }
}
```

### 3. 文档化继承关系 (Document Inheritance Relationships)

```valkyrie
# 清晰的继承文档
↯doc("""
MultiProcessor 继承关系：
- DataProcessor: 提供数据处理能力
- NetworkHandler: 提供网络通信能力
- Logger: 提供日志记录能力

C3 线性化顺序：MultiProcessor -> DataProcessor -> NetworkHandler -> Logger
""")
class MultiProcessor(DataProcessor, NetworkHandler, Logger) {
    # 实现
}
```

## 总结 (Summary)

Valkyrie 的多继承系统特点：

1. **C3 线性化**：确保方法解析的一致性
2. **重命名机制**：解决方法名冲突
3. **无 super 关键字**：通过重命名明确访问父类方法
4. **匿名类支持**：支持临时的多继承类定义
5. **类型安全**：编译时检查继承关系的合法性

正确使用多继承可以实现灵活的代码复用，但应该谨慎使用，优先考虑组合、具名 trait 和匿名 row 的适当分层。
