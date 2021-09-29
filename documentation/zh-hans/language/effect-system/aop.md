# 面向切面编程 (Aspect-Oriented Programming)

面向切面编程（AOP）是一种编程范式，通过将横切关注点（如日志、安全、事务管理等）从核心业务逻辑中分离出来，提高代码的模块化程度。在 Valkyrie 中，AOP 通过 Effect 系统实现，提供了强大而灵活的切面编程能力。

## 基本概念

### 切面 (Aspect)
切面是横切关注点的模块化单元，包含了在特定连接点执行的代码。

### 连接点 (Join Point)
连接点是程序执行过程中可以插入切面代码的点，如方法调用、字段访问等。

### 通知 (Advice)
通知是在连接点执行的代码，包括前置通知、后置通知、环绕通知等。

## Effect 系统中的 AOP

### 定义切面 Effect

```valkyrie
# 定义日志切面消息类型
class LogBefore {
    class_name: string
    method_name: string
    args: [Any]
}

class LogAfter {
    class_name: string
    method_name: string
    result: Any
}

class LogError {
    class_name: string
    method_name: string
    error: Any
}

# 定义性能监控切面消息类型
class MetricsStart {
    operation: string
}

class MetricsEnd {
    operation: string
}

class MetricsRecord {
    name: string
    value: f64
}
```

### 实现切面处理器

```valkyrie
# 日志切面处理器
class ConsoleLogHandler {
    micro handle_log<T>(self, operation: { -> T }) -> T {
        try {
            operation()
        }
        .catch {
            case LogBefore { class_name, method_name, args }:
                print("→ {class_name}.{method_name}({args})")
                resume(())
            case LogAfter { class_name, method_name, result }:
                print("← {class_name}.{method_name} = {result}")
                resume(())
            case LogError { class_name, method_name, error }:
                print("× {class_name}.{method_name} throws {error}")
                resume(())
        }
    }
}

# 性能监控处理器
class MetricsHandler {
    private mut timings: {string: DateTime} = {}
    
    micro handle_metrics<T>(self, operation: { -> T }) -> T {
        try {
            operation()
        }
        .catch {
            case MetricsStart { operation: op }:
                let start_time = now()
                self.timings[op] = start_time
                resume(TimingContext { operation: op, start_time: start_time })
            case MetricsEnd { operation: op }:
                let end_time = now()
                let start_time = self.timings[op]
                let duration = end_time - start_time
                self.timings.remove(op)
                resume(duration)
            case MetricsRecord { name, value }:
                metrics_store.record(name, value)
                resume(())
        }
    }
}
```

### 使用切面注解

```valkyrie
class PaymentService {
    # 使用多个切面
    @around(LogAspect, MetricsAspect)
    micro process_payment(self, order_id: string, amount: Decimal) -> Receipt {
        # 前置通知自动执行
        raise LogBefore { class_name: "PaymentService", method_name: "process_payment", args: [order_id, amount] }
        let timing_ctx = raise MetricsStart { operation: "payment_processing" }
        
        try {
            # 核心业务逻辑
            let receipt = self.do_payment(order_id, amount)
            
            # 后置通知
            raise LogAfter { class_name: "PaymentService", method_name: "process_payment", result: receipt }
            let duration = raise MetricsEnd { operation: "payment_processing" }
            raise MetricsRecord { name: "payment_duration", value: duration.as_millis() }
            
            receipt
        }
        .catch {
            case error:
                raise LogError { class_name: "PaymentService", method_name: "process_payment", error: error }
                raise error
        }
    }
    
    private micro do_payment(self, order_id: string, amount: Decimal) -> Receipt {
        # 实际支付逻辑
        Receipt { id: generate_id(), order_id, amount, timestamp: now() }
    }
}
```

### 组合切面

```valkyrie
# 定义审计切面消息类型
class AuditAccess {
    user_id: string
    resource: string
    action: string
}

class AuditDataChange {
    table: string
    record_id: string
    old_value: Any
    new_value: Any
}

# 安全审计服务
class SecurityAuditService {
    @around(LogAspect, AuditAspect)
    micro update_user_profile(self, user_id: string, profile: UserProfile) -> Unit {
        raise AuditAccess { user_id, resource: "user_profile", action: "update" }
        
        let old_profile = self.get_user_profile(user_id)
        
        # 更新逻辑
        self.save_user_profile(user_id, profile)
        
        raise AuditDataChange { table: "users", record_id: user_id, old_value: old_profile, new_value: profile }
    }
}
```

### 条件切面

