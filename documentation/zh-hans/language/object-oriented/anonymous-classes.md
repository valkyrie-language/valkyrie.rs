# 匿名类 (Anonymous Classes)

## 概述

Valkyrie 支持匿名类，允许在需要时临时定义类而无需显式声明。匿名类特别适用于回调函数、临时对象创建和函数式编程场景。

## 基本语法

在 Valkyrie 中，`class` 既用于声明具名类，也用于创建匿名对象实例。

### 简单匿名对象

如果你只是需要一个临时的结构化数据，可以直接定义并实例化：

```valkyrie
# 创建一个简单的匿名对象
let point = class {
    x: 10.0,
    y: 20.0,
    
    micro distance(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

print("Distance: {point.distance()}")
```

### 作为返回值 (推荐方式)

为了保持代码的可读性，**强烈建议**不要在函数签名中直接书写复杂的匿名类定义。通常应当返回一个 **Trait**（即存在类型），而内部使用匿名对象来实现它。

```valkyrie
trait Shape {
    micro area(self) -> f64
    micro draw(self)
}

# 推荐写法：返回一个具名 Trait
micro create_circle(radius: f64) -> Shape {
    # 内部使用匿名对象实现 Trait
    class: Shape {
        radius: radius,
        
        micro area(self) -> f64 {
            3.14159 * self.radius * self.radius
        }
        
        micro draw(self) {
            print("Drawing circle with radius {self.radius}")
        }
    }
}
```

## 匿名类继承与多重实现

匿名对象不仅可以作为纯数据结构，还可以继承类或实现多个 Trait。

### 实现多个 Trait

使用 `class: Trait1 + Trait2 { ... }` 语法：

```valkyrie
trait Drawable { micro draw(self) }
trait Loggable { micro log(self) }

micro create_dynamic_obj() -> Drawable + Loggable {
    class: Drawable + Loggable {
        micro draw(self) { print("Drawing...") }
        micro log(self) { print("Logging...") }
    }
}
```

## 匿名类的高级用法

### 1. 工厂模式与存在类型

结合 Trait，匿名对象可以非常优雅地实现工厂模式，同时保持外部签名的简洁。

```valkyrie
trait Handler {
    micro handle(self, request: string) -> string
}

# 外部只看到 Handler 接口，不需要知道内部匿名类的具体结构
micro create_handler(handler_type: string) -> Handler {
    match handler_type {
        case "json" => class: Handler {
            micro handle(self, request: string) -> string {
                f"{{\"response\": \"{}\"}}".format(request)
            }
        },
        case "xml" => class: Handler {
            micro handle(self, request: string) -> string {
                f"<response>{}</response>".format(request)
            }
        },
        case _ => class: Handler {
            micro handle(self, request: string) -> string {
                f"Plain response: {}".format(request)
            }
        }
    }
}
```

### 2. 策略模式

匿名对象允许在运行时根据条件注入不同的算法实现，而无需为每种算法创建一个单独的源文件。

```valkyrie
trait SortStrategy {
    micro sort(self, data: &mut [i32])
}

micro get_sort_strategy(strategy_name: string) -> SortStrategy {
    match strategy_name {
        case "bubble" => class: SortStrategy {
            micro sort(self, data: &mut [i32]) {
                # 冒泡排序实现
                loop i in 0..data.length {
                    loop j in 0..(data.length - 1 - i) {
                        if data[j] > data[j + 1] {
                            data.swap(j, j + 1)
                        }
                    }
                }
            }
        },
        case "quick" => class: SortStrategy {
            micro sort(self, data: &mut [i32]) {
                # 快速排序实现
                self.quick_sort(data, 0, data.length as i32 - 1)
            }
            
            # 匿名对象内部可以定义辅助方法
            micro quick_sort(self, data: &mut [i32], low: i32, high: i32) {
                if low < high {
                    let pi = self.partition(data, low, high)
                    self.quick_sort(data, low, pi - 1)
                    self.quick_sort(data, pi + 1, high)
                }
            }
            
            micro partition(self, data: &mut [i32], low: i32, high: i32) -> i32 {
                let pivot = data[high as usize]
                let mut i = low - 1
                loop j in low..high {
                    if data[j as usize] <= pivot {
                        i += 1
                        data.swap(i as usize, j as usize)
                    }
                }
                data.swap((i + 1) as usize, high as usize)
                i + 1
            }
        },
        case _ => class: SortStrategy {
            micro sort(self, data: &mut [i32]) {
                data.sort()
            }
        }
    }
}
```

### 3. 建造者模式

