# Trait System (Trait System)

## Overview (Overview)

Valkyrie's Trait system provides a powerful abstraction mechanism, supporting interface definitions, default implementations, and multiple inheritance. The Trait system is a core component of object-oriented programming in Valkyrie.

## Basic Definitions and Implementations (Basic Definitions and Implementations)

In Valkyrie, a Trait defines the behavior of a type, and the `imply` keyword is used to implement these behaviors for a specific type.

### Simple Trait Definition (Simple Trait Definition)

```valkyrie
trait Display {
    micro fmt(self) -> utf8
}

trait Clone {
    micro clone(self) -> Self
}

trait Debug {
    micro debug_fmt(self) -> utf8 {
        # Default implementation
        f"{self.type_name()}@{self:p}"
    }
}
```

### Basic Implementation (imply) (Basic Implementation (imply))

```valkyrie
class Point {
    x: f64,
    y: f64,
}

imply Point: Display {
    micro fmt(self) -> utf8 {
        f"({self.x}, {self.y})"
    }
}

imply Point: Clone {
    micro clone(self) -> Self {
        Point { x: self.x, y: self.y }
    }
}
```

## Advanced Definitions (Advanced Definitions)

As abstraction needs increase, Traits can include associated types, inherit from other Traits, or add constraints.

### Traits with Associated Types (Traits with Associated Types)

```valkyrie
trait Iterator {
    type Item
    
    micro next(self) -> Self::Item?
    
    micro collect⟨C: FromIterator⟨Self::Item⟩⟩(self) -> C {
        C::from_iter(self)
    }
}

trait FromIterator⟨T⟩ {
    micro from_iter⟨I: Iterator⟨Item = T⟩⟩(iter: I) -> Self
}
```

### Trait Inheritance and Constraints (Trait Inheritance and Constraints)

```valkyrie
trait PartialEq⟨Rhs = Self⟩ {
    micro eq(self, other: Rhs) -> bool
    
    micro ne(self, other: Rhs) -> bool {
        !self.eq(other)
    }
}

# Ord inherits from PartialEq and PartialOrd
trait Ord: PartialEq + PartialOrd {
    micro cmp(self, other: Self) -> Ordering
}
```

## Generics and Conditional Implementation (Generics and Conditional Implementation)

Generics allow Traits to handle multiple types, while conditional implementations allow providing implementations based on the constraints satisfied by the type.

### Generic Implementation (Generic Implementation)

Generics use mathematical angle brackets `⟨ ⟩`.

```valkyrie
imply⟨T: Display⟩ [T]: Display {
    micro fmt(self) -> utf8 {
        let items = self.iter()
            .map { $.fmt() }
            .collect::⟨[utf8]⟩()
            .join(", ")
        f"[{items}]"
    }
}

imply⟨T: Clone⟩ [T]: Clone {
    micro clone(self) -> Self {
        self.iter().map { $.clone() }.collect()
    }
}
```

### Conditional Implementation

```valkyrie
imply⟨T: PartialEq⟩ [T]: PartialEq {
    micro eq(self, other: Self) -> bool {
        self.length == other.length && 
        self.iter().zip(other.iter()).all { $1 == $2 }
    }
}
```

## Trait Objects and Dynamic Dispatch

In some cases, we want to process a set of different types that implement the same Trait. This is where **Trait Objects** come in.

### Trait Object Example

```valkyrie
trait Animal {
    micro make_sound(self)
    micro name(self) -> utf8
}

class Dog {
    name: utf8,
}

imply Dog: Animal {
    micro make_sound(self) {
        print("Woof!")
    }
    
    micro name(self) -> utf8 {
        self.name.clone()
    }
}

class Cat {
    name: utf8,
}

imply Cat: Animal {
    micro make_sound(self) {
        print("Meow!")
    }
    
    micro name(self) -> utf8 {
        self.name.clone()
    }
}

# Using trait objects: put different types into the same array
let animals: [Animal] = [
    Dog { name: "Buddy" },
    Cat { name: "Whiskers" },
]

for animal in animals {
    print("{} says:", animal.name())
    animal.make_sound() # Dynamic dispatch
}
```

## Internal Principles: Witness Table

Valkyrie's dynamic dispatch mechanism relies on the **Witness Table**. This allows Valkyrie to break through the limitations of traditional static polymorphism.

### What is "Witnessing"? (Witnessing Existence)

From a type theory perspective, the Trait system is actually an expression of **Existential Quantification**:

