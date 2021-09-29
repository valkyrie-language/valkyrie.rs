
## 面向对象编程

### 特殊类类型

- [神经网络类型 (Neural)](./neural.md) - 用于机器学习的特殊类类型
- [界面组件类型 (Widget)](./widget.md) - 用于 UI 开发的特殊类类型

### 字段定义

```valkyrie
# 基本字段定义
name: String
age: i32
is_active: bool = true  # 默认值

# 访问控制
public username: String
private password: String
protected internal_id: i64

# 只读字段
readonly created_at: DateTime
```

### 方法定义

```valkyrie
# 实例方法
greet(self) {
    print("Hello, I'm ${self.name}")
}

# 可变方法
set_age(mut self, new_age: i32) {
    self.age = new_age
}

# 静态方法
static create_anonymous() -> Person {
    Person { name: "Anonymous", age: 0 }
}

# 带返回值的方法
get_info(self) -> String {
    "${self.name} is ${self.age} years old"
}
```

### 类定义

```valkyrie
# 基本类定义
class Person {
    name: String
    age: i32
    
    new(name: String, age: i32) -> Self {
        Self { name, age }
    }
    
    greet(self) {
        print("Hello, I'm ${self.name}")
    }
}

# 继承类
class Student extends Person {
    student_id: String
    grade: i32
    
    new(name: String, age: i32, student_id: String) -> Self {
        Self {
            name,
            age,
            student_id,
            grade: 1
        }
    }
    
    study(self, subject: String) {
        print("${self.name} is studying ${subject}")
    }
}
```

### 类的继承和多态

```valkyrie
# 基础类
class Animal {
    name: String
    
    new(name: String) -> Self {
        Self { name }
    }
    
    virtual make_sound(self) {
        print("Some generic animal sound")
    }
}

# 派生类
class Dog extends Animal {
    breed: String
    
    new(name: String, breed: String) -> Self {
        Self {
            name,
            breed
        }
    }
    
    override make_sound(self) {
        print("Woof!")
    }
    
    fetch(self) {
        print("${self.name} is fetching")
    }
}
```

### 访问控制

```valkyrie
class BankAccount {
    private balance: f64
    protected account_number: String
    public owner: String
    
    new(owner: String, account_number: String) -> Self {
        Self {
            owner,
            account_number,
            balance: 0.0
        }
    }
    
    public deposit(mut self, amount: f64) {
        if amount > 0 {
            self.balance += amount
        }
    }
    
    public get_balance(self) -> f64 {
        self.balance
    }
    
    private validate_transaction(self, amount: f64) -> bool {
        amount > 0 && amount <= self.balance
    }
}
```



## 标志类型 (flags)

### 基本标志类型

```valkyrie
# 简单标志
flags FilePermissions {
    READ = 1,
    WRITE = 2,
    EXECUTE = 4
}

# 使用标志
let perms = FilePermissions::READ | FilePermissions::WRITE
if perms.contains(FilePermissions::READ) {
    print("可读")
}

# 复杂标志
flags WindowStyle {
    RESIZABLE = 0x01,
    MINIMIZABLE = 0x02,
    MAXIMIZABLE = 0x04,
    CLOSABLE = 0x08,
    TITLEBAR = 0x10,
    BORDER = 0x20,
    
    # 组合标志
    DEFAULT = RESIZABLE | MINIMIZABLE | MAXIMIZABLE | CLOSABLE | TITLEBAR | BORDER,
    DIALOG = CLOSABLE | TITLEBAR | BORDER
}
```

### 标志操作

```valkyrie
flags Permissions {
    READ = 1,
    write = 2,
    execute = 4,
    
    # 方法
    micro has_read(self) -> bool {
        self.contains(Permissions::read)
    }
    
    micro add_write(mut self) {
        self |= Permissions::write
    }
    
    micro remove_execute(mut self) {
        self &= !Permissions::execute
    }
}
```

## 特征定义 (trait)

### 基本特征

