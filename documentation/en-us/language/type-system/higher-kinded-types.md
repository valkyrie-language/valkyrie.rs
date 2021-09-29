# Higher-Kinded Types (HKT)

Higher-Kinded Types (HKT) are an advanced feature of the Valkyrie type system that allow for the abstraction over type constructors, enabling more powerful generic programming patterns.

## Basic Concepts

### 1. Kinds

In Valkyrie, types have different "kinds":

```valkyrie
# Kind *: Concrete type
let x: i32        # The kind of i32 is *
let y: utf8       # The kind of utf8 is *

# Kind * -> *: Unary type constructor
type [_]          # The kind of [_] is * -> *
type Option       # The kind of Option is * -> *

# Kind * -> * -> *: Binary type constructor
type Result⟨T, E⟩ # The kind of Result is * -> * -> *
type {K: V}       # The kind of {K: V} is * -> * -> *

# Kind (* -> *) -> *: Higher-kinded type constructor
type Monad⟨M⟩     # The kind of M is * -> *
```

### 2. Abstraction over Type Constructors

```valkyrie
# Define a higher-kinded type trait
trait Functor where Self: * -> * {
    micro map⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> B) -> Self⟨B⟩
}

# Implementation for a concrete type
imply Option: Functor {
    micro map⟨A, B⟩(self: Option⟨A⟩, f: micro(A) -> B) -> Option⟨B⟩ {
        match self {
            case value: f(value)
            case None: None
        }
    }
}

# Implementation for arrays
imply []: Functor {
    micro map⟨A, B⟩(self: [A], f: micro(A) -> B) -> [B] {
        [ f(item) for item in self ]
    }
}
```

---

## Monad Pattern

### Monad Trait Definition

```valkyrie
# Monad trait
trait Monad where Self: * -> * {
    # Wrap a value into the monad
    micro pure⟨A⟩(value: A) -> Self⟨A⟩
    
    # Monad bind operation
    micro bind⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> Self⟨B⟩) -> Self⟨B⟩
    
    # Convenience method: map can be implemented via bind and pure
    micro map⟨A, B⟩(self: Self⟨A⟩, f: micro(A) -> B) -> Self⟨B⟩ {
        self.bind { Self::pure(f($)) }
    }
}

# Option Monad implementation
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

# Result Monad implementation
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
```

---

## Advanced Application: Lenses (Lens)

Lenses are functional references that solve the problem of accessing and updating data deep within nested and immutable data structures.

### 1. Defining a Lens
A Lens consists of a pair of `get` and `set` functions:
```valkyrie
structure Lens⟨S, A⟩ {
    get: micro(S) -> A,
    set: micro(S, A) -> S,
}
```

### 2. Composing Lenses
The power of Lenses lies in their ability to be composed:
```valkyrie
micro compose⟨S, A, B⟩(l1: Lens⟨S, A⟩, l2: Lens⟨A, B⟩) -> Lens⟨S, B⟩ {
    Lens {
        get: micro(s) { l2.get(l1.get(s)) },
        set: micro(s, b) { l1.set(s, l2.set(l1.get(s), b)) },
    }
}
```

### 3. Scenario: Nested Record Updates
```valkyrie
type Address = { city: utf8, street: utf8 }
type User = { name: utf8, addr: Address }

# Define a Lens for User.addr
let user_addr = Lens⟨User, Address⟩ {
    get: micro(u) { u.addr },
    set: micro(u, a) { { addr: a, ...u } }, # Using row update syntax
}

# Define a Lens for Address.city
let addr_city = Lens⟨Address, utf8⟩ {
    get: micro(a) { a.city },
    set: micro(a, c) { { city: c, ...a } },
}

# Compose them: Create a User -> City Lens
let user_city = compose(user_addr, addr_city)

micro main() {
    let u = User { name: "Alice", addr: { city: "Tokyo", street: "Ginza" } }
    
    # Read deep data
    print(user_city.get(u)) # Tokyo
    
    # Update deep data
    let u2 = user_city.set(u, "Kyoto")
    print(u2.addr.city) # Kyoto
}
```

---

**Previous**: [Type Functions](./type-function.md) | **Next**: [Type-Level Programming](./type-level.md)