```valkyrie
# 定义条件切面消息类型
class ConditionalAdvice {
    context: AspectContext
}

class DebugAspect {
    micro handle_debug<T>(self, operation: { -> T }) -> T {
        try {
            operation()
        }
        .catch {
            case ConditionalAdvice { context }:
                # 只在调试模式下应用
                if config.debug_mode && context.method_name.starts_with("debug_") {
                    print(f"Debug: {context.class_name}.{context.method_name}")
                }
                resume(())
        }
    }
}
```

### 切面组合器

```valkyrie
# 切面组合器
class AspectComposer {
    micro compose_aspects<T>(self, aspects: [AspectHandler], context: AspectContext, operation: { -> T }) -> T {
        # 按顺序执行所有切面的前置通知
        loop aspect in aspects {
            aspect.before(context)
        }
        
        try {
            let result = operation()
            
            # 按逆序执行所有切面的后置通知
            loop aspect in aspects.reverse() {
                aspect.after(context, result)
            }
            
            result
        }
        .catch {
            case error:
                # 执行异常通知
                loop aspect in aspects.reverse() {
                    aspect.on_error(context, error)
                }
                raise error
        }
    }
}
```

## 高级特性

### 动态切面

```valkyrie
# 动态切面管理器
class DynamicAspectManager {
    private mut aspects: [AspectHandler] = []
    
    micro add_aspect(self, aspect: AspectHandler) {
        self.aspects.push(aspect)
    }
    
    micro remove_aspect(self, aspect: AspectHandler) {
        self.aspects.retain { % != aspect }
    }
    
    @around(DynamicAspect)
    micro execute_with_aspects<T>(self, operation: { -> T }) -> T {
        let context = AspectContext::new()
        
        # 动态应用所有注册的切面
        loop aspect in self.aspects {
            aspect.before(context)
        }
        
        try {
            let result = operation()
            
            loop aspect in self.aspects.reverse() {
                aspect.after(context, result)
            }
            
            result
        }
        .catch {
            case error:
                loop aspect in self.aspects.reverse() {
                    aspect.on_error(context, error)
                }
                raise error
        }
    }
}
```

### 切面链

```valkyrie
# 切面链处理
class AspectChain {
    private aspects: [AspectHandler]
    private mut current_index: usize = 0
    
    micro proceed<T>(mut self, context: AspectContext) -> T {
        if self.current_index >= self.aspects.length {
            # 执行原始方法
            context.target_method()
        } else {
            let aspect = self.aspects[self.current_index]
            self.current_index += 1
            
            # 执行当前切面
            aspect.around(context, { self.proceed(context) })
        }
    }
}
```

## 最佳实践

### 1. 切面职责单一
每个切面应该只关注一个横切关注点，避免在单个切面中混合多种功能。

### 2. 合理使用切面顺序
当多个切面作用于同一个连接点时，要考虑它们的执行顺序。

### 3. 避免切面间的强耦合
切面之间应该保持独立，避免相互依赖。

### 4. 性能考虑
切面会增加方法调用的开销，在性能敏感的场景中要谨慎使用。

```valkyrie
# 示例：完整的 AOP 应用
class OrderService {
    @around(LogAspect, MetricsAspect, SecurityAspect, TransactionAspect)
    micro create_order(self, user_id: string, items: [OrderItem]) -> Order {
        # 所有切面的前置通知会自动执行
        # - 日志记录方法调用
        # - 开始性能计时
        # - 安全检查
        # - 开始事务
        
        let order = Order {
            id: generate_order_id(),
            user_id,
            items,
            status: OrderStatus.Pending,
            created_at: now()
        }
        
        self.save_order(order)
        
        # 所有切面的后置通知会自动执行
        # - 提交事务
        # - 记录安全日志
        # - 记录性能指标
        # - 记录方法返回值
        
        order
    }
}

# 使用示例
let order_service = OrderService {}
let log_handler = ConsoleLogHandler {}
let metrics_handler = MetricsHandler {}

try {
    let order = log_handler.handle_log {
        metrics_handler.handle_metrics {
            order_service.create_order("user123", [
                OrderItem { product_id: "p1", quantity: 2 },
                OrderItem { product_id: "p2", quantity: 1 }
            ])
        }
    }
    print(f"Created order: {order.id}")
}
.catch {
    case LogBefore { class_name, method_name, args }:
        print("→ {class_name}.{method_name}({args})")
        resume(())
    case LogAfter { class_name, method_name, result }:
        print("← {class_name}.{method_name} = {result}")
        resume(())
    case MetricsStart { operation }:
        # 处理性能监控开始
        resume(TimingContext::new(operation))
    case MetricsEnd { operation }:
        # 处理性能监控结束
        resume(Duration::default())
}
```

通过 Effect 系统实现的 AOP 提供了类型安全、组合性强、易于测试的切面编程能力，使得横切关注点的管理变得更加优雅和强大。
