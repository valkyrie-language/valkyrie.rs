# Aspect-Oriented Programming (AOP)

Aspect-Oriented Programming is a programming paradigm that improves code modularity by separating cross-cutting concerns (such as logging, security, transaction management) from core business logic. In Valkyrie, AOP is implemented through the Effect system, providing powerful and flexible aspect-oriented programming capabilities.

## Basic Concepts

### Aspect
An aspect is a modular unit of cross-cutting concerns, containing code that executes at specific join points.

### Join Point
A join point is a point in program execution where aspect code can be inserted, such as method calls, field access, etc.

### Advice
Advice is the code executed at join points, including before advice, after advice, around advice, etc.

## AOP in the Effect System

### Defining Aspect Effects

```valkyrie
# Define logging aspect Effect
effect LogAspect {
    before_method(class_name: string, method_name: string, args: [Any]): Unit
    after_method(class_name: string, method_name: string, result: Any): Unit
    on_error(class_name: string, method_name: string, error: Any): Unit
}

# Define performance monitoring aspect Effect
effect MetricsAspect {
    start_timing(operation: string): TimingContext
    end_timing(context: TimingContext): Duration
    record_metric(name: string, value: f64): Unit
}
```

### Implementing Aspect Handlers

```valkyrie
# Logging aspect handler
class ConsoleLogHandler {
    handle LogAspect {
        before_method(class_name, method_name, args) {
            print("→ ${class_name}.${method_name}(${args})")
        }
        
        after_method(class_name, method_name, result) {
            print("← ${class_name}.${method_name} = ${result}")
        }
        
        on_error(class_name, method_name, error) {
            print("× ${class_name}.${method_name} throws ${error}")
        }
    }
}

# Performance monitoring handler
class MetricsHandler {
    private mut timings: {string: DateTime} = {}
    
    handle MetricsAspect {
        start_timing(operation) -> TimingContext {
            let start_time = now()
            self.timings[operation] = start_time
            TimingContext { operation, start_time }
        }
        
        end_timing(context) -> Duration {
            let end_time = now()
            let duration = end_time - context.start_time
            self.timings.remove(context.operation)
            duration
        }
        
        record_metric(name, value) {
            metrics_store.record(name, value)
        }
    }
}
```

### Using Aspect Annotations

```valkyrie
class PaymentService {
    # Use multiple aspects
    @around(LogAspect, MetricsAspect)
    micro process_payment(self, order_id: string, amount: Decimal) -> Receipt {
        # Before advice executes automatically
        perform LogAspect.before_method("PaymentService", "process_payment", [order_id, amount])
        let timing_ctx = perform MetricsAspect.start_timing("payment_processing")
        
        try {
            # Core business logic
            let receipt = self.do_payment(order_id, amount)
            
            # After advice
            perform LogAspect.after_method("PaymentService", "process_payment", receipt)
            let duration = perform MetricsAspect.end_timing(timing_ctx)
            perform MetricsAspect.record_metric("payment_duration", duration.as_millis())
            
            receipt
        }
        .catch {
            case _:
                perform LogAspect.on_error("PaymentService", "process_payment", error)
                raise error
        }
    }
    
    private micro do_payment(self, order_id: string, amount: Decimal) -> Receipt {
        # Actual payment logic
        Receipt { id: generate_id(), order_id, amount, timestamp: now() }
    }
}
```

### Composing Aspects

```valkyrie
# Define composite aspect
effect AuditAspect {
    log_access(user_id: string, resource: string, action: string): Unit
    log_data_change(table: string, record_id: string, old_value: Any, new_value: Any): Unit
}

# Security audit service
class SecurityAuditService {
    @around(LogAspect, AuditAspect)
    micro update_user_profile(self, user_id: string, profile: UserProfile) -> Unit {
        perform AuditAspect.log_access(user_id, "user_profile", "update")
        
        let old_profile = self.get_user_profile(user_id)
        
        # Update logic
        self.save_user_profile(user_id, profile)
        
        perform AuditAspect.log_data_change("users", user_id, old_profile, profile)
    }
}
```

### Conditional Aspects

