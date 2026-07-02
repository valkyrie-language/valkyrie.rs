# 编译时计算 (Compile-time Computation)

Valkyrie 允许在编译期间执行代码，从而实现零开销的抽象和高度灵活的代码生成。

## 常量表达式 (evaluate)

使用 `evaluate` 编译器内置指令可以强制要求编译器在编译时计算表达式的值。

```valkyrie
# 编译时计算斐波那契数列
let FIB_20: i32 = evaluate(fibonacci(20))

# 编译时格式化字符串
let VERSION: string = evaluate(f"v{}.{}.{}".format(1, 0, 5))
```

## 编译时函数 (@const_fn)

只有标记为 `@const_fn` 的函数才能在编译时被安全地执行。这些函数必须是纯函数（无副作用）。

```valkyrie
@const_fn
micro square(n: i32) -> i32 {
    n * n
}

# 合法调用
let X: i32 = evaluate(square(10))
```

## 编译时反射

编译器提供了一系列内置指令来获取类型信息或环境信息：

- `type_of(expr)`: 获取表达式的类型。
- `name_of(sym)`: 获取符号的名称字符串。
- `env("VAR_NAME")`: 读取编译环境的环境变量。
- `is_defined(sym)`: 检查某个符号是否已定义。

## 外部资源嵌入

你可以在编译时读取外部文件并将其内容嵌入到生成的二进制中：

```valkyrie
# 嵌入文本文件
let SHADER_SOURCE: string = evaluate(read_file("src/shaders/basic.glsl"))

# 嵌入二进制文件
let ICON_DATA: [u8] = evaluate(read_bytes("assets/icon.png"))
```

## 为什么使用编译时计算？

1. **性能**: 将运行时开销转移到编译时。
2. **验证**: 在编译阶段捕获无效的配置或参数。
3. **灵活性**: 根据环境参数（如开发/生产模式）生成不同的代码路径。

---
**相关章节**:
- [宏系统](./macro-system.md) - 更高级的代码生成工具
- [类型函数](../type-system/type-function.md) - 类型层面的编译时计算