```valkyrie
# 简单特征
trait Display {
    micro to_string(self) -> String
}

# 带默认实现的特征
trait Debug {
    micro debug(self) -> String
    
    # 默认实现
    micro print_debug(self) {
        print(self.debug())
    }
}

# 泛型特征
trait Iterator<T> {
    micro next(mut self) -> Option<T>
    
    # 默认方法
    micro collect(mut self) -> [T] {
        let mut result = []
        while let Some(item) = self.next() {
            result.push(item)
        }
        result
    }
    
    micro map<U>(self, f: micro(T) -> U) -> MapIterator<T, U> {
        MapIterator::new(self, f)
    }
}
```

### 特征实现

```valkyrie
# 为类型实现特征
impl Display for Person {
    micro to_string(self) -> String {
        "${self.name} (${self.age} years old)"
    }
}

impl Debug for Person {
    micro debug(self) -> String {
        "Person { name: \"${self.name}\", age: ${self.age} }"
    }
}

# 条件实现
impl<T> Display for Option<T> where T: Display {
    micro to_string(self) -> String {
        match self {
            Some(value) => "Some(${value.to_string()})",
            None => "None"
        }
    }
}
```

### 特征约束

```valkyrie
# 函数中的特征约束
micro print_items<T>(items: [T]) where T: Display {
    for item in items {
        print(item.to_string())
    }
}

# 多重约束
micro process<T>(value: T) -> String 
where 
    T: Display + Debug + Clone 
{
    let cloned = value.clone()
    "Display: ${value.to_string()}, Debug: ${cloned.debug()}"
}

# 关联类型
trait Collect<T> {
    type Output
    
    micro collect(self) -> Self::Output
}
```

## 类型别名

```valkyrie
# 简单类型别名
type UserId = i64
type UserName = String
type Coordinates = (f64, f64)

# 泛型类型别名
type Result<T> = Result<T, String>
type HashMap<K, V> = std::collections::HashMap<K, V>

# 函数类型别名
type Handler = micro(Request) -> Response
type Predicate<T> = micro(T) -> bool
```

## 常量定义

```valkyrie
# 基本常量
const PI: f64 = 3.14159265359
const MAX_USERS: i32 = 1000
const APP_NAME: String = "MyApp"

# 复杂常量
const DEFAULT_CONFIG: Config = Config {
    timeout: 30,
    retries: 3,
    debug: false
}

# 计算常量
const BUFFER_SIZE: usize = 1024 * 1024  # 1MB
const HALF_PI: f64 = PI / 2.0
```

## 模块定义

```valkyrie
# 模块声明
mod utils {
    pub micro helper_function() {
        # 实现
    }
    
    pub class UtilityClass {
        # 实现
    }
}

# 使用模块
use utils::helper_function
use utils::UtilityClass

# 重导出
pub use utils::*
```

## 泛型定义

```valkyrie
# 泛型函数
micro swap<T>(a: &mut T, b: &mut T) {
    let temp = *a
    *a = *b
    *b = temp
}

# 泛型类
class Container<T> {
    value: T
    
    new(value: T) {
        Self { value }
    }
    
    get(self) -> &T {
        &self.value
    }
    
    set(mut self, new_value: T) {
        self.value = new_value
    }
}

# 约束泛型
class SortedList<T> where T: Ord {
    items: [T]
    
    insert(mut self, item: T) {
        # 保持排序插入
        let pos = self.items.binary_search(&item).unwrap_or_else({ $e })
        self.items.insert(pos, item)
    }
}
```

## 属性和装饰器

```valkyrie
# 属性装饰器
@.derive(Debug, Clone, PartialEq)
class Point {
    x: f64
    y: f64
}

@.test
micro test_addition() {
    @assert_equal(2 + 2, 4)
}

@.deprecated("Use new_function instead")
micro old_function() {
    # 已废弃的函数
}

@.inline
micro fast_calculation(x: i32) -> i32 {
    x * x + 2 * x + 1
}
```