-   **Generics are Universal Quantification ($\forall T$)**: The caller has the power to choose $T$, and the called function must be able to handle any $T$ provided by the caller.
-   **Interfaces are Existential Quantification ($\exists T$)**: The constructor has the power to choose $T$, while the caller only knows that "there exists a type $T$ that satisfies certain constraints," but does not know specifically what $T$ is.

In the existential proposition $\exists T. P(T)$, the specific type $T$ that makes the proposition true is called the **Witness Type**.

### Traditional Dilemma of Existential Types

In many static languages (e.g., Rust), when you upcast an object to a Trait object, the specific witness type information is completely erased. This usually leads to:

1.  **Memory Layout Loss**: The compiler no longer knows the Size and Alignment of the original type, making it impossible to handle these objects directly on the stack.
2.  **Method Call Restrictions**: Methods that return `Self` or have generic parameters are often unusable in dynamic dispatch because the caller loses the ability to "recover" this information.

### Valkyrie's Solution: Open Existential Types

Valkyrie completely solves the aforementioned dilemma through the **Witness Table**. The witness table is not just an index of methods; it also contains the **full metadata** of the type.

-   **Open**: The witness table guides the runtime on how to "open" this opaque existential type, regaining its memory layout and type information.
-   **Full Dynamic Support**: This design ensures that Valkyrie's Trait objects **have no so-called "object safety" restrictions**. Even methods that return `Self` can be safely used in dynamic dispatch because the runtime can dynamically process the return value via the witness table.

### Transformation Principle

During the compilation stage, the Valkyrie compiler transforms Trait definitions and their implementations into the underlying witness table structure:

1.  **Trait Definition -> Layout Template**: The compiler defines a method list template for each Trait.
2.  **imply Implementation -> Witness Table Instance**: For each `imply Class: Trait` block, the compiler generates a specific witness table instance. This table contains pointers (or bytecode indices) to the specific method implementations for that class.
3.  **Trait Object -> Fat Pointer**: A runtime Trait object is actually a "fat pointer" consisting of two parts:
    -   A pointer to the specific data instance.
    -   A pointer to the corresponding witness table.

### Differences between Witness Table and vtable

While both witness tables and traditional virtual function tables (vtable) are used to implement polymorphism, they have significant differences in design philosophy and memory layout:

| Feature | vtable (Traditional OOP) | Witness Table (Valkyrie/Trait System) |
| :--- | :--- | :--- |
| **Binding Relationship** | **Strong Binding**: Embedded in the inheritance hierarchy of the class definition, usually requiring the type to declare the interface at definition time. | **Weak Binding**: Independent of the class definition, defined in `imply` blocks, supporting late binding. |
| **Memory Layout** | **Intrusive**: Each object instance typically contains a pointer to its class vtable internally. | **Non-intrusive**: The object itself maintains its original layout. Polymorphism is implemented by an external "fat pointer" carrying the witness table. |
| **Extensibility** | **Closed**: Usually impossible to add new virtual functions or interface implementations to an existing class from the outside. | **Open**: New Traits can be implemented for any existing type (including primitive types) at any time. |
| **Multiple Inheritance Handling** | **Complex**: Requires handling complex offset calculations or maintaining multiple vtable pointers. | **Flat**: Each Trait corresponds to an independent witness table, with clear combination logic and consistent performance. |

### Advantages of Witness Tables

1.  **Non-intrusive Extension (Retrofitting)**:
    You can implement your own Traits for types in third-party libraries. This breaks the "parent class before child class" hierarchical constraint in traditional OOP.
2.  **Binary Compatibility (ABI Stability)**:
    Adding new Trait implementations to a class does not change the class's memory layout. This means that even if a library is upgraded with new features, old binary code can still safely access the object.
3.  **Zero-Overhead Potential**:
    During the generic instantiation stage, the compiler can directly eliminate the witness table through monomorphization, achieving performance consistent with static calls.
4.  **Dispatch on Demand**:
    Fat pointers are generated only when the code truly needs to treat an object as a `Trait Object`. In regular calls, there is no indirect overhead from a virtual table at all.

### Runtime Example (Bytecode Level)

```valkyrie
# For the Dog implementation, the compiler generates a witness table instance Animal_for_Dog
imply Dog: Animal { ... }

# When this function is executed:
micro shout(animal: Animal) {
    # 1. Get the witness table from the animal fat pointer
    # 2. Look up the index of make_sound in the witness table
    # 3. Call the specific implementation corresponding to that index
    animal.make_sound()
}
```

## Anonymous Classes and Structural Constraints

Valkyrie supports structural constraints through **Anonymous Classes**. This allows you to describe a set of behaviors a type must satisfy without explicitly defining a Trait. This is similar to the `object : Interface { }` syntax in Kotlin, but in Valkyrie, it is unified into the anonymous class system.

