# 繼承

繼承是物件導向程式設計的核心概念之一，允許子類重用和擴展父類的程式碼。

## 類別定義

### 基本類別

```valkyrie
# 定義一個基本類別
class Person {
    name: string
    age: i32
    
    # 建構函式
    new(name: string, age: i32) {
        self.name = name
        self.age = age
    }
    
    # 方法
    micro introduce(self) -> string {
        "Hello, I'm {self.name}, {self.age} years old."
    }
}
```

### 繼承語法

```valkyrie
# 子類繼承父類
class Student : Person {
    school: string
    grade: i32
    
    # 子類建構函式
    new(name: string, age: i32, school: string, grade: i32) {
        super(name, age)  # 呼叫父類建構函式
        self.school = school
        self.grade = grade
    }
    
    # 重寫父類方法
    override micro introduce(self) -> string {
        "{super.introduce()} I study at {self.school}, grade {self.grade}."
    }
}
```

## 方法重寫

### override 關鍵字

使用 `override` 關鍵字明確表示要重寫父類方法：

```valkyrie
class Animal {
    micro speak(self) -> string {
        "Some sound"
    }
}

class Dog : Animal {
    override micro speak(self) -> string {
        "Woof!"
    }
}

class Cat : Animal {
    override micro speak(self) -> string {
        "Meow!"
    }
}
```

### 呼叫父類方法

使用 `super` 關鍵字呼叫父類方法：

```valkyrie
class Vehicle {
    speed: i32
    
    micro start(self) {
        print("Vehicle starting...")
    }
}

class Car : Vehicle {
    fuel: i32
    
    override micro start(self) {
        super.start()  # 呼叫父類方法
        print("Car engine running...")
    }
}
```

## 多型

### 向上轉型

子類物件可以賦值給父類型別的變數：

```valkyrie
let animal: Animal = Dog {}
animal.speak()  # 輸出 "Woof!"（多型）
```

### 型別檢查與轉換

```valkyrie
micro describe(animal: Animal) {
    match animal {
        case dog: Dog: print("This is a dog: {dog.breed}")
        case cat: Cat: print("This is a cat: {cat.color}")
        else: print("Unknown animal")
    }
}
```

## 建構函式鏈

### 建構函式呼叫順序

```valkyrie
class A {
    a_value: i32
    
    new() {
        self.a_value = 1
        print("A constructed")
    }
}

class B : A {
    b_value: i32
    
    new() {
        super()  # 必須先呼叫父類建構函式
        self.b_value = 2
        print("B constructed")
    }
}

class C : B {
    c_value: i32
    
    new() {
        super()
        self.c_value = 3
        print("C constructed")
    }
}

let c = C {}  # 輸出順序: A -> B -> C
```

## 存取控制

### 可見性修飾符

```valkyrie
class Base {
    public public_field: i32      # 公開，任何地方可存取
    protected protected_field: i32 # 保護，子類可存取
    private private_field: i32    # 私有，僅本類可存取
    
    public micro public_method(self) {}
    protected micro protected_method(self) {}
    private micro private_method(self) {}
}

class Derived : Base {
    micro access_base(self) {
        self.public_field = 1         # OK
        self.protected_field = 2      # OK
        # self.private_field = 3      # 錯誤：無法存取私有成員
    }
}
```

## 抽象類別

### 定義抽象類別

```valkyrie
abstract class Shape {
    # 抽象方法，子類必須實現
    abstract micro area(self) -> f64
    abstract micro perimeter(self) -> f64
    
    # 具體方法
    micro describe(self) -> string {
        "Area: {self.area()}, Perimeter: {self.perimeter()}"
    }
}

class Rectangle : Shape {
    width: f64
    height: f64
    
    override micro area(self) -> f64 {
        self.width * self.height
    }
    
    override micro perimeter(self) -> f64 {
        2 * (self.width + self.height)
    }
}
```

## 最佳實踐

### 1. 優先使用組合

```valkyrie
# 組合優於繼承
class Engine {
    micro start(self) {}
}

class Car {
    engine: Engine  # 組合
    
    micro start(self) {
        self.engine.start()
    }
}
```

### 2. 避免過深繼承

```valkyrie
# 避免
class A {}
class B : A {}
class C : B {}
class D : C {}
class E : D {}  # 繼承層級太深

# 推薦：使用 Trait 組合
trait Flyable { micro fly(self) }
trait Swimmable { micro swim(self) }

class Duck : Animal, Flyable, Swimmable {}
```

### 3. 里氏替換原則

子類應該能夠替換父類而不破壞程式正確性：

```valkyrie
# 好的設計
class Bird {
    micro move(self) {
        self.walk()
    }
    
    micro walk(self) {}
}

class Sparrow : Bird {
    override micro move(self) {
        self.fly()  # 擴展行為，不破壞父類契約
    }
    
    micro fly(self) {}
}
```
