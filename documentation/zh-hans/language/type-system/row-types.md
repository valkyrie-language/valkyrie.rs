# 行类型与行多态 (Row Types & Row Polymorphism)

在当前的 Valkyrie 设计里，`row` 不是“字段结构兼容”的别名，而是方法行约束。它用于表达某个值当前是否提供一组必需的方法签名。

## 语义定义

- `row` 是匿名能力约束，不是具名 `trait`。
- `row` 只描述“你现在会不会这些方法”，不描述“你是谁”。
- `row` 不产生独立 witness，也不参与具名 trait coherence。
- `field` 在语义上视为 `getter + setter` 两个方法，因此 row 判定最终仍落在方法面上。

匿名 `trait` 语法在语义上应解释为 `method row requirement`，而不是另一种具名协议系统。

## 形式化定义

设 `R` 为一个匿名 row requirement，`A` 为某个候选类型。

- `M(A)` 表示 `A` 的公开方法面。
- `M(A) ⊒ R` 表示 `A` 的方法面覆盖 `R` 的全部 requirement。
- 若形参类型写作匿名 row，则实参是否可接受，仅由 `M(A) ⊒ R` 判定。

据此，row 的正式语义规则为：

- row 判定必须只检查方法面。
- row 判定不得直接依赖物理字段布局。
- row 判定可以接受任何能提供所需方法面的值，包括 `class`、匿名对象和局部适配器。
- row 判定结果不得自动提升为具名 `trait` 满足事实。

## 最小示例

```valkyrie
micro invoke_g(value: { g() -> unit }) {
    value.g()
}
```

这里的 `{ g() -> unit }` 仅表示：

- 形参不要求某个具名 `trait`
- 也不要求某个 `class`
- 只要求传入值当前提供 `g() -> unit`

任何满足该方法签名的 `class`、匿名对象或局部适配器都可以传入。

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

对应规则如下：

- 只读属性要求 getter
- 可写属性要求 getter + setter
- row 层只关心调用面，不关心底层是不是对象字段、计算属性还是代理方法

可以形式化写成：

- 只读属性 requirement 归约为 getter method requirement。
- 可写属性 requirement 归约为 getter 与 setter 两个 method requirement。
- 因此 row 层不存在独立的“field 结构兼容”判定。

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

形式化地说，`...R` 只表示“其余未显式枚举的方法 requirement”，不引入新的名义类型关系，也不引入具名协议 identity。

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

例如，下面这种写法应视为非法或未定义：

```valkyrie
# 非法 row 设计
{ type Item, next() -> Option⟨Item⟩ }
```

如果你需要 `Item` 这类协议级事实，应定义具名 `trait`：

```valkyrie
trait Iterator {
    type Item
    micro next(mut self) -> Option⟨Self::Item⟩
}
```

因此，匿名 row 的禁止规则可以写成：

- 匿名 row 不得声明 `associated type`。
- 匿名 row 不得声明默认实现。
- 匿名 row 不得形成独立 witness。
- 匿名 row 不得参与 impl coherence。

## row 与重载解析

在重载或候选选择里，row 的优先级应低于名义类型和具名 trait。

顺序如下：

```text
nominal exact
  > nominal subtype
  > named trait
  > row
```

该顺序表示：

- 具体 `class` 优先于协议约束
- 具名 `trait` 优先于匿名能力约束
- row 只作为最轻量的能力匹配手段

若两个候选都仅通过 row 判定命中，且不存在额外规则能区分二者，则实现应报告歧义，而不是继续发明匿名 row 的偏序。

## row 在编译流水线里的位置

- `row` 应在 `HIR` 类型检查阶段完成满足检查。
- 进入 `MIR` 时，它应已经闭合为具体成员调用。
- 后端不应再看到“开放 row evidence object”。

等价地说：

- `HIR` 必须保留 row closure 的结果。
- `MIR` 不得把 row 继续表示为独立 witness object。
- backend 不得重新执行 row match。

row 文档必须与编译架构文档保持一致：它是前端静态判定，而不是后端可见协议实体。

## 与展开语法的一致性

`...` 语法仍然保留，因为它和对象更新、解构写法保持一致：

```valkyrie
match user {
    case { name, ...rest }: print(name)
}

let new_user = { name: "new", ...old_user }
```

在 row requirement 中使用同样的 `...R`，表示“保留其余 requirement”，而不是重新引入字段结构兼容模型。

## 选型规则

- 仅要求局部方法面时，使用 row。
- 需要具名协议语义，例如关联类型、默认实现或协议身份时，使用具名 `trait`。
- 需要对象类型本体或继承层级时，使用 `class`。
- 需要具名 union 与互斥分支集合时，使用 `unite`。

## 结论

row 关注的是值当前可提供的方法面，而不是对象的外观形状。

- row 是匿名方法约束
- trait 是具名协议
- class 是名义类型
- unite 是具名 union

把这三层分清，才能让类型检查、重载解析、`HIR/MIR` 边界和后端输入保持稳定。

---

**上一页**: [关联类型](./associated-types.md) | **下一页**: [交集与并集](./intersection-union.md)
