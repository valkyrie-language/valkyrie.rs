# 单例对象 (Singleton Objects)

## 概述

单例对象是一种特殊的类型定义，它结合了"类定义"与"实例声明"。当你定义一个单例时，系统会自动创建一个该类型的全局唯一实例。

## 定义单例

使用 `singleton` 关键字代替 `class`：

```valkyrie
singleton AppConfig {
    public mut debug_mode: bool = false
    public mut api_key: utf8 = ""

    micro load_from_env(mut self) {
        # 从环境变量加载配置...
    }
}
```

## 访问单例

与普通类不同，访问单例的成员使用 `.` 运算符，且无需实例化：

```valkyrie
# 在任何地方直接使用
if AppConfig.debug_mode {
    print("Debug info...")
}

# 修改单例状态
AppConfig.api_key = "secret_value"

# 调用单例方法
AppConfig.load_from_env()
```

## 单例 vs 静态方法

这是开发者最容易混淆的地方：

| 特性 | 类 (Class) | 单例 (Singleton) |
|:---|:---|:---|
| **定义关键字** | `class` | `singleton` |
| **访问成员** | `Type::member` (静态) | `Instance.member` |
| **实例化** | 需要 `let x = Type()` | 自动实例化，全局唯一 |
| **状态存储** | 通常存储在实例中 | 存储在全局唯一的实例中 |
| **可以实现 Trait** | 是 | 是 |

```valkyrie
# 类的静态方法
class User {
    micro static find(id: i32) -> User? {
        # 查询数据库...
    }
}
let user = User::find(1)  # 使用 :: 调用静态方法

# 单例的方法
singleton Database {
    micro query(self, sql: utf8) -> Result {
        # 执行查询...
    }
}
let result = Database.query("SELECT * FROM users")  # 使用 . 调用实例方法
```

## 应用场景

### 全局状态管理

用于存储应用运行时的全局配置、缓存或计数器：

```valkyrie
singleton Counter {
    public mut total_requests: i64 = 0
    
    micro increment(mut self) -> i64 {
        self.total_requests += 1
        self.total_requests
    }
}

# 在任何地方调用
let count = Counter.increment()
```

### 资源包装器

对于数据库连接池、日志记录器等只需要一个实例的资源：

```valkyrie
singleton Logger {
    level: LogLevel = LogLevel::INFO
    
    micro info(self, message: utf8) {
        self.log(LogLevel::INFO, message)
    }
    
    micro error(self, message: utf8) {
        self.log(LogLevel::ERROR, message)
    }
    
    private micro log(self, level: LogLevel, message: utf8) {
        # 写入日志...
    }
}

# 调用
Logger.info("Application started")
Logger.error("Connection failed")
```

### 服务定位器模式

```valkyrie
singleton ServiceLocator {
    services: HashMap<utf8, Any> = {}
    
    micro register(mut self, name: utf8, service: Any) {
        self.services.insert(name, service)
    }
    
    micro get(self, name: utf8) -> Any? {
        self.services.get(name)
    }
}

# 注册服务
ServiceLocator.register("cache", RedisCache::new())
ServiceLocator.register("queue", RabbitMQ::new())

# 获取服务
let cache = ServiceLocator.get("cache") as RedisCache
```

## 单例与 Trait

单例可以实现 Trait，就像普通类一样：

```valkyrie
trait Serializable {
    micro serialize(self) -> utf8
    micro deserialize(mut self, data: utf8)
}

singleton Settings: Serializable {
    theme: utf8 = "dark"
    language: utf8 = "zh-CN"
    
    micro serialize(self) -> utf8 {
        f"{{\"theme\": \"{self.theme}\", \"language\": \"{self.language}\"}}"
    }
    
    micro deserialize(mut self, data: utf8) {
        # 解析 JSON 并更新字段...
    }
}
```

## 线程安全

Valkyrie 的单例默认是线程安全的：

```valkyrie
singleton ThreadSafeCounter {
    public mut count: AtomicI64 = AtomicI64::new(0)
    
    micro increment(mut self) -> i64 {
        self.count.fetch_add(1, Ordering::SeqCst)
    }
    
    micro get(self) -> i64 {
        self.count.load(Ordering::SeqCst)
    }
}
```

## 最佳实践

### 1. 谨慎使用可变状态

```valkyrie
# 避免：过多的可变状态
singleton BadExample {
    mut state1: Type1
    mut state2: Type2
    mut state3: Type3
    # ...难以追踪的状态变化
}

# 推荐：最小化可变状态
singleton GoodExample {
    # 只读配置
    config: Config = Config::load()
    
    # 必要的可变状态使用原子操作
    mut request_count: AtomicI64
}
```

### 2. 优先使用依赖注入

```valkyrie
# 在复杂应用中，优先使用依赖注入而非单例
class UserService {
    @inject
    database: Database
    
    micro get_user(self, id: i32) -> User? {
        self.database.query(...)
    }
}
```

### 3. 单例适合无状态或配置型服务

```valkyrie
# 好的单例使用场景
singleton ApiEndpoints {
    base_url: utf8 = "https://api.example.com"
    version: utf8 = "v1"
    
    micro get_url(self, endpoint: utf8) -> utf8 {
        "{self.base_url}/{self.version}/{endpoint}"
    }
}
```

## 注意事项

1. **测试困难**：单例的全局状态可能导致测试之间的相互影响
2. **隐藏依赖**：使用单例的代码可能隐藏了对外部状态的依赖
3. **生命周期**：单例的生命周期与应用相同，无法提前释放资源

在复杂的业务逻辑中，建议优先考虑依赖注入模式。
