# 匿名类 (Anonymous Classes)

## 概述

Valkyrie 支持匿名类，允许在需要时临时定义类而无需显式声明。匿名类特别适用于回调函数、临时对象创建和函数式编程场景。

## 基本匿名类语法

### 简单匿名类

```valkyrie
# 匿名类语法：class { 字段和方法定义 }
micro create_point() -> class {
    x: f64,
    y: f64,
    
    micro distance_from_origin(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
} {
    class {
        x: 10.0,
        y: 20.0,
        
        micro distance_from_origin(self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }
}

# 使用匿名类
let point = create_point()
let distance = point.distance_from_origin()
println("Distance: {}", distance)
```

### 匿名类作为参数

```valkyrie
# 接受匿名类作为参数
micro process_drawable(drawable: class {
    micro draw(self)
    micro get_area(self) -> f64
}) {
    println("Area: {}", drawable.get_area())
    drawable.draw()
}

# 传递匿名类实例
process_drawable(class {
    radius: f64,
    
    micro draw(self) {
        println("Drawing circle with radius {}", self.radius)
    }
    
    micro get_area(self) -> f64 {
        3.14159 * self.radius * self.radius
    }
} { radius: 5.0 })
```

### 匿名类与闭包的区别

```valkyrie
# 闭包语法：{ 参数 表达式 }
let closure = { $x $x * 2 }
let result = closure(5)  # 结果：10

# 匿名类语法：class { 字段和方法 }
let anonymous_obj = class {
    multiplier: i32,
    
    micro multiply(self, x: i32) -> i32 {
        x * self.multiplier
    }
} { multiplier: 2 }

let result = anonymous_obj.multiply(5)  # 结果：10
```

## 匿名类继承

### 继承具名类

```valkyrie
class Shape {
    color: String,
    
    micro set_color(mut self, color: String) {
        self.color = color
    }
    
    micro get_color(self) -> String {
        self.color.clone()
    }
}

# 匿名类继承具名类
micro create_circle(radius: f64) -> class(Shape) {
    radius: f64,
    
    micro area(self) -> f64 {
        3.14159 * self.radius * self.radius
    }
    
    micro draw(self) {
        println("Drawing {} circle with radius {}", 
                self.get_color(), self.radius)
    }
} {
    class(Shape) {
        color: "red".to_string(),
        radius: radius,
        
        micro area(self) -> f64 {
            3.14159 * self.radius * self.radius
        }
        
        micro draw(self) {
            println("Drawing {} circle with radius {}", 
                    self.get_color(), self.radius)
        }
    }
}
```

### 多重继承的匿名类

```valkyrie
trait Drawable {
    micro draw(self)
}

trait Movable {
    micro move_to(mut self, x: f64, y: f64)
    micro get_position(self) -> (f64, f64)
}

class GameObject {
    id: u32,
    
    micro get_id(self) -> u32 {
        self.id
    }
}

# 匿名类多重继承
micro create_sprite() -> class(GameObject): Drawable + Movable {
    x: f64,
    y: f64,
    sprite_name: String,
} {
    class(GameObject): Drawable + Movable {
        id: 1001,
        x: 0.0,
        y: 0.0,
        sprite_name: "player".to_string(),
        
        micro draw(self) {
            println("Drawing sprite '{}' at ({}, {})", 
                    self.sprite_name, self.x, self.y)
        }
        
        micro move_to(mut self, x: f64, y: f64) {
            self.x = x
            self.y = y
        }
        
        micro get_position(self) -> (f64, f64) {
            (self.x, self.y)
        }
    }
}
```

## 匿名类的高级用法

### 工厂模式

```valkyrie
# 使用匿名类实现工厂模式
micro create_handler(handler_type: String) -> class {
    micro handle(self, request: String) -> String
} {
    match handler_type {
        case "json" => class {
            micro handle(self, request: String) -> String {
                @format("{{\"response\": \"{}\"}}", request)
            }
        },
        case "xml" => class {
            micro handle(self, request: String) -> String {
                @format("<response>{}</response>", request)
            }
        },
        case _ => class {
            micro handle(self, request: String) -> String {
                @format("Plain response: {}", request)
            }
        }
    }
}

let json_handler = create_handler("json")
let response = json_handler.handle("Hello World")
```

### 策略模式

