# 编程模式 (Patterns)

本章节介绍 Valkyrie 中常用的编程模式和最佳实践，帮助开发者编写更优雅、更可维护的代码。

## 设计模式

### 构建器模式 (Builder Pattern)

用于逐步构建复杂对象，特别适合具有多个可选参数的类型：

```valkyrie
class HttpRequest {
    method: string
    url: string
    headers: Map<string, string>
    body: Option<string>
    timeout: Duration
}

class HttpRequestBuilder {
    private mut method: string = "GET"
    private mut url: string = ""
    private mut headers: Map<string, string> = Map.new()
    private mut body: Option<string> = None
    private mut timeout: Duration = Duration.seconds(30)
    
    micro method(mut self, m: string) -> Self {
        self.method = m
        self
    }
    
    micro url(mut self, u: string) -> Self {
        self.url = u
        self
    }
    
    micro header(mut self, key: string, value: string) -> Self {
        self.headers.insert(key, value)
        self
    }
    
    micro body(mut self, b: string) -> Self {
        self.body = Some(b)
        self
    }
    
    micro timeout(mut self, t: Duration) -> Self {
        self.timeout = t
        self
    }
    
    micro build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout
        }
    }
}

# 使用示例
let request = HttpRequestBuilder.new()
    .method("POST")
    .url("https://api.example.com/users")
    .header("Content-Type", "application/json")
    .body(json_data)
    .timeout(Duration.seconds(60))
    .build()
```

### 工厂模式 (Factory Pattern)

用于封装对象的创建逻辑：

```valkyrie
trait Animal {
    micro speak(self) -> string
}

class Dog: Animal {
    name: string
    
    micro speak(self) -> string {
        "{self.name} says: Woof!"
    }
}

class Cat: Animal {
    name: string
    
    micro speak(self) -> string {
        "{self.name} says: Meow!"
    }
}

class AnimalFactory {
    micro create(type: string, name: string) -> Animal {
        match type {
            case "dog": Dog { name }
            case "cat": Cat { name }
            else: raise "Unknown animal type: {type}"
        }
    }
}

# 使用示例
let factory = AnimalFactory.new()
let dog = factory.create("dog", "Buddy")
let cat = factory.create("cat", "Whiskers")
```

### 单例模式 (Singleton Pattern)

Valkyrie 提供内置的 `singleton` 关键字：

```valkyrie
singleton Database {
    private connection: Connection
    
    initiate(mut self) {
        self.connection = Connection.connect("localhost:5432")
    }
    
    micro query(self, sql: string) -> Result<[Row], Error> {
        self.connection.execute(sql)
    }
}

# 使用示例
let result = Database.query("SELECT * FROM users")
```

### 观察者模式 (Observer Pattern)

用于实现事件通知机制：

```valkyrie
trait Observer<T> {
    micro on_event(self, event: T)
}

class Subject<T> {
    private observers: [Observer<T>] = []
    
    micro attach(mut self, observer: Observer<T>) {
        self.observers.push(observer)
    }
    
    micro detach(mut self, observer: Observer<T>) {
        self.observers = self.observers.filter { % != observer }
    }
    
    micro notify(self, event: T) {
        loop observer in self.observers {
            observer.on_event(event)
        }
    }
}

# 使用示例
class EmailNotifier: Observer<UserEvent> {
    micro on_event(self, event: UserEvent) {
        match event {
            case UserEvent.Created { user }:
                send_welcome_email(user.email)
            case UserEvent.Deleted { user_id }:
                log_info("User {user_id} deleted")
        }
    }
}
```

## 函数式模式

### 链式调用 (Method Chaining)

```valkyrie
let result = [1, 2, 3, 4, 5]
    .map { % * 2 }
    .filter { % > 4 }
    .reduce { %acc + % }
# 结果: 18
```

### 柯里化 (Currying)

```valkyrie
micro add(a: i32) -> micro(i32) -> i32 {
    micro(b) { a + b }
}

let add_five = add(5)
let result = add_five(3)  # 结果: 8
```

### 组合 (Composition)

```valkyrie
micro compose<A, B, C>(f: micro(B) -> C, g: micro(A) -> B) -> micro(A) -> C {
    micro(x) { f(g(x)) }
}

let double = { % * 2 }
let increment = { % + 1 }
let double_then_increment = compose(increment, double)

let result = double_then_increment(5)  # 结果: 11
```

## 错误处理模式

### Result 类型模式

```valkyrie
micro divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 {
        Fail { error: "Division by zero" }
    } else {
        Fine { value: a / b }
    }
}

# 链式错误处理
let result = divide(10.0, 2.0)
    .map { % * 3 }
    .and_then { divide(%, 5.0) }
```

### 选项模式 (Option Pattern)

```valkyrie
micro find_user(id: i64) -> Option<User> {
    let users = get_all_users()
    users.find { %.id == id }
}

# 安全访问
let user_name = find_user(42)
    .map { %.name }
    .unwrap_or("Unknown")
```

## 最佳实践

### 1. 优先使用不可变数据

```valkyrie
# 推荐：不可变
let config = Config {
    host: "localhost",
    port: 8080
}

# 不推荐：可变（除非必要）
let mut counter = 0
```

### 2. 使用模式匹配替代条件判断

```valkyrie
# 推荐
match status {
    case 200..=299: "Success"
    case 400..=499: "Client Error"
    case 500..=599: "Server Error"
    case _: "Unknown"
}

# 不推荐
if status >= 200 && status < 300 {
    "Success"
} else if status >= 400 && status < 500 {
    "Client Error"
} ...
```

### 3. 使用类型系统表达业务规则

```valkyrie
# 使用类型确保正确性
unite EmailAddress {
    Valid { value: string },
    Invalid { reason: string }
}

micro parse_email(input: string) -> EmailAddress {
    if input.contains("@") {
        EmailAddress.Valid { value: input }
    } else {
        EmailAddress.Invalid { reason: "Missing @" }
    }
}
```

---

更多设计模式和最佳实践，请参阅各领域的示例文档。
