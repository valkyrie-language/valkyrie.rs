# Witness Table

该模块定义 Witness Table 的核心数据结构。

## 概述

Witness Table 是 trait 实现的运行时表示，
用于动态方法分发。每个 `impl Trait for Type` 
都会生成一个 Witness Table。

## 数据结构

```text
struct WitnessTable {
    trait_id: Identifier,
    type_id: Identifier,
    methods: Vec<WitnessMethod>,
    associated_types: Vec<AssociatedType>,
}
```

## 方法分发

Witness Table 支持以下分发方式：

- **静态分发**: 编译时已知具体类型
- **动态分发**: 通过 vtable 运行时查找
