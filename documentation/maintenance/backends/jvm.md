# JVM 后端

JVM 后端将 Valkyrie 编译为 **手写 Java class 文件**（再可打包为 jar）。正式路径与 seed 同构，不依赖 `javac` 或外部字节码库作为生成器。

## 编译流水线

```text
FragmentSubmission (含 MIR)
  -> nyar-emitter/lowering/backends/jvm  (指令与类结构)
  -> std-data/binary/class               (class 文件编解码)
  -> .class / .jar
```

- **入口**：`projects/nyar-emitter/src/lowering/backends/jvm/`（`mod.rs`、`mir.rs` 等）。
- **二进制**：`projects/std-data/src/binary/class/`。
- 栈机：从 MIR 生成操作数栈指令，不做面向寄存器机的 LIR 分配。

## 硬性约束

| 允许 | 禁止 |
| --- | --- |
| 手写 class 常量池与 Code 属性 | `javac`、ASM、Javassist 等作为**生成**依赖 |
| `java` 仅用于运行/验读 | 语言层名为 `string` 的类型 |

## 类型系统

语言类型映射到 JVM 描述符时：

| Valkyrie | JVM 描述符 / 说明 |
| --- | --- |
| `i32` | `I` |
| `i64` | `J` |
| `f32` | `F` |
| `f64` | `D` |
| `bool` | `Z` |
| `utf8` / `Utf8Text` | `Ljava/lang/String;`（语义：Unicode **标量**） |
| `utf16` / `Utf16Text` | `Ljava/lang/String;`（语义：UTF-16 **code unit**） |
| `unit` | 返回位常用 `V`（**仅 ABI**；ADT 代数 = 1） |
| `void`（`NyarType::Bottom`） | 永不返回 / 空类型（ADT 代数 = 0）；**不得与 `unit` 混用** |
| `class` / `sealed class` / `unite` | `Lpath/to/Class;` |

**没有**语言类型 `string`。宿主 `java/lang/String` 只是编码落点。

文本 API 分叉（与 CLR 对称）：

- `Utf16Text.length` / 切片可对齐 `String.length` / `substring`（code units）。
- `Utf8Text` 的 length/slice/indexing 走标量语义（`std.adaptor.jvm.text`），禁止误降为 code-unit API。

## 表达式与调用（摘要）

- 算术 / 比较 / 位运算按 JVM 指令族映射（`iadd`、`if_icmp*`、`lcmp` 等）。
- 引用类型（含 utf8/utf16、class、unite）比较用 `if_acmp*`。
- 函数默认发射为类中的 `static` 方法；动态调用可走 `MethodHandle`。
- 泛型在描述符中擦除为 `Ljava/lang/Object;`，可用 `Signature` 属性保留信息。

## 代数效应

- 非 resume 的 `raise` ≈ `athrow`。
- Handler 可映射 `exception_table`；完整 continuation 见 `jvm/suspend.rs` 路线。

## 当前进度（以源码为准）

- [x] Class 文件结构与常量池
- [x] 基本类型映射（含 **utf8 / utf16 / unit**，无语言 `string`）
- [x] 算术、位运算、比较、数组与 `invokestatic` 基础路径
- [x] 效应与 suspend 相关骨架
- [ ] 更完整的 `invokevirtual` / `invokeinterface`、调试属性、闭包/`invokedynamic`

短期：字段深度访问、调用种类补齐、与 `valkyrie.v` class 包同构验证。  
长期：增量 class、与 Java 标准库互操作性能——生成路径仍须保持手写。
