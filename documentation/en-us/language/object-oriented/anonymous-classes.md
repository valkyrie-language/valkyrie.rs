# Anonymous Classes (Anonymous Classes)

## Overview (Overview)

Valkyrie supports anonymous classes, allowing for the temporary definition of classes without explicit declaration when needed. Anonymous classes are particularly useful for callback functions, temporary object creation, and functional programming scenarios.

## Basic Syntax (Basic Syntax)

In Valkyrie, `class` is used both for declaring named classes and for creating anonymous object instances.

### Simple Anonymous Object (Simple Anonymous Object)

If you only need temporary structured data, you can define and instantiate it directly:

```valkyrie
# Create a simple anonymous object
let point = class {
    x: 10.0,
    y: 20.0,
    
    micro distance(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

print("Distance: {point.distance()}")
```

### As a Return Value (Recommended Approach) (As a Return Value)

To maintain code readability, it is **strongly recommended** not to write complex anonymous class definitions directly in function signatures. Instead, you should typically return a **Trait** (an existential type) and use an anonymous object internally to implement it.

```valkyrie
trait Shape {
    micro area(self) -> f64
    micro draw(self)
}

# Recommended approach: return a named Trait
micro create_circle(radius: f64) -> Shape {
    # Internally use an anonymous object to implement the Trait
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

## Anonymous Class Inheritance and Multiple Implementation (Anonymous Class Inheritance and Multiple Implementation)

Anonymous objects can serve not only as pure data structures but also as subclasses or implementations of multiple Traits.

### Implementing Multiple Traits (Implementing Multiple Traits)

Use the `class: Trait1 + Trait2 { ... }` syntax:

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

## Advanced Usage of Anonymous Classes (Advanced Usage of Anonymous Classes)

### 1. Factory Pattern and Existential Types (Factory Pattern and Existential Types)

Combined with Traits, anonymous objects can elegantly implement the factory pattern while keeping external signatures concise.

```valkyrie
trait Handler {
    micro handle(self, request: utf8) -> utf8
}

# External view only sees the Handler interface, no need to know the internal structure
micro create_handler(handler_type: utf8) -> Handler {
    match handler_type {
        case "json" => class: Handler {
            micro handle(self, request: utf8) -> utf8 {
                f"{{\"response\": \"{request}\"}}"
            }
        },
        case "xml" => class: Handler {
            micro handle(self, request: utf8) -> utf8 {
                f"<response>{request}</response>"
            }
        },
        case _ => class: Handler {
            micro handle(self, request: utf8) -> utf8 {
                f"Plain response: {request}"
            }
        }
    }
}
```

### 2. Strategy Pattern

Anonymous objects allow for the injection of different algorithmic implementations at runtime based on conditions, without needing a separate source file for each algorithm.

```valkyrie
trait SortStrategy {
    micro sort(self, data: &mut [i32])
}

micro get_sort_strategy(strategy_name: string) -> SortStrategy {
    match strategy_name {
        case "bubble" => class: SortStrategy {
            micro sort(self, data: &mut [i32]) {
                # Bubble sort implementation
                for i in 0..data.length {
                    for j in 0..(data.length - 1 - i) {
                        if data[j] > data[j + 1] {
                            data.swap(j, j + 1)
                        }
                    }
                }
            }
        },
        case "quick" => class: SortStrategy {
            micro sort(self, data: &mut [i32]) {
                # Quick sort implementation
                self.quick_sort(data, 0, data.length as i32 - 1)
            }
            
            # Helper methods can be defined inside the anonymous object
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
            micro sort(self, data: &mut [i32]) {
                data.sort()
            }
        }
    }
}
```

### 3. Builder Pattern

Using Trait constraints on returned anonymous objects enables fluent APIs without exposing internal implementation details.

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

## Anonymous Objects and Generics

Anonymous objects also support generics. You can define a generic Trait and return an anonymous object specialized according to type parameters.

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

### Constrained Generics

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

## Variable Capture

Anonymous objects can capture variables from their defining environment. Since Valkyrie is a GC-managed language, the lifecycle of these variables is automatically handled by the garbage collector.

```valkyrie
trait Counter {
    micro next(mut self) -> i32
}

micro create_counter(start: i32) -> Counter {
    # Capture external variable start
    let mut count = start
    
    class: Counter {
        micro next(mut self) -> i32 {
            count += 1
            count
        }
    }
}
```

## Best Practices

### 1. Prefer Returning Traits

As mentioned, returning an anonymous class directly significantly reduces code readability and maintainability. Always prioritize defining a Trait and returning its anonymous implementation.

### 2. Keep It Concise

Anonymous objects are suitable for "one-off" tasks with simple logic. If an anonymous object becomes too large (over 50 lines) or contains complex internal logic, consider refactoring it into a named `class`.

### 3. Structured Parameters

When handling temporary data or configuration items, using anonymous classes as **parameter types** is very useful, providing a lightweight structured constraint.

```valkyrie
micro configure_logger(config: class {
    level: string,
    output: string,
}) {
    print("Logging to {config.output} at level {config.level}")
}

# Pass an anonymous object during call
configure_logger(class {
    level: "INFO",
    output: "stdout",
})
```

### Summary of Don'ts

- **Function Return Values**: Never use an anonymous class as a return type directly.
- **Public API**: Avoid exposing anonymous classes in public interfaces.
- **Long-term Storage**: Do not store anonymous class instances for extended periods.
- **Complex Inheritance**: Avoid deep inheritance chains for anonymous classes.

### 1. Proper Use of Anonymous Classes

```valkyrie
# Good usage: temporary objects
micro process_data(processor: class {
    micro process(self, data: string) -> string
}) -> string {
    processor.process("input data")
}

# Avoid: overly complex anonymous classes
# If an anonymous class is too complex, define it as a named class
```

### 2. Keep Anonymous Classes Concise

```valkyrie
# Good design: concise anonymous class
let validator = class {
    micro validate(self, input: string) -> bool {
        !input.is_empty() && input.length <= 100
    }
}

# Avoid: overly complex anonymous classes
# class {
#     // large number of fields and methods
# }
```

### 3. Clear Type Annotations

```valkyrie
# Good practice: clear types
micro create_handler() -> class {
    micro handle(self, request: string) -> Result⟨string, Error⟩
} {
    # Implementation
}

# Avoid: ambiguous types
# micro create_handler() -> class { ... }  # Interface is unclear
```

## Summary

Valkyrie's Anonymous Class features:

1.  **Flexibility**: Create objects without pre-defining a class.
2.  **Inheritance Support**: Support for inheriting named classes and implementing Traits.
3.  **Generics Support**: Support for generic parameters and constraints.
4.  **Closure Distinction**: Clearly distinguished from closure syntax.
5.  **Pattern Support**: Applicable to design patterns like Factory, Strategy, and Builder.

Anonymous classes are especially suitable for scenarios requiring temporary objects, callback handling, and functional programming.
