# 语义到目标的降低边界

降低只翻译已经确定的语义，不创造声明、类型、evidence 或调用事实。

## 阶段合同

Valkyrie frontend 完成名称、类型、place、泛型与 evidence 绑定，将语法糖展开为普通调用或控制流。Semantic MIR 保留普通 Invoke、结构操作与稳定身份。

linker 选择 adaptor/import，Canonical validation 验证跨包身份和完整实例。优化保留语义；Representation Planner 为稳定语义 ID 选择稀疏 carrier/layout 行。BackendLowering.prepare 生产私有计划，backend 通过 pre-encode validation 后才编码。

Acorn 拥有外部二进制格式；Nyar 自有字节码由 nyar-bytecode 拥有。编码器不能通过名称、布局形状或默认值修复上游语义。

## 特性约束

- 模式匹配的类型、穷尽性、FieldId/VariantId 与 owner 在语义阶段确定，后端只实现已定控制流与表示。
- effect/handler 与 trait evidence 保留结构化事实；支持范围由目标合同校验，不能伪装静态成功。
- 名义类型的值/引用复制语义由声明生产，不由 layout 反推。
- utf8/utf16 与 void/unit 的语言区别必须保留，目标 ABI 折叠由显式表示合同说明。

## 验收

同声明不同实例不得混用；缺 field/variant/callable/evidence 在所属阶段拒绝。每项降低同时有合法行为和缺事实失败的验证，不以单个 target 指令输出证明语义正确。
