# 常见问题 (FAQ)

本页面收集了 Valkyrie 开发过程中的常见问题和解答。

## 语言基础

### Q: Valkyrie 与其他函数式语言有什么区别？

A: Valkyrie 的独特之处在于：
- **代数效应系统**：原生支持代数效应，优雅处理副作用
- **多目标编译**：可编译到 WebAssembly、JavaScript 和原生代码
- **现代语法**：结合函数式特性与现代语法设计
- **渐进式采用**：可与现有 JavaScript/TypeScript 项目无缝集成
- **强类型推导**：先进的类型系统，减少显式类型注解

### Q: 什么是代数效应？为什么重要？

A: 代数效应是一种处理副作用的抽象机制：
- **统一抽象**：将异常、异步、状态管理等副作用统一处理
- **可组合性**：效应可以自由组合和嵌套
- **控制反转**：调用者决定如何处理效应，而不是被调用者
- **类型安全**：效应在类型系统中得到体现

```valkyrie
class State<T> {
    get(): T
    set(value: T): void
}

micro counter() -> i32 {
    let current = @State::get()
    @State::set(current + 1)
    current + 1
}
```

### Q: Valkyrie 支持哪些数据类型？

A: Valkyrie 支持丰富的类型系统：
- **基础类型**：i32, f32, utf8, bool, void
- **容器类型**：List⟨T⟩, Array⟨T⟩, Map⟨K, V⟩, Set⟨T⟩
- **可选类型**：Option⟨T⟩ (Some { value: T } | None)
- **结果类型**：Result⟨T, E⟩ (Fine { value: T } | Fail { error: E })
- **函数类型**：(A, B) -> C
- **代数数据类型**：自定义的 sum 与 product 类型
- **效应类型**：带有效应标注的函数类型

## 语法与特性

### Q: 如何定义 与 使用代数数据类型？

A: 使用 `unite` 定义。它的默认表示是抽象类；如果需要显式的 tagged union，可以额外写 `[tag(XXXKind)]`，语言不会自动生成 tag：

```valkyrie
// Sum 类型（联合类型）
unite Result⟨T, E⟩ {
    Fine { value: T };
    Fail { error: E };
}

// Product 类型（结构体）
structure User {
    id: i32;
    name: utf8;
    email: utf8?;
}

// 模式匹配
match result {
    case Fine { value }: print("成功: {value}");
    case Fail { error }: print("错误: {error}");
};
```

### Q: 如何处理异步操作？

A: Valkyrie 支持原生的 `async/await` 语法：

```valkyrie
micro fetch_user_data(id: i32) -> User {
    let response = fetch("/api/users/{id}").await;
    parse_json(response);
}

// 顶层也可以使用 await
let user = fetch_user_data(42).await;
print("User: {user.name}");
```

### Q: 如何进行错误处理？

A: Valkyrie 提供多种错误处理方式：

```valkyrie
// 1. 使用 Result 类型
micro divide(a: f64, b: f64) -> Result⟨f64, utf8⟩ {
    if b == 0.0 {
        Fail { error: "除零错误" };
    } else {
        Fine { value: a / b };
    };
}

// 2. 使用异常效应
micro safe_divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        raise "除零错误"
    } else {
        a / b
    }
}
```

## 编译 与 部署

### Q: Valkyrie 如何编译到不同目标？

A: Valkyrie 支持多目标编译：

```bash
# 编译到 WebAssembly
legion build --target wasm

# 编译到 JavaScript
legion build --target js

# 编译到 原生代码
legion build --target native

# 编译到 TypeScript 定义
legion build --target ts-defs
```

### Q: 如何与现有 JavaScript 项目集成？

A: Valkyrie 提供无缝集成：

```valkyrie
// 导入外部模块
using hxo::std::fetch;
using hxo::std::console;

// 公有函数定义
@export(js)
micro greet(name: utf8) -> utf8 {
    "Hello, {name}!"
}

// 使用 JavaScript 对象
micro process_data(data: JSObject) -> JSObject {
    // 处理逻辑
    data
}
```

