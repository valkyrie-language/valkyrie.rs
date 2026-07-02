# x86_64

自研 x86-64 指令编码层，服务 Windows MSVC native 后端。

## 职责

- 提供最小 `X64Instruction` IR 与两遍编码器。
- 封装 Microsoft x64 调用约定辅助（shadow space、参数寄存器）。

## 边界

- 只负责机器码字节，不写 COFF/PE 容器。
- 不承载语言语义；driver 负责把 fragment 降级为指令序列。
