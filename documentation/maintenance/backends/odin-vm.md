# OVM 与 OdinVM

OVM 目标将已验证的语言 IR 或 residual 经语言与目标适配器降低到 OVM 私有计划及 LIR。普通函数和合成包装共用 LIR 编码，产出 `.odin`。

OdinVM 拥有加载、验证、执行帧、VM 对象、GC 和可选 JIT。语言 runtime 通过显式合同接入，VM 不按函数名恢复语言语义，也不回调 Compiler 修复事实。加载前验证版本、section、跳转、签名和资源需求，再调用本次产物的真实入口。

LegendVM 提供语言自有解释器的 guest 解释与专门化支撑，职责不同。Wasm、CLR、JVM、native 直接进入各自产物链，不经 `.odin`。这些边界不代表全部能力已经交付，须核验对应提交的真实执行证据。