使用 Trait 约束返回的匿名对象，可以实现流式 API 而不暴露内部实现细节。

```valkyrie
trait ConfigBuilder {
    micro set_host(mut self, host: string) -> Self
    micro set_port(mut self, port: u16) -> Self
    micro build(self) -> Config
}

micro create_config_builder() -> ConfigBuilder {
    class: ConfigBuilder {
        host: "localhost",
        port: 8080,
        
        micro set_host(mut self, host: string) -> Self {
            self.host = host
            self
        }
        
        micro set_port(mut self, port: u16) -> Self {
            self.port = port
            self
        }
        
        micro build(self) -> Config {
            Config { host: self.host, port: self.port }
        }
    }
}
```

## 匿名对象与泛型

匿名对象同样支持泛型。你可以定义一个泛型 Trait，并返回一个根据类型参数特化的匿名对象。

```valkyrie
trait Container⟨T⟩ {
    micro get(self) -> T
    micro set(mut self, value: T)
}

micro create_container⟨T⟩(initial: T) -> Container⟨T⟩ {
    class: Container⟨T⟩ {
        value: initial,
        
        micro get(self) -> T {
            self.value
        }
        
        micro set(mut self, value: T) {
            self.value = value
        }
    }
}
```

### 约束泛型

```valkyrie
trait ComparablePair⟨T⟩ {
    micro max(self) -> T
}

micro create_pair⟨T: PartialOrd + Clone⟩(a: T, b: T) -> ComparablePair⟨T⟩ {
    class: ComparablePair⟨T⟩ {
        first: a,
        second: b,
        
        micro max(self) -> T {
            if self.first > self.second { self.first.clone() }
            else { self.second.clone() }
        }
    }
}
```

## 变量捕获

匿名对象可以捕获其定义环境中的变量。由于 Valkyrie 是 GC 语言，这些变量的生命周期由垃圾回收器自动管理。

```valkyrie
trait Counter {
    micro next(mut self) -> i32
}

micro create_counter(start: i32) -> Counter {
    # 捕获外部变量 start
    let mut count = start
    
    class: Counter {
        micro next(mut self) -> i32 {
            count += 1
            count
        }
    }
}
```
## 最佳实践

### 1. 优先返回 Trait

如前所述，直接将匿名类作为返回类型会极大降低代码的可读性和可维护性。始终优先定义一个 Trait，并返回该 Trait 的匿名实现。

### 2. 保持简洁

匿名对象适合处理逻辑简单的“一次性”任务。如果一个匿名对象变得过于庞大（超过 50 行），或者包含复杂的内部逻辑，应当考虑将其重构为一个具名的 `class`。

### 3. 结构化参数

在处理临时数据或配置项时，匿名类作为 **参数类型** 是非常有用的，这提供了一种轻量级的结构化约束。

```valkyrie
micro configure_logger(config: class {
    level: string,
    output: string,
}) {
    print("Logging to {config.output} at level {config.level}")
}

# 调用时传入匿名对象
configure_logger(class {
    level: "INFO",
    output: "stdout",
})
```

- **函数返回值**：永远不要将匿名类作为返回值
- **公共 API**：避免在公共接口中暴露匿名类
- **长期存储**：不要将匿名类实例长期保存
- **复杂继承**：避免复杂的匿名类继承链

### 1. 适当使用匿名类

```valkyrie
# 好的用法：临时对象
micro process_data(processor: class {
    micro process(self, data: string) -> string
}) -> string {
    processor.process("input data")
}

# 避免：复杂的匿名类
# 如果匿名类过于复杂，应该定义为具名类
```

### 2. 保持匿名类简洁

```valkyrie
# 好的设计：简洁的匿名类
let validator = class {
    micro validate(self, input: string) -> bool {
        !input.is_empty() && input.length <= 100
    }
}

# 避免：过于复杂的匿名类
# class {
#     // 大量字段和方法
# }
```

### 3. 明确类型注解

```valkyrie
# 好的做法：明确类型
micro create_handler() -> class {
    micro handle(self, request: string) -> Result⟨string, Error⟩
} {
    # 实现
}

# 避免：模糊的类型
# micro create_handler() -> class { ... }  # 不清楚接口
```

## 总结

Valkyrie 的匿名类特性：

1. **灵活性**：无需预先定义类即可创建对象
2. **继承支持**：支持继承具名类和实现 trait
3. **泛型支持**：支持泛型参数和约束
4. **闭包区分**：与闭包语法明确区分
5. **模式支持**：适用于工厂、策略、建造者等设计模式

匿名类特别适用于需要临时对象、回调处理和函数式编程的场景。