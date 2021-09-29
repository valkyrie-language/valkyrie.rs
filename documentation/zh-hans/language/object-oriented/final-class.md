# Final 类与方法 (Final Classes and Methods)

## 概述

`final` 关键字用于禁止继承或重写。被标记为 `final` 的类不能被继承，被标记为 `final` 的方法不能被子类重写。这是一种重要的封装手段，用于保护关键实现不被修改。

## Final 类

被标记为 `final` 的类不能被继承：

```valkyrie
# Final 类 - 禁止继承
final class ApiClient {
    base_url: utf8
    
    initiate(mut self, base_url: utf8) {
        self.base_url = base_url
    }
    
    micro request(self, endpoint: utf8) -> Response {
        # 实现细节...
    }
}

# 编译错误：不能继承 final 类
# class CustomApiClient(ApiClient) { ... }
```

### 使用场景

#### 1. 安全敏感的类

```valkyrie
# 防止子类修改安全逻辑
final class Authenticator {
    secret_key: utf8
    
    micro verify_token(self, token: utf8) -> bool {
        # 安全验证逻辑，不能被子类覆盖
        self.decode_and_verify(token)
    }
    
    private micro decode_and_verify(self, token: utf8) -> bool {
        # 内部实现...
    }
}
```

#### 2. 不可变值类型

```valkyrie
# 保证不可变性不被破坏
final class Money {
    readonly amount: Decimal
    readonly currency: utf8
    
    initiate(mut self, amount: Decimal, currency: utf8) {
        self.amount = amount
        self.currency = currency
    }
    
    micro add(self, other: Money) -> Money {
        if self.currency != other.currency {
            panic("货币不匹配")
        }
        Money { amount: self.amount + other.amount, currency: self.currency }
    }
}
```

#### 3. 性能关键类

```valkyrie
# 允许编译器进行内联优化
final class FastMath {
    micro static square(x: f64) -> f64 {
        x * x  # 可以被内联
    }
    
    micro static cube(x: f64) -> f64 {
        x * x * x  # 可以被内联
    }
}
```

## Final 方法

被标记为 `final` 的方法不能被子类重写：

```valkyrie
class BaseController {
    # Final 方法 - 子类不能重写
    final micro handle_request(self, request: Request) -> Response {
        let validated = self.validate(request)
        let result = self.process(validated)
        self.format_response(result)
    }
    
    # 可被子类重写的方法
    micro validate(self, request: Request) -> ValidatedRequest {
        # 默认验证逻辑
    }
    
    micro process(self, request: ValidatedRequest) -> Result {
        # 默认处理逻辑
    }
    
    micro format_response(self, result: Result) -> Response {
        # 默认响应格式
    }
}

class CustomController(BaseController) {
    # 可以重写非 final 方法
    override micro process(self, request: ValidatedRequest) -> Result {
        # 自定义处理逻辑
    }
    
    # 编译错误：不能重写 final 方法
    # override micro handle_request(self, request: Request) -> Response { ... }
}
```

### 使用场景

#### 1. 模板方法模式

```valkyrie
abstract class DataProcessor {
    # 模板方法 - 固定算法骨架
    final micro process(self, data: utf8) -> utf8 {
        let step1 = self.preprocess(data)
        let step2 = self.transform(step1)
        let step3 = self.postprocess(step2)
        self.log(step3)
        step3
    }
    
    # 子类可重写的步骤
    abstract micro preprocess(self, data: utf8) -> utf8
    abstract micro transform(self, data: utf8) -> utf8
    abstract micro postprocess(self, data: utf8) -> utf8
    
    # 可选重写
    micro log(self, result: utf8) {
        print("处理完成: {result}")
    }
}
```

#### 2. 关键业务逻辑

```valkyrie
class BankAccount {
    balance: f64
    
    # Final 方法 - 防止子类修改转账逻辑
    final micro transfer(mut self, to: &mut BankAccount, amount: f64) {
        if amount <= 0 {
            panic("转账金额必须为正数")
        }
        if self.balance < amount {
            panic("余额不足")
        }
        self.balance -= amount
        to.balance += amount
        self.record_transaction(to, amount)
    }
    
    micro record_transaction(self, to: &BankAccount, amount: f64) {
        # 记录交易日志
    }
}
```

