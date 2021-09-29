# Functional Programming

Valkyrie is a language deeply influenced by functional programming concepts, providing rich functional features that make code more concise, composable, and easy to test.

## Core Concepts

### First-Class Functions

In Valkyrie, functions are first-class citizens. This means functions can:
- Be assigned to variables
- Be passed as arguments to other functions
- Be returned as values from other functions

### Immutability

Valkyrie encourages the use of immutable data. Although mutability is supported (through the `mut` keyword), variables are immutable by default, which helps reduce side effects and improve concurrency safety.

### Pure Functions

While Valkyrie allows side effects, writing pure functions is recommended. Pure functions have outputs that depend only on their inputs and produce no observable side effects.

## Main Features

### [Anonymous Functions and Closures](./anonymous-functions.md)

Anonymous functions are functions without names, while closures are anonymous functions that can capture variables from their defining environment. Valkyrie provides concise closure syntax and automatic parameter inference.

### [Pattern Matching](./pattern-match.md)

A powerful pattern matching system that allows you to branch based on data structure, supporting destructuring of tuples, arrays, objects, and custom union types.

### Higher-Order Functions

Higher-order functions are functions that accept functions as arguments or return functions. Common built-in higher-order functions include `map`, `filter`, `fold`, etc.

```valkyrie
let numbers = [1, 2, 3, 4, 5]

# Using higher-order functions for composition
let result = numbers
    .filter { $x % 2 == 0 }
    .map { $x * $x }
    .fold(0) { $acc + $item }
```

### Function Composition

By composing simple functions into complex ones, you can build highly modular systems.

## Advantages

- **Conciseness**: Reduces boilerplate code, making logic clearer.
- **Composability**: Build complex functionality by composing small, single-responsibility functions.
- **Testability**: Pure functions are easy to unit test since they don't depend on external state.
- **Safety**: Reduces complexity and potential errors from state management.
