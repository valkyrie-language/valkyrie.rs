# Type System

Valkyrie possesses a highly expressive and rigorous type system designed to balance development efficiency, runtime performance, and compile-time safety.

This chapter will take you from shallow to deep, exploring the various dimensions of Valkyrie's type system.

## Learning Roadmap

Read in the following order for the best learning experience:

### 1. Basic Modeling
- **[Algebraic Data Types (ADT)](./algebraic-data-types.md)**: Understand sum types built with explicitly tagged `unite` definitions and product types (record), which are the core of building data structures.
- **[Union Types (Unite Types)](./union.md)**: Deep dive into explicitly tagged `unite` definitions and their application in pattern matching.
- **[Pointers and References](./pointer-type.md)**: Master raw pointers, reference modifiers, and their relationship with memory layout.

### 2. Abstraction and Polymorphism
- **[Generic Programming](./generics.md)**: Learn how to write parameterized code and how to add constraints through Traits.
- **[Associated Types](./associated-types.md)**: Explore type placeholders in Traits and understand how they simplify complex abstractions.
- **[Row Types and Polymorphism](./row-types.md)**: Master the dynamic expansion of structured data and Row Variables.

### 3. Type Algebra and Hierarchy
- **[Intersections and Unions](./intersection-union.md)**: Understand logical combinations of types (AND/OR relationships).
- **[Variance and Subtyping](./polarity-type.md)**: Dive into algebraic subtyping theory, mastering Polarity, Covariance, and Contravariance.

### 4. Metaprogramming and High-Level Abstractions
- **[Type Functions](./type-function.md)**: Learn how to transform types through functions at compile time.
- **[Higher-Kinded Types (HKT)](./higher-kinded-types.md)**: Explore abstractions over type constructors (like Functor and Monad).
- **[Type-Level Programming](./type-level.md)**: Utilize the type system for compile-time computation and static proof.

### 5. Cutting-Edge Features
- **[Dependent Types](./dependent-types.md)**: Blur the boundaries between values and types to achieve extremely rigorous static constraints.
- **[Linear Types](./linear-types.md)**: Achieve precise resource management through ownership and usage restrictions.
- **[Effect Types](./effect-type.md)**: Statically track side effects, achieving a perfect combination of pure functions and algebraic effects.

---

## Core Design Philosophy

Valkyrie's type system is based on the following principles:

1. **Composition over Inheritance**: Provides more flexible data composition through ADTs and row types.
2. **Explicit over Implicit**: Key type conversions and effect tracking must be clearly visible.
3. **Zero-Cost Abstractions**: All type hierarchies produce no runtime overhead after monomorphization.
4. **Algebraic Consistency**: Relationships between types (subtyping, intersection, and union) follow strict mathematical lattice theory.

---

**Next**: [Algebraic Data Types (ADT)](./algebraic-data-types.md)
