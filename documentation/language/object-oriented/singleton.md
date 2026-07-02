# 单例对象 (Singleton Objects)

## 概述

单例对象是一种特殊的类型定义，它结合了"类定义"与"实例声明"。当你定义一个单例时，系统会自动创建一个该类型的全局唯一实例。

编译器层面的固定契约如下：

- 唯一实例字段名固定为 `INSTANCE`
- eager `singleton` 的访问器名固定为 `instance()`
- lazy `singleton` 的访问器名固定为 `get_instance()`
- 用户表面语法始终写 `Counter.total`、`Counter.increment()`，后端再把它 lowering 成"访问器取唯一实例 + 实例字段/实例方法"

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

这不是“静态方法假象”。`AppConfig.load_from_env()` 的真实语义仍然是对唯一实例的方法调用，而不是对类型本身做静态调用。

## 初始化模式

### eager singleton

普通 `singleton` 默认是 eager 模式，会在模块加载或 fragment 初始化阶段准备好唯一实例：

```valkyrie
singleton Counter {
    public mut total: i64 = 0
}
```

### lazy singleton

`lazy singleton` 会把初始化延后到第一次访问：

```valkyrie
lazy singleton Cache {
    public mut hits: i64 = 0
}
```

无论 eager 还是 lazy，语言表面的字段与方法访问方式都保持一致，差异只在初始化时机和后端访问器命名。

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

## accessor 命名规则

用户表面语法始终写 `Counter.total`、`Counter.increment()`，编译器会自动把它 lowering 成"调用 accessor 取唯一实例 + 实例字段/实例方法调用"。用户不需要、也不应该直接调用 accessor。

accessor 的命名由初始化模式固定决定：

| 初始化模式 | accessor 名 | 触发时机 |
|:---|:---|:---|
| eager `singleton` | `instance()` | 模块加载 / fragment 初始化阶段已建好实例，`instance()` 直接返回 |
| `lazy singleton` | `get_instance()` | 第一次访问时 null-check，为空则分配并写回全局槽，再返回 |

这意味着如果你在反射或 FFI 互操作场景里需要按符号名查找 singleton accessor，必须同时考虑 `instance` 与 `get_instance` 两个候选，具体取决于该 singleton 是否带 `lazy` 修饰。

## singleton 方法体内部的 self 语义

在 singleton 方法体内部，`self` 直接绑定到唯一实例本身，**不会**再次发射 accessor 调用。换句话说：

```valkyrie
singleton Counter {
    mut total: i64 = 0

    micro increment(mut self) -> i64 {
        self.total += 1
        self.total
    }
}
```

`Counter.increment()` 在外部调用时会被 lowering 成"取实例 + 调方法"两步；但 `increment` 方法体内部的 `self.total` 直接访问传入的实例参数，不会再触发一次 `Counter.instance()`。这与"静态方法假象"有本质区别——singleton 方法始终是实例方法，`self` 就是那个全局唯一实例。

`self` 的可变性遵循普通实例方法规则：`micro f(self)` 内部只能读字段，`micro f(mut self)` 内部可以写字段。singleton 字段还需要额外带 `mut` 修饰才允许写入，否则即使方法声明了 `mut self` 也会被 singleton 读写检查拒绝。

## 后端实现差异

各后端用不同方式物化"全局唯一实例 + accessor"契约，但都遵守相同的命名与初始化边界。下表汇总当前各后端的实现策略：

| 后端 | INSTANCE 存储 | eager 初始化 | lazy 初始化 | 实例方法归位 |
|:---|:---|:---|:---|:---|
| `CLR` | 类型上的 `static` 字段 `INSTANCE` | `.cctor` 调 `.ctor` 写回，`instance()` 做 `ldsfld` | lock 字段 + double-check 的 `get_instance()` | 作为该类型的实例方法发射，`ldarg.0` 即 `this` |
| `JVM` | 类上的 `static` 字段 `INSTANCE` | `<clinit>` 分配并 `putstatic`，`instance()` 做 `getstatic` | `get_instance()` 做 null-check + allocate + `putstatic` | 作为该类的实例方法发射 |
| `NyarVM` | 模块全局 `{name}.INSTANCE` | 模块 init 函数分配记录并写回全局 | `get_instance()` 导出做 null-check + alloc + store | 通过 `{name}__{method}` 导出名调用，首参为实例 |
| `WASM` | 可变 `i32` 全局（null 指针占位） | accessor 直接 `global.get` | `global.get` + `i32.eqz` + stub alloc + `global.set` | 由统一元数据契约描述，待对象 runtime 落地 |
| `Native` (PE/ELF) | `.rdata` / `.rodata` 中 8 字节槽 `singleton_slot_{name}` | 由元数据契约描述 | 由元数据契约描述 | 待可写数据段（`.bss`）与 runtime 落地后补齐 |

注意 `WASM` 与 `Native` 当前仍以"统一元数据契约 + 占位存储"为主，真正可变实例存储和 accessor 函数体细节会随对应 runtime 一起落地。元数据行的格式固定为 `{namespace}|{name}|{instance_field}|{mode}|{accessor}`，其中 `mode` 为 `eager` 或 `lazy`。

## 泛型 singleton 的当前限制

`HirSingleton` 在 HIR 层保留了 `generics` 字段，因此 `singleton Foo<T> { ... }` 这样的语法可以解析。但当前语义管线**未落地**泛型 singleton：

- `SingletonInstancePlan` 只携带简单名，不携带泛型参数信息。
- 各后端用简单名作为 `INSTANCE` 字段类型，无法表达 `Foo<i32>` 这样的具体实例化。
- singleton 的"全局唯一实例"语义与泛型参数化存在语义冲突：泛型 singleton 通常期望"每个实例化对应一个唯一实例"（类似 C# 泛型 static 类），但当前 `INSTANCE` 是单一全局槽，无法按类型参数键化。

因此在后端 plan 携带泛型信息、且 `INSTANCE` 存储改为按实例化键化之前，**不应使用泛型 singleton**。未来放开时需要同步更新 `collect_singleton_instance_plans` 与所有后端的 `augment_*_with_singletons` 路径。

## 线程安全边界

单例语义保证"全局唯一实例"与固定初始化入口，但不等于所有后端今天都提供相同级别的线程安全实现。

- `CLR` 的 lazy singleton 当前按带锁延迟初始化建模（lock 字段 + double-check）
- `JVM` 的 lazy singleton 在 `<clinit>` 之外通过 `get_instance()` 做 null-check + allocate + putstatic，依赖 class init 的线程安全保证实例创建，但 `get_instance` 自身未加显式锁
- `NyarVM` 当前契约只保证首次访问时做 null-check 并写回全局槽，不提供跨线程 lazy 初始化保证，仅适用于单线程语义
- `WASM`、`Native` 仍以统一元数据契约为主，线程安全细节依赖后续 runtime 落地

因此，如果你的单例包含可变共享状态，仍应显式使用原子类型、锁或等价同步原语：

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
