# 抽象类 (Abstract Classes)

## 概述

抽象类是一种不能被直接实例化的类，它用于定义子类必须实现的抽象方法。抽象类是介于普通类和 Trait 之间的中间形态：它既可以包含抽象方法（像 Trait），也可以包含具体实现（像普通类）。

## 定义抽象类

使用 `abstract class` 关键字定义抽象类：

```valkyrie
abstract class Shape {
    # 抽象属性 - 子类必须实现
    abstract get area(self) -> f64
    abstract get perimeter(self) -> f64
    
    # 抽象方法 - 子类必须实现
    abstract micro draw(self)
    
    # 具体方法 - 子类直接继承
    micro describe(self) {
        print("面积: {self.area}, 周长: {self.perimeter}")
    }
    
    # 具体方法 - 可被子类重写
    micro scale(mut self, factor: f64) {
        print("缩放因子: {factor}")
    }
}
```

## 实现抽象类

子类必须实现所有抽象成员：

```valkyrie
class Circle(Shape) {
    radius: f64
    
    # 实现抽象属性
    override get area(self) -> f64 {
        3.14159 * self.radius * self.radius
    }
    
    override get perimeter(self) -> f64 {
        2.0 * 3.14159 * self.radius
    }
    
    # 实现抽象方法
    override micro draw(self) {
        print("绘制半径为 {self.radius} 的圆")
    }
}

class Rectangle(Shape) {
    width: f64
    height: f64
    
    override get area(self) -> f64 {
        self.width * self.height
    }
    
    override get perimeter(self) -> f64 {
        2.0 * (self.width + self.height)
    }
    
    override micro draw(self) {
        print("绘制 {self.width}x{self.height} 的矩形")
    }
}
```

## 抽象类 vs Trait

| 特性 | 抽象类 | Trait |
|:---|:---|:---|
| **实例化** | 不能直接实例化 | 不能实例化 |
| **具体方法** | ✅ 支持 | ✅ 支持（默认实现） |
| **字段** | ✅ 支持实例字段 | ❌ 不支持实例字段 |
| **多继承** | ❌ 单继承 | ✅ 可实现多个 |
| **构造函数** | ✅ 支持 | ❌ 不支持 |
| **访问控制** | ✅ 完整支持 | ⚠️ 有限支持 |

### 选择指南

```valkyrie
# 使用抽象类：当需要共享状态和构造逻辑
abstract class Vehicle {
    # 共享状态
    speed: f64 = 0.0
    max_speed: f64
    
    # 构造逻辑
    initiate(mut self, max_speed: f64) {
        self.max_speed = max_speed
    }
    
    # 共享行为
    micro accelerate(mut self, delta: f64) {
        self.speed = (self.speed + delta).min(self.max_speed)
    }
    
    # 子类必须实现
    abstract micro start(self)
    abstract micro stop(self)
}

# 使用 Trait：当只需要定义行为契约
trait Movable {
    micro move_to(self, x: f64, y: f64)
    micro get_position(self) -> (f64, f64)
}

trait Drawable {
    micro draw(self)
    micro set_color(self, color: Color)
}
```

## 抽象属性

抽象类可以定义抽象属性，要求子类提供 getter 或 setter：

```valkyrie
abstract class Entity {
    # 抽象只读属性
    abstract get id(self) -> u64
    
    # 抽象读写属性
    abstract get name(self) -> utf8
    abstract set name(mut self, value: utf8)
    
    # 具体属性（基于抽象属性）
    get display_name(self) -> utf8 {
        "[{self.id}] {self.name}"
    }
}

class User(Entity) {
    user_id: u64
    user_name: utf8
    
    override get id(self) -> u64 {
        self.user_id
    }
    
    override get name(self) -> utf8 {
        self.user_name
    }
    
    override set name(mut self, value: utf8) {
        self.user_name = value
    }
}
```

## 构造函数链

抽象类可以定义构造函数，子类必须调用：

```valkyrie
abstract class Base {
    value: i32
    
    initiate(mut self, value: i32) {
        self.value = value
    }
}

class Derived(Base) {
    extra: utf8
    
    # 子类构造函数必须调用父类构造函数
    initiate(mut self, value: i32, extra: utf8) {
        super.initiate(value)  # 调用父类构造函数
        self.extra = extra
    }
}
```

## 抽象类与多继承

抽象类可以参与多继承：

```valkyrie
abstract class Reader {
    abstract micro read(self) -> utf8
}

abstract class Writer {
    abstract micro write(self, data: utf8)
}

# 多继承抽象类
class FileProcessor(Reader, Writer) {
    path: utf8
    
    override micro read(self) -> utf8 {
        # 实现读取逻辑
    }
    
    override micro write(self, data: utf8) {
        # 实现写入逻辑
    }
}
```

## 模板方法模式

抽象类非常适合实现模板方法模式：

```valkyrie
abstract class DataProcessor {
    # 模板方法 - 定义算法骨架
    micro process(self, data: utf8) -> utf8 {
        let validated = self.validate(data)
        let transformed = self.transform(validated)
        let result = self.finalize(transformed)
        self.log(result)
        result
    }
    
    # 抽象步骤 - 子类实现
    abstract micro validate(self, data: utf8) -> utf8
    abstract micro transform(self, data: utf8) -> utf8
    abstract micro finalize(self, data: utf8) -> utf8
    
    # 可选钩子 - 子类可重写
    micro log(self, result: utf8) {
        print("处理完成: {result}")
    }
}

class JsonProcessor(DataProcessor) {
    override micro validate(self, data: utf8) -> utf8 {
        # JSON 验证逻辑
    }
    
    override micro transform(self, data: utf8) -> utf8 {
        # JSON 转换逻辑
    }
    
    override micro finalize(self, data: utf8) -> utf8 {
        # JSON 最终处理
    }
    
    override micro log(self, result: utf8) {
        # 自定义日志
    }
}
```

## 最佳实践

### 1. 合理划分抽象层次

```valkyrie
# 好的设计：清晰的抽象层次
abstract class Animal {
    name: utf8
    
    # 所有动物都有名字
    initiate(mut self, name: utf8) {
        self.name = name
    }
    
    # 抽象行为
    abstract micro make_sound(self)
    abstract micro move(self)
}

# 避免：过度抽象
abstract class Thing {
    abstract micro do_something(self)  # 太模糊
}
```

### 2. 提供有意义的默认实现

```valkyrie
abstract class Handler {
    abstract micro handle(self, request: Request) -> Response
    
    # 提供有意义的默认实现
    micro on_error(self, error: Error) -> Response {
        Response::error(500, error.message)
    }
    
    micro on_success(self, data: Any) -> Response {
        Response::json(data)
    }
}
```

### 3. 使用 protected 保护内部实现

```valkyrie
abstract class BaseService {
    protected mut cache: HashMap<utf8, Any>
    
    # 公开接口
    abstract micro get(self, key: utf8) -> Any?
    
    # 受保护的辅助方法
    protected micro cache_get(self, key: utf8) -> Any? {
        self.cache.get(key)
    }
    
    protected micro cache_set(mut self, key: utf8, value: Any) {
        self.cache.insert(key, value)
    }
}
```

## 总结

抽象类的核心特点：

1. **不可实例化**：必须通过子类继承使用
2. **混合抽象与具体**：同时支持抽象方法和具体实现
3. **状态共享**：可以定义字段和构造函数
4. **模板方法**：非常适合定义算法骨架

在需要共享状态和行为时选择抽象类，在只需要行为契约时选择 Trait。
