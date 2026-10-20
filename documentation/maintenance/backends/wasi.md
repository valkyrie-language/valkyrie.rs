# Wasm 与 WASI 后端

Wasm 模块编码与具体宿主能力分开。Node、浏览器和 WASI 的导入、ABI 与入口包装由显式 target/profile 和 linker 选择确定。

## 输入与编码

```text
Canonical + RepresentationPlan → Wasm BackendPrivatePlan
  → pre-encode validation → acorn-wasm → Wasm artifact
  → signature-driven host glue / target runtime
```

typeidx/local/table/rec-group、GC 对象与线性内存 FFI carrier 属于私有计划。linker 提供 ImportIndex、签名与 adaptor identity；backend 不按 module/field 名补 ABI，不无条件补宿主导入，不按 callee 尾名决定换行。

宿主字符串的 UTF-16 表示不能修改语言文本的索引语义；转换由已绑定 adaptor 和 ABI 合同实现。null payload 不能默认编码为整数零。

## 验收

所有生产编码入口先验证 callable、import signature、carrier、layout 与 payload。真实验证包含非空 artifact、export、签名驱动传参/返回值、invoke 及完整题意用例；模块验读或 Node wrapper 启动不能替代执行。
