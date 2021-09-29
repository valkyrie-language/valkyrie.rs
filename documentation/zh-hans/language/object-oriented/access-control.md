# 访问控制 (Access Control)

## 概述

Valkyrie 提供了完整的访问控制机制，用于控制类成员、模块成员的可见性。访问控制是封装的核心实现手段，确保类型的内部实现细节不被外部代码意外访问或修改。

## 访问级别

Valkyrie 提供四个访问级别，从宽松到严格：

| 修饰符 | 说明 | 可见范围 |
|:---|:---|:---|
| `public` | 公开 | 所有代码可见 |
| `protected` | 受保护 | 当前类及其子类可见 |
| `internal` | 内部 | 当前模块可见 |
| `private` | 私有 | 仅当前类/文件可见 |

### 默认可见性

```valkyrie
# 字段默认为 private
class Example {
    name: utf8          # 等同于 private name: utf8
    public age: i32     # 显式公开
    protected id: u64   # 受保护
}

# 方法默认为 public
class Service {
    micro process(self) { }     # 等同于 public micro process(self) { }
    private micro helper(self)  # 显式私有
}
```

## 字段访问控制

```valkyrie
class User {
    # 公开字段 - 任何代码可访问
    public username: utf8
    
    # 私有字段 - 仅本类可访问
    private password_hash: utf8
    
    # 受保护字段 - 本类及子类可访问
    protected created_at: DateTime
    
    # 内部字段 - 同模块可访问
    internal cache: HashMap<utf8, Any>
    
    # 只读字段 - 公开但不可修改
    readonly id: u64
    
    initiate(mut self, username: utf8, password: utf8) {
        self.username = username
        self.password_hash = hash(password)
        self.created_at = DateTime::now()
        self.id = generate_id()
    }
}
```

## 方法访问控制

```valkyrie
class Database {
    private connection: Connection
    
    # 公开方法 - 外部调用入口
    public micro query(self, sql: utf8) -> Result<[Row], Error> {
        self.validate_query(sql)
        self.execute(sql)
    }
    
    # 受保护方法 - 子类可扩展
    protected micro validate_query(self, sql: utf8) {
        if sql.contains("DROP") {
            panic("不允许执行 DROP 操作")
        }
    }
    
    # 私有方法 - 内部实现细节
    private micro execute(self, sql: utf8) -> Result<[Row], Error> {
        self.connection.execute(sql)
    }
    
    # 内部方法 - 同模块的工具方法
    internal micro reset_connection(mut self) {
        self.connection = Connection::new()
    }
}
```

## 属性访问控制

```valkyrie
class BankAccount {
    private balance: f64
    
    # 公开 getter，私有 setter
    public get balance(self) -> f64 {
        self.balance
    }
    private set balance(mut self, value: f64) {
        self.balance = value
    }
    
    # 受保护属性
    protected get account_type(self) -> utf8 {
        "standard"
    }
    
    # 内部属性
    internal get internal_id(self) -> u64 {
        self.id
    }
    
    # 公开方法修改私有字段
    public micro deposit(mut self, amount: f64) {
        if amount <= 0 {
            panic("存款金额必须为正数")
        }
        self.balance = self.balance + amount
    }
}
```

## 继承中的访问控制

```valkyrie
class Base {
    public public_field: i32
    protected protected_field: i32
    private private_field: i32
    internal internal_field: i32
    
    public micro public_method(self) { }
    protected micro protected_method(self) { }
    private micro private_method(self) { }
    internal micro internal_method(self) { }
}

class Derived(Base) {
    micro test_access(self) {
        # ✅ 可以访问
        self.public_field = 1
        self.protected_field = 2
        self.internal_field = 3  # 如果同模块
        self.public_method()
        self.protected_method()
        
        # ❌ 无法访问
        # self.private_field = 4
        # self.private_method()
    }
    
    # 可以重写 protected 方法
    override protected micro protected_method(self) {
        # 新实现
    }
    
    # 不能重写 private 方法（子类不知道它的存在）
}
```

## 模块级访问控制

