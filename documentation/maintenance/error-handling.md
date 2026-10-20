# 诊断与语义准入

诊断由发现事实缺失的阶段生产，Compiler 汇总。统一展示不能抹除事实所有者，也不能把所有错误包装为 ParseError。

## 诊断字段与归属

诊断保留 stage、code、severity、owner、source/span 和相关位置。消息渲染与稳定数据分开，源码位置使用对应输入快照。

- 输入层：读取、解码、manifest 与 package graph 错误。
- Oak：语法、missing/error 与恢复诊断。
- Valkyrie frontend：名称、类型、place、调用、约束与 evidence。
- Nyar：Canonical/IR、实例、表示、私有计划与执行验证。
- Acorn：二进制格式与编码。
- ArtifactStore/Legion：原子提交、缓存身份、产物与执行编排。

缺布局不能提示检查括号，lowering panic 不能统一包装为模板未展开。正常用户错误使用结构化失败，不依赖 catch_unwind 作为阶段协议。

## 恢复与准入

Oak 同时返回可用树、恢复状态与诊断，保留后续有效声明；缺 token 不补成普通语法。frontend 按可达语义准入，可达坏 body 不能进入成功 MIR，不可达坏 body 诊断不能自动阻断入口。

输入读取/解码、声明索引和 body 错误分别定义阻断范围。Lint 消费树或语义事实提出维护建议，不建立额外语言合法性；工程 check/LSP 共享同一依赖快照。

## 验收

验证阶段、code、owner、source/span 和阻断范围，不只匹配报错文本。不可达/可达错误、恢复后有效声明、未声明依赖、缺实例/evidence 与缺 backend 输入均须有对应性质验证。