### Q: 性能如何？有哪些优化？

A: Valkyrie 提供多种性能优化：
- **尾调用优化**：自动优化尾递归
- **内联优化**：小函数自动内联
- **死代码消除**：移除未使用的代码
- **效应优化**：编译时优化效应处理
- **内存管理**：智能的垃圾回收 与 内存复用

## 工具 与 生态

### Q: 有哪些开发工具支持？

A: Valkyrie 提供完整的工具链：
- **编译器**：`legion` CLI 工具
- **包管理器**：内置的依赖管理
- **格式化工具**：`legion fmt` 代码格式化
- **语言服务器**：VS Code、Vim 等编辑器支持
- **调试器**：源码级调试支持
- **测试框架**：内置单元测试 与 集成测试

### Q: 配置文件格式是什么？

A: Valkyrie 使用 `voc.config.von` 作为配置文件：

```von
name: "my-project"
version: "0.1.0"
dependencies: {
    "std": "0.1.0"
}
```

### Q: 如何管理多包工作区？

A: 使用 `legions.von` 管理工作区：

```von
workspace: {
    members: [
        "packages/*"
    ]
}
```

### Q: 如何编写和运行测试？

A: 使用内置测试框架：

```valkyrie
// 单元测试
#test
micro test_addition() {
    @assert_eq(add(2, 3), 5)
    @assert_eq(add(-1, 1), 0)
}

// 属性测试
#test
micro test_addition_commutative() {
    forall (a: i32, b: i32) {
        @assert_eq(add(a, b), add(b, a))
    }
}

// 效应测试
#test
micro test_state_effect() {
    let result = try {
        counter()
    } catch State::get || {
        resume 0;
    } catch State::set |value| {
        resume ();
    }
    @assert_eq(result, 1);
}
```

### Q: 如何管理项目依赖？

A: 使用 `voc.config.von` 配置文件：

```von
{
    name: "my-project",
    version: "0.1.0",
    authors: ["Your Name <your.email@example.com>"],
    dependencies: {
        std: "1.0",
        http: "0.3",
        json: "0.2"
    },
    build: {
        targets: ["js", "wasm"],
        optimization: "release"
    }
}
```

## 学习和社区

### Q: 如何学习 Valkyrie？

A: 推荐的学习路径：
1. **基础语法**：从函数式编程概念开始
2. **类型系统**：理解代数数据类型和模式匹配
3. **代数效应**：掌握效应的定义和处理
4. **实践项目**：构建小型应用程序
5. **高级特性**：学习性能优化和工具使用

### Q: 有哪些学习资源？

A: 可用的学习资源：
- **官方教程**：[快速开始指南](/guide/getting-started)
- **示例项目**：[代码示例](/examples/)
- **API 文档**：完整的标准库文档
- **社区论坛**：GitHub Discussions
- **视频教程**：YouTube 频道

### Q: 如何贡献到 Valkyrie 项目？

A: 贡献方式：
1. **报告问题**：提交 bug 报告和功能请求
2. **改进文档**：完善文档和示例
3. **编写代码**：实现新功能或修复问题
4. **测试反馈**：使用预发布版本并提供反馈
5. **社区支持**：帮助其他用户解决问题

### Q: Valkyrie 的发展路线图是什么？

A: 主要发展方向：
- **语言特性**：模块系统、宏系统、并发原语
- **工具改进**：更好的错误信息、调试体验、IDE 支持
- **性能优化**：编译速度、运行时性能、内存使用
- **生态建设**：标准库扩展、第三方包、框架支持
- **平台支持**：更多编译目标、移动平台、嵌入式系统

---

如果您的问题没有在这里找到答案，请：
- 查看 [官方文档](/guide/)
- 提交 [GitHub Issue](https://github.com/valkyrie-lang/valkyrie/issues)
- 参与 [社区讨论](https://github.com/valkyrie-lang/valkyrie/discussions)
- 加入 [Discord 社区](https://discord.gg/valkyrie-lang)
