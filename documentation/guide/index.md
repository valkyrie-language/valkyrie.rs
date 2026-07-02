# Valkyrie 语言快速入门

欢迎使用 Valkyrie 语言！Valkyrie 是一个现代化的函数式编程语言，提供强大的类型系统、灵活的模块系统和丰富的语言特性。

## 什么是 Valkyrie？

Valkyrie 是一个多范式编程语言，它提供：

- 🎯 **强大的类型系统**：支持泛型、高阶类型、类型推导等高级特性
- 🚀 **现代语法**：简洁直观的语法，支持模式匹配、闭包等现代特性
- 🔒 **内存安全**：垃圾回收器自动管理内存，避免内存泄漏
- ⚡ **高性能**：零成本抽象，编译时优化
- 🔧 **灵活的模块系统**：基于命名空间的模块组织方式

## 快速体验

### Hello World

```valkyrie
micro main() {
    print("Hello, Valkyrie!")
}
```

### 基本运算

```valkyrie
# 变量定义
let name = "Alice"
let age = 30

# 函数定义
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# 模式匹配
match value {
    case 1: "one"
    case 2: "two"
    case _: "other"
}
```

## 下一步

- **[语言特性指南](./features.md)** - 深入了解 Valkyrie 的核心特性
- **[语言教程](./tutorial.md)** - 逐步学习 Valkyrie 的语法和特性
- **[语言参考](../language/index.md)** - 完整的语言规范
- **[示例集合](../examples/index.md)** - 各领域的完整示例