#### 3. 回调钩子保护

```valkyrie
class EventEmitter {
    listeners: HashMap<utf8, [Callback]>
    
    # Final 方法 - 保证事件分发逻辑一致
    final micro emit(self, event: utf8, data: Any) {
        if let Some(callbacks) = self.listeners.get(event) {
            loop callback in callbacks {
                callback(data)
            }
        }
    }
    
    # 可被子类扩展的方法
    micro on(mut self, event: utf8, callback: Callback) {
        self.listeners.get_or_insert(event, []).push(callback)
    }
}
```

## Final 属性

属性也可以标记为 `final`：

```valkyrie
class Entity {
    id: u64
    
    # Final 属性 - 子类不能重写
    final get entity_id(self) -> u64 {
        self.id
    }
    
    # 可被子类重写的属性
    get display_name(self) -> utf8 {
        "Entity-{self.id}"
    }
}

class User(Entity) {
    name: utf8
    
    # 可以重写非 final 属性
    override get display_name(self) -> utf8 {
        self.name
    }
    
    # 编译错误：不能重写 final 属性
    # override get entity_id(self) -> u64 { ... }
}
```

## Final vs Sealed vs Abstract

| 特性 | final | sealed | abstract |
|:---|:---|:---|:---|
| **可实例化** | ✅ 是 | ✅ 是（非抽象子类） | ❌ 否 |
| **可继承** | ❌ 禁止 | ⚠️ 限同一文件/模块 | ✅ 必须 |
| **方法可重写** | ❌ 禁止 | ✅ 允许 | ✅ 允许 |
| **用途** | 保护实现 | 限制类型层级 | 定义抽象契约 |

```valkyrie
# 层次结构示例
abstract class Base {
    abstract micro must_implement(self)
    micro can_override(self) { }
    final micro cannot_override(self) { }
}

# 同一文件内
sealed class Middle(Base) {
    override micro must_implement(self) { }
}

# 完全封闭
final class Concrete(Middle) {
    # 不能再被继承
}
```

## 性能优化

`final` 关键字可以帮助编译器进行优化：

```valkyrie
# 编译器可以将 final 方法内联
final class Math {
    final micro static add(a: i32, b: i32) -> i32 {
        a + b
    }
}

# 调用点可以直接内联为 a + b
let result = Math::add(1, 2)
```

## 最佳实践

### 1. 默认考虑 final

```valkyrie
# 好的实践：默认使用 final，有需要时再开放
final class EmailSender {
    micro send(self, to: utf8, subject: utf8, body: utf8) {
        # 实现
    }
}

# 如果确实需要继承，移除 final
class EmailSender {
    micro send(self, to: utf8, subject: utf8, body: utf8) {
        # 实现
    }
}
```

### 2. 保护关键方法

```valkyrie
class Cache {
    data: HashMap<utf8, Any>
    
    # 保护缓存一致性
    final micro get(self, key: utf8) -> Any? {
        self.data.get(key)
    }
    
    final micro set(mut self, key: utf8, value: Any) {
        self.data.insert(key, value)
    }
    
    # 子类可自定义驱逐策略
    micro evict_expired(mut self) {
        # 可重写
    }
}
```

### 3. 文档化 final 决策

```valkyrie
↯doc("""
UserSession 被标记为 final，因为：
1. 包含敏感的认证状态
2. 生命周期管理逻辑不应被修改
3. 性能优化考虑
""")
final class UserSession {
    # ...
}
```

## 总结

`final` 关键字的核心作用：

1. **保护实现**：防止关键逻辑被意外修改
2. **性能优化**：允许编译器进行内联等优化
3. **设计意图**：明确表达"不应被继承/重写"的设计决策
4. **安全性**：保护安全敏感的类和方法

合理使用 `final` 可以提高代码的健壮性和可维护性。