### Anonymous Constraint Syntax

You can define a set of temporary behavior requirements using the `class` keyword directly at the type declaration. This is typically used for structural constraints on function parameters.

> **Tip**: While syntactically supported, please avoid using complex anonymous class definitions in function return types. Prefer defining and returning a named Trait.

```valkyrie
# Anonymous class as a type constraint for a parameter
micro process_drawable(drawable: class {
    micro draw(self)
    micro get_bounds(self) -> Rectangle
}) {
    let bounds = drawable.get_bounds()
    print("Drawing object with bounds: {}", bounds)
    drawable.draw()
}
```

### Composition Constraints

Anonymous classes can inherit from existing Traits and add additional members. This "composition constraint" uses the `class: Base1 + Base2 { ... }` syntax.

```valkyrie
# Anonymous constraint combining existing Traits
micro handle_serializable(obj: class: Display + Clone {
    micro serialize(self) -> utf8
}) {
    print("Object: {}", obj.fmt())
    let cloned = obj.clone()
    let serialized = obj.serialize()
    print("Serialized: {}", serialized)
}
```

## Advanced Features

### Associated Constants

```valkyrie
trait MathConstants {
    const PI: f64 = 3.14159265359
    const E: f64 = 2.71828182846
    
    micro circle_area(radius: f64) -> f64 {
        Self::PI * radius * radius
    }
}

class Calculator {}

imply Calculator: MathConstants {}

let area = Calculator::circle_area(5.0)
```

### Trait Aliases

```valkyrie
# Define a trait alias
trait Printable = Display + Debug + Clone

# Use the trait alias
micro print_info⟨T: Printable⟩(item: T) {
    print("Display: {}", item.fmt())
    print("Debug: {}", item.debug_fmt())
    let cloned = item.clone()
    print("Cloned: {}", cloned.fmt())
}
```

## Derivation Macros

Valkyrie provides macros to automatically derive commonly used traits:

```valkyrie
@derive(Debug, Clone, PartialEq, Eq, Hash)
class User {
    id: u64,
    name: utf8,
    email: utf8,
}

@derive(Display)
class Point {
    x: f64,
    y: f64,
}

# Custom derivation behavior
@derive(Debug, Clone)
@derive_display(format = "User({})", field = "name")
class SimpleUser {
    name: utf8,
    internal_id: u64,  # Won't be shown in Display
}
```

## Best Practices

### 1. Trait Design Principles

```valkyrie
# Good design: Single Responsibility
trait Readable {
    micro read(mut self, buffer: mut [u8]) -> Result⟨usize, Error⟩
}

trait Writable {
    micro write(self, data: [u8]) -> Result⟨usize, Error⟩
}

# Combined use
trait ReadWrite: Readable + Writable {}
```

### 2. Associated Types vs. Generic Parameters

```valkyrie
# Use associated types: only one implementation per type
trait Iterator {
    type Item
    micro next(mut self) -> Self::Item?
}

# Use generic parameters: can have multiple implementations
trait From⟨T⟩ {
    micro from(value: T) -> Self
}

# utf8 can be converted from multiple types
imply utf8: From⟨utf8⟩ { ... }
imply utf8: From⟨char⟩ { ... }
imply utf8: From⟨[char]⟩ { ... }
```

### 3. Error Handling

```valkyrie
trait TryFrom⟨T⟩ {
    type Error
    
    micro try_from(value: T) -> Result⟨Self, Self::Error⟩
}

trait TryInto⟨T⟩ {
    type Error
    
    micro try_into(self) -> Result⟨T, Self::Error⟩
}

# Automatic implementation
imply⟨T, U⟩ T: TryInto⟨U⟩ 
where U: TryFrom⟨T⟩ {
    type Error = U::Error
    
    micro try_into(self) -> Result⟨U, Self::Error⟩ {
        U::try_from(self)
    }
}
```

## Summary

Valkyrie's trait system provides:

1.  **Flexible Abstraction**: Define behavioral interfaces through traits.
2.  **Code Reuse**: Via default implementations and generics.
3.  **Type Safety**: Compile-time checking of trait boundaries.
4.  **Dynamic Dispatch**: Runtime polymorphism support via trait objects.
5.  **Anonymous Classes and Structural Constraints**: Support for defining temporary behavioral boundaries via anonymous classes.
6.  **Composition Capability**: Support for combining multiple Traits and extra members via the `class:` syntax.

Proper use of the trait system allows for the creation of code that is both flexible and type-safe.
