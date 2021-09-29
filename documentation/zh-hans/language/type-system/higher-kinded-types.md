# 高阶类型 (Higher-Kinded Types)

高阶类型（HKT）是 Valkyrie 类型系统的高级特性，允许对类型构造器进行抽象，实现更强大的泛型编程模式。

## 基本概念

### 1. 类型的种类 (Kinds)

在 Valkyrie 中，类型有不同的"种类"：

```valkyrie
# 种类 *：具体类型
let x: i32        # i32 的种类是 *
let y: utf8       # utf8 的种类是 *

# 种类 * -> *：一元类型构造器
type [_]          # [_] 的种类是 * -> *
type Option       # Option 的种类是 * -> *

# 种类 * -> * -> *：二元类型构造器
type Result⟨T, E⟩ # Result 的种类是 * -> * -> *
type {K: V}       # {K: V} 的种类是 * -> * -> *

# 种类 (* -> *) -> *：高阶类型构造器
type Monad⟨M⟩     # M 的种类是 * -> *
```

### 2. 类型构造器抽象

```valkyrie
# 定义高阶类型特征
trait Functor where Self: * -> * {
    micro map⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> B) -> Self⟨B⟩
}

# 为具体类型实现
imply Option: Functor {
    micro map⟨A, B⟩(self: Option⟨A⟩, f: micro(A) -> B) -> Option⟨B⟩ {
        match self {
            case value: f(value)
            case None: None
        }
    }
}

# 为数组实现
imply []: Functor {
    micro map⟨A, B⟩(self: [A], f: micro(A) -> B) -> [B] {
        let mut result = []
        loop item in self {
            result.push(f(item))
        }
        result
    }
}
```

---

## 单子模式 (Monad Pattern)

### 单子特征定义

```valkyrie
# 单子特征
trait Monad where Self: * -> * {
    # 将值包装到单子中
    micro pure⟨A⟩(value: A) -> Self⟨A⟩
    
    # 单子绑定操作
    micro bind⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> Self⟨B⟩) -> Self⟨B⟩
    
    # 便利方法：map 可以通过 bind 和 pure 实现
    micro map⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> B) -> Self⟨B⟩ {
        self.bind { Self::pure(f(%)) }
    }
}

# Option 单子实现
imply Option: Monad {
    micro pure⟨A⟩(value: A) -> Option⟨A⟩ {
        value
    }
    
    micro bind⟨A, B⟩(self: Option⟨A⟩, f: micro(A) -> Option⟨B⟩) -> Option⟨B⟩ {
        match self {
            case value: f(value)
            case None: None
        }
    }
}

# Result 单子实现
imply⟨E⟩ Result⟨_, E⟩: Monad {
    micro pure⟨A⟩(value: A) -> Result⟨A, E⟩ {
        Fine(value)
    }
    
    micro bind⟨A, B⟩(self: Result⟨A, E⟩, f: micro(A) -> Result⟨B, E⟩) -> Result⟨B, E⟩ {
        match self {
            case Fine(value): f(value)
            case Fail(error): Fail(error)
        }
    }
}

---

## 进阶应用：透镜 (Lens)

透镜是一种函数式引用，它解决了在嵌套且不可变的数据结构中进行深层访问和更新的问题。

### 1. 定义 Lens
一个 Lens 由一对 `get` 和 `set` 函数组成：
```valkyrie
structure Lens⟨S, A⟩ {
    get: micro(S) -> A,
    set: micro(S, A) -> S,
}
```

### 2. 组合 Lens
Lens 的强大之处在于它们可以组合：
```valkyrie
micro compose⟨S, A, B⟩(l1: Lens⟨S, A⟩, l2: Lens⟨A, B⟩) -> Lens⟨S, B⟩ {
    Lens {
        get: micro(s) { l2.get(l1.get(s)) },
        set: micro(s, b) { l1.set(s, l2.set(l1.get(s), b)) },
    }
}
```

### 3. 场景：嵌套 Record 更新
```valkyrie
type Address = { city: utf8, street: utf8 }
type User = { name: utf8, addr: Address }

# 定义针对 User.addr 的 Lens
let user_addr = Lens⟨User, Address⟩ {
    get: micro(u) { u.addr },
    set: micro(u, a) { { addr: a, ...u } },
}, # 利用行更新语法
}

# 定义针对 Address.city 的 Lens
let addr_city = Lens⟨Address, utf8⟩ {
    get: micro(a) { a.city },
    set: micro(a, c) { { city: c, ...a } },
}

# 组合它们：创建一个 User -> City 的 Lens
let user_city = compose(user_addr, addr_city)

micro main() {
    let u = User { name: "Alice", addr: { city: "Tokyo", street: "Ginza" } }
    
    # 读取深层数据
    print(user_city.get(u)) # Tokyo
    
    # 更新深层数据
    let u2 = user_city.set(u, "Kyoto")
    print(u2.addr.city) # Kyoto
}
```

---

**上一页**: [类型函数](./type-function.md) | **下一页**: [类型级编程](./type-level.md)