```valkyrie
# module.vk
module myapp {
    # 公开模块 - 其他模块可导入
    public mod api {
        public class Handler { }
    }
    
    # 内部模块 - 仅当前模块可见
    internal mod internal_utils {
        class Helper { }
    }
    
    # 私有模块 - 仅当前文件可见
    private mod private_impl {
        class Secret { }
    }
}

# 其他文件
using myapp::api::Handler      # ✅ 可以导入
# using myapp::internal_utils  # ❌ 无法导入（不同模块）
```

## 构造函数访问控制

```valkyrie
class Singleton {
    private static instance: Singleton?
    
    # 私有构造函数 - 防止外部实例化
    private initiate(mut self) { }
    
    # 公开工厂方法
    public micro static get_instance() -> Singleton {
        if Self::instance.is_none() {
            Self::instance = Some(Singleton {})
        }
        Self::instance.unwrap()
    }
}

# let s = Singleton {}        # 编译错误：私有构造函数
let s = Singleton::get_instance()  # ✅ 通过工厂方法获取
```

## 访问控制与 Trait 实现

```valkyrie
trait Serializable {
    micro serialize(self) -> utf8
}

class User {
    private data: HashMap<utf8, Any>
    
    # Trait 实现方法必须是 public
    public micro serialize(self) -> utf8 {
        # 实现细节
    }
    
    # 内部序列化方法
    internal micro serialize_internal(self) -> utf8 {
        # 包含更多细节的序列化
    }
}
```

## 访问控制最佳实践

### 1. 最小权限原则

```valkyrie
# 好的实践：默认私有，按需公开
class GoodExample {
    private data: [u8]           # 内部数据
    private cache: HashMap       # 内部缓存
    
    public get count(self) -> usize {  # 只读访问
        self.data.len()
    }
    
    public micro add(mut self, item: u8) {  # 受控修改
        self.validate(item)
        self.data.push(item)
    }
    
    private micro validate(self, item: u8) {  # 内部验证
        # ...
    }
}

# 避免：过度公开
class BadExample {
    public data: [u8]  # 外部可直接修改，破坏封装
}
```

### 2. 保护内部状态

```valkyrie
class Counter {
    private mut count: u32 = 0
    
    # 通过方法控制访问
    public micro increment(mut self) -> u32 {
        self.count += 1
        self.count
    }
    
    public micro get(self) -> u32 {
        self.count
    }
    
    # 受保护的重置方法，子类可使用
    protected micro reset(mut self) {
        self.count = 0
    }
}
```

### 3. 使用 internal 实现模块内协作

```valkyrie
# 在同一模块内的多个类可以共享 internal 成员
module database {
    class Connection {
        internal pool: ConnectionPool
        
        internal micro return_to_pool(mut self) {
            self.pool.return_connection(self)
        }
    }
    
    class ConnectionPool {
        internal micro return_connection(mut self, conn: Connection) {
            # 模块内部协作
        }
    }
}
```

### 4. 只读字段 vs 私有字段 + getter

```valkyrie
# 方式 1：只读字段（简洁）
class Point {
    readonly x: f64
    readonly y: f64
}

# 方式 2：私有字段 + getter（灵活）
class Point {
    private x: f64
    private y: f64
    
    get x(self) -> f64 { self.x }
    get y(self) -> f64 { self.y }
    
    # 可以添加验证逻辑
    micro set_x(mut self, value: f64) {
        if value < 0 { panic("坐标不能为负") }
        self.x = value
    }
}
```

## 访问控制总结

| 场景 | 推荐访问级别 |
|:---|:---|
| 公开 API | `public` |
| 子类可扩展的成员 | `protected` |
| 模块内协作 | `internal` |
| 实现细节 | `private` |
| 不可变公开数据 | `readonly` + `public` |

正确使用访问控制可以：
1. **封装实现细节**：隐藏复杂的内部逻辑
2. **保护数据完整性**：防止外部代码破坏内部状态
3. **降低耦合**：限制代码间的依赖关系
4. **提高可维护性**：修改私有实现不影响外部代码
