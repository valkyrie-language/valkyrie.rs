# 密封类 (Sealed Classes)

## 概述

`sealed class` 是 Valkyrie 当前存在的正式语法，用来定义封闭类层级。

`unite` 也仍然是当前语法，但它不是独立于对象系统的另一套东西。更准确地说：

- `unite` 的默认表示是抽象类
- `unite variant` 是该 `unite` 之下的内部抽象变体类型
- 这组 variant 的集合是封闭的，因此可以做穷尽性检查
- `[tag(XXXKind)]` 是可选优化，用于要求 tagged union
- 还有少数利基优化，但通常不需要在前端专门展开

因此，`sealed class` 这一页讲的是显式写法；`unite` 则是更紧凑的写法。

## 显式写法

当你希望把封闭层级完整展开时，可以直接写 `sealed class`：

```valkyrie
sealed class Result<T, E> {
    abstract get is_fine(self) -> bool
    abstract get is_fail(self) -> bool
}
```

这种写法强调的是：

- 抽象基类显式可见
- variant 层级由显式封闭类族承载
- 变体集合是封闭的
- 对象成员和层级关系都直接写在语法里

## 紧凑写法

如果你的重点是声明互斥分支，而不是手写整套类层级，可以使用 `unite`：

```valkyrie
unite Result<T, E> {
    Fine { value: T }
    Fail { error: E }
}
```

这里更准确的理解方式是：

- `Result<T, E>` 默认表示为一个抽象类
- `Fine / Fail` 是它的 `variant`
- 每个 `variant` 对应内部的抽象变体类型
- 穷尽性检查来自这组已声明 variant

## 可选的 tagged union

默认情况下，不需要写 `tag`。只有在你明确要选择 tagged union 形态时，才额外标注：

```valkyrie
[tag(ResultKind)]
unite Result<T, E> {
    Fine { value: T }
    Fail { error: E }
}
```

这表示你在要求一种特定表示优化，而不是说所有 `unite` 都自动带 tag。

## 与普通 `class` 的区别

- `class` 处理普通对象与继承层级
- `sealed class` 处理显式封闭的类层级
- `unite` 用紧凑语法声明一个抽象类及其封闭 variant 集合

它们回答的问题不同：

- `class`：你是不是这个对象层级里的成员
- `unite`：你当前属于哪一个 variant

## 一句话结论

`sealed class` 是正式语法；`unite` 的默认表示是抽象类，而 `unite variant` 是其内部抽象变体类型。`[tag(XXXKind)]` 只是可选优化，利基优化则属于少数特殊 lowering。