```valkyrie
# Define conditional aspect Effect
effect ConditionalAspect {
    should_apply(context: AspectContext) -> bool
    apply_advice(context: AspectContext): Unit
}

class DebugAspect {
    handle ConditionalAspect {
        should_apply(context) -> bool {
            # Only apply in debug mode
            config.debug_mode && context.method_name.starts_with("debug_")
        }
        
        apply_advice(context) {
            print(f"Debug: {context.class_name}.{context.method_name}")
        }
    }
}
```

### Aspect Combinator

```valkyrie
# Aspect combinator
class AspectComposer {
    micro compose_aspects⟨T⟩(aspects: [Effect]) -> Effect {
        effect ComposedAspect {
            execute(context: AspectContext): T
        }
        
        handle ComposedAspect {
            execute(context) -> T {
                # Execute all aspects' before advice in order
                for aspect in aspects {
                    perform aspect.before(context)
                }
                
                try {
                    let result = context.proceed()
                    
                    # Execute all aspects' after advice in reverse order
                    for aspect in aspects.reverse() {
                        perform aspect.after(context, result)
                    }
                    
                    result
                }
                .catch {
                    case _:
                        # Execute exception advice
                        for aspect in aspects.reverse() {
                            perform aspect.on_error(context, error)
                        }
                        raise error
                }
            }
        }
    }
}
```

## Advanced Features

### Dynamic Aspects

```valkyrie
# Dynamic aspect manager
class DynamicAspectManager {
    private mut aspects: [Effect] = []
    
    micro add_aspect(self, aspect: Effect) {
        self.aspects.push(aspect)
    }
    
    micro remove_aspect(self, aspect: Effect) {
        self.aspects.retain { $ != aspect }
    }
    
    @around(DynamicAspect)
    micro execute_with_aspects<T>(self, operation: { -> T }) -> T {
        let context = AspectContext::new()
        
        # Dynamically apply all registered aspects
        for aspect in self.aspects {
            perform aspect.before(context)
        }
        
        try {
            let result = operation()
            
            for aspect in self.aspects.reverse() {
                perform aspect.after(context, result)
            }
            
            result
        }
        .catch {
            case _:
                for aspect in self.aspects.reverse() {
                    perform aspect.on_error(context, error)
                }
                raise error
        }
    }
}
```

### Aspect Chain

```valkyrie
# Aspect chain processing
class AspectChain {
    private aspects: [Effect]
    private mut current_index: usize = 0
    
    micro proceed<T>(mut self, context: AspectContext) -> T {
        if self.current_index >= self.aspects.length {
            # Execute original method
            context.target_method()
        } else {
            let aspect = self.aspects[self.current_index]
            self.current_index += 1
            
            # Execute current aspect
            perform aspect.around(context, { self.proceed(context) })
        }
    }
}
```

## Best Practices

### 1. Single Responsibility for Aspects
Each aspect should focus on only one cross-cutting concern, avoiding mixing multiple functionalities in a single aspect.

### 2. Reasonable Aspect Ordering
When multiple aspects apply to the same join point, consider their execution order.

### 3. Avoid Strong Coupling Between Aspects
Aspects should remain independent and avoid mutual dependencies.

### 4. Performance Considerations
Aspects add overhead to method calls, use them carefully in performance-sensitive scenarios.

```valkyrie
# Example: Complete AOP application
class OrderService {
    @around(LogAspect, MetricsAspect, SecurityAspect, TransactionAspect)
    micro create_order(self, user_id: string, items: [OrderItem]) -> Order {
        # All aspects' before advice executes automatically
        # - Log method call
        # - Start performance timing
        # - Security check
        # - Begin transaction
        
        let order = Order {
            id: generate_order_id(),
            user_id,
            items,
            status: OrderStatus.Pending,
            created_at: now()
        }
        
        self.save_order(order)
        
        # All aspects' after advice executes automatically
        # - Commit transaction
        # - Record security log
        # - Record performance metrics
        # - Log method return value
        
        order
    }
}

# Usage example
let order_service = OrderService {}
let log_handler = ConsoleLogHandler {}
let metrics_handler = MetricsHandler {}

with log_handler, metrics_handler {
    let order = order_service.create_order("user123", [
        OrderItem { product_id: "p1", quantity: 2 },
        OrderItem { product_id: "p2", quantity: 1 }
    ])
    print(f"Created order: {order.id}")
}
```

AOP implemented through the Effect system provides type-safe, composable, and easily testable aspect-oriented programming capabilities, making the management of cross-cutting concerns more elegant and powerful.
