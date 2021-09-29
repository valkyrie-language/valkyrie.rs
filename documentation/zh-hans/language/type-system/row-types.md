# 行类型与行多态 (Row Types & Row Polymorphism)

在当前的 Valkyrie 设计里，`row` 不是“字段结构兼容”的别名，而是更轻量的 **方法行约束**。它主要用于表达：某个值当前是否提供一组必需的方法签名。

## 核心定位

- `row` 是匿名能力约束，不是具名 `trait`。
- `row` 只描述“你现在会不会这些方法”，不描述“你是谁”。
- `row` 不产生独立 witness，也不参与具名 trait coherence。
- `field` 在语义上视为 `getter + setter` 两个方法，因此 row 判定最终仍落在方法面上。

换句话说，匿名 `trait` 语法在语义上应理解为 `method row requirement`，而不是另一种具名协议系统。

## 基本例子

```valkyrie
micro invoke_g(value: { g() -> unit }) {
    value.g()
}
```

这里的 `{ g() -> unit }` 表示：

- 形参不要求某个具名 `trait`
- 也不要求某个 `class`
- 只要求传入值当前提供 `g() -> unit`

如果某个 `class`、匿名对象或局部适配器都满足这个方法签名，它们都可以传入。

## row 判定是方法判定，不是字段判定

Valkyrie 中的属性访问最终会收敛为方法语义，因此 row 层不直接检查“物理字段”。

```valkyrie
micro rename(value: {
    get_name() -> utf8,
    set_name(utf8) -> unit,
}) {
    let old = value.get_name()
    value.set_name(old + "!")
}
```

可以把它理解为：

- 只读属性要求 getter
- 可写属性要求 getter + setter
- row 层只关心调用面，不关心底层是不是对象字段、计算属性还是代理方法

## 开放行

row 仍然支持“保留其余方法”的开放形式。

```valkyrie
micro use_clock⟨R⟩(value: {
    now() -> i64,
    ...R
}) -> i64 {
    value.now()
}
```

这里的 `R` 表示“除 `now()` 之外的其余方法 requirement”。它的作用是：

- 让约束保持开放
- 不抹掉原值上其他可用的方法事实

## row 不是名义子类型

需要特别区分三件事：

- `row`：你会不会这些方法
- `trait`：你是否满足某个具名协议
- `class`：你是不是这个名义类型层级里的成员
- `unite`：你当前属于哪一个已声明 variant

因此以下判断不成立：

- 某个 `class` 只因为方法长得像，就自动成为另一个 `class`
- 某个值只因为方法长得像，就自动成为某个 `unite`
- 某个匿名 row 只因为方法集相同，就自动等同于具名 `trait`

## row 与 trait 的区别

下面两种写法看起来相近，但语义并不相同：

```valkyrie
micro f1(x: { write(utf8) -> unit }) { ... }

trait Writer {
    micro write(self, text: utf8) -> unit
}

micro f2⟨T: Writer⟩(x: T) { ... }
```

区别在于：

- `f1` 只要求方法行满足
- `f2` 要求具名协议 `Writer`
- `f2` 若通过结构化满足进入主链，也必须收敛成具名 witness
- `f1` 则不产生独立 witness

## row 不支持关联类型

匿名 row 不是 trait system 的一部分，因此不支持：

- `associated type`
- 默认实现
- trait inheritance
- 独立 impl/witness 身份

例如下面这种写法应被视为非法或未定义：

```valkyrie
# 不推荐，也不应视为合法 row 设计
{ type Item, next() -> Option⟨Item⟩ }
```

如果你需要 `Item` 这类协议级事实，应定义具名 `trait`：

```valkyrie
trait Iterator {
    type Item
    micro next(mut self) -> Option⟨Self::Item⟩
}
```

## row 与重载解析

在重载或候选选择里，row 的优先级应低于名义类型和具名 trait。

推荐顺序是：

```text
nominal exact
  > nominal subtype
  > named trait
  > row
```

这保证了：

- 具体 `class` 优先于协议约束
- 具名 `trait` 优先于匿名能力约束
- row 只作为最轻量的能力匹配手段

## row 在编译流水线里的位置

- `row` 应在 `HIR` 类型检查阶段完成满足检查。
- 进入 `MIR` 时，它应已经闭合为具体成员调用。
- 后端不应再看到“开放 row evidence object”。

这也是为什么 row 文档必须和编译架构文档保持一致：它是前端静态判定，不是后端可见协议实体。

## 与展开语法的一致性

`...` 语法仍然保留，因为它和对象更新、解构写法保持一致：

```valkyrie
match user {
    case { name, ...rest }: print(name)
}

let new_user = { name: "new", ...old_user }
```

在 row requirement 中使用同样的 `...R`，表示“保留其余 requirement”，而不是重新引入字段结构兼容模型。

## 使用建议

- 想表达“只要会这些方法就行”，用 row。
- 想表达“这是一个真正的协议，有关联类型、默认实现或协议身份”，用具名 `trait`。
- 想表达“这是一个对象类型本体或继承层级”，用 `class`。
- 想表达“这是一个具名 union 与一组互斥分支”，用 `unite`。

## 总结

row 的核心不是“对象长得像什么”，而是“这个值当前能做什么”。

- row 是匿名方法约束
- trait 是具名协议
- class 是名义类型
- unite 是具名 union

把这三层分清，才能让类型检查、重载解析、`HIR/MIR` 边界和后端输入保持稳定。

---

**上一页**: [关联类型](./associated-types.md) | **下一页**: [交集与并集](./intersection-union.md)