```valkyrie
# 策略接口
trait SortStrategy {
    micro sort(self, data: &mut Vec<i32>)
}

# 使用匿名类实现不同策略
micro get_sort_strategy(strategy_name: String) -> class: SortStrategy {
    match strategy_name {
        case "bubble" => class: SortStrategy {
            micro sort(self, data: &mut Vec<i32>) {
                # 冒泡排序实现
                for i in 0..data.len() {
                    for j in 0..(data.len() - 1 - i) {
                        if data[j] > data[j + 1] {
                            data.swap(j, j + 1)
                        }
                    }
                }
            }
        },
        case "quick" => class: SortStrategy {
            micro sort(self, data: &mut Vec<i32>) {
                # 快速排序实现
                self.quick_sort(data, 0, data.len() as i32 - 1)
            }
            
            micro quick_sort(self, data: &mut Vec<i32>, low: i32, high: i32) {
                if low < high {
                    let pi = self.partition(data, low, high)
                    self.quick_sort(data, low, pi - 1)
                    self.quick_sort(data, pi + 1, high)
                }
            }
            
            micro partition(self, data: &mut Vec<i32>, low: i32, high: i32) -> i32 {
                # 分区实现
                let pivot = data[high as usize]
                let mut i = low - 1
                
                for j in low..high {
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
            micro sort(self, data: &mut Vec<i32>) {
                data.sort()  # 使用默认排序
            }
        }
    }
}
```

### 建造者模式

```valkyrie
# 使用匿名类实现建造者模式
micro create_config_builder() -> class {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u32>,
    
    micro set_host(mut self, host: String) -> Self {
        self.host = Some(host)
        self
    }
    
    micro set_port(mut self, port: u16) -> Self {
        self.port = Some(port)
        self
    }
    
    micro set_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout)
        self
    }
    
    micro build(self) -> Config {
        Config {
            host: self.host.unwrap_or("localhost".to_string()),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        }
    }
} {
    class {
        host: None,
        port: None,
        timeout: None,
        
        micro set_host(mut self, host: String) -> Self {
            self.host = Some(host)
            self
        }
        
        micro set_port(mut self, port: u16) -> Self {
            self.port = Some(port)
            self
        }
        
        micro set_timeout(mut self, timeout: u32) -> Self {
            self.timeout = Some(timeout)
            self
        }
        
        micro build(self) -> Config {
            Config {
                host: self.host.unwrap_or("localhost".to_string()),
                port: self.port.unwrap_or(8080),
                timeout: self.timeout.unwrap_or(30),
            }
        }
    }
}

# 使用建造者
let config = create_config_builder()
    .set_host("example.com".to_string())
    .set_port(9000)
    .set_timeout(60)
    .build()
```

## 匿名类与泛型

### 泛型匿名类

```valkyrie
# 泛型匿名类
micro create_container<T>(value: T) -> class {
    value: T,
    
    micro get(self) -> &T {
        &self.value
    }
    
    micro set(mut self, new_value: T) {
        self.value = new_value
    }
} {
    class {
        value: value,
        
        micro get(self) -> &T {
            &self.value
        }
        
        micro set(mut self, new_value: T) {
            self.value = new_value
        }
    }
}

let string_container = create_container("Hello".to_string())
let number_container = create_container(42)
```

### 约束泛型匿名类

```valkyrie
# 带约束的泛型匿名类
micro create_comparable_pair<T: PartialOrd + Clone>(a: T, b: T) -> class {
    first: T,
    second: T,
    
    micro max(self) -> T {
        if self.first > self.second {
            self.first.clone()
        } else {
            self.second.clone()
        }
    }
    
    micro min(self) -> T {
        if self.first < self.second {
            self.first.clone()
        } else {
            self.second.clone()
        }
    }
} {
    class {
        first: a,
        second: b,
        
        micro max(self) -> T {
            if self.first > self.second {
                self.first.clone()
            } else {
                self.second.clone()
            }
        }
        
        micro min(self) -> T {
            if self.first < self.second {
                self.first.clone()
            } else {
                self.second.clone()
            }
        }
    }
}
```

## 匿名类的生命周期

### 捕获外部变量

```valkyrie
micro create_counter(initial: i32) -> class {
    count: i32,
    
    micro increment(mut self) -> i32 {
        self.count += 1
        self.count
    }
    
    micro get_count(self) -> i32 {
        self.count
    }
} {
    class {
        count: initial,  # 捕获外部变量
        
        micro increment(mut self) -> i32 {
            self.count += 1
            self.count
        }
        
        micro get_count(self) -> i32 {
            self.count
        }
    }
}

let counter = create_counter(10)
let value1 = counter.increment()  # 11
let value2 = counter.increment()  # 12
```

## 最佳实践

### 1. 适当使用匿名类

```valkyrie
# 好的用法：临时对象
micro process_data(processor: class {
    micro process(self, data: String) -> String
}) -> String {
    processor.process("input data")
}

# 避免：复杂的匿名类
# 如果匿名类过于复杂，应该定义为具名类
```

### 2. 保持匿名类简洁

```valkyrie
# 好的设计：简洁的匿名类
let validator = class {
    micro validate(self, input: String) -> bool {
        !input.is_empty() && input.len() <= 100
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
    micro handle(self, request: String) -> Result<String, Error>
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