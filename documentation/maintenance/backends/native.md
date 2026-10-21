# Native 后端

Native 表示由目标私有计划决定。寄存器、机器指令、栈、重定位和镜像格式不进入公共 Semantic MIR。

linker 和表示规划先冻结 callable/import、ABI、carrier 与 layout；backend 只编码经过验证的 TargetExecutionPlan。对应 Acorn crate 负责机器/容器格式，不承担语言语义。

缺身份、ABI 或目标能力结构化失败，禁止以其他目标产物、宿主代编译或默认布局替代。机器码或镜像存在不代表入口与调用正确，必须验证真实执行结果。

Native 专题不改变当前 nyar/wasm 题意验收范围。共同边界见 [后端指南](index.md)。
