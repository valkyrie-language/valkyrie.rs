# JVM 后端

JVM backend 将已绑定语义与表示计划降低为私有 class/方法计划，不重新解析语言。

## 输入与编码

```text
Canonical + RepresentationPlan → JVM TargetExecutionPlan
  → pre-encode validation → acorn-jvm → class / jar
```

descriptor、constant-pool index、方法/字段编号、操作数栈与 Code 属性属于私有计划。语言类型、泛型实例、调用和 text 语义由上游冻结，不能因宿主引用表示相同而混用。

linker 选择 adaptor/import，表示规划选择擦除或保留方式；backend 不能统一按引用比较、方法名或类型名猜测语言操作。

## 验收

缺 callable、import signature、carrier 或 layout 必须失败。格式有效还需真实入口调用和结果校验；class 文件存在、单条指令输出或某个 fixture 通过不证明全部语言能力完成。
