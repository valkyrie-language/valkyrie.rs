# CLR 后端

CLR 专题规定 IL/程序集表示边界，不声明完整语言或自举能力完成。当前目标验收与其他 target 分开判断。

## 输入与编码

```text
Canonical + RepresentationPlan → CLR BackendPrivatePlan
  → pre-encode validation → acorn-pe → assembly
```

CLR metadata token、方法体、值类型表示和调用约定属于私有计划。语言类型、复制语义、callable instance 与 evidence 来自上游，不通过 System.String、类型名或字段形状重建。

Acorn 拥有 PE/CLI 格式。执行/验读工具可以检查产物，不能代替正式语义编译。

## 验收

缺实例、签名、layout 或导入时，在所属边界结构化失败；不能靠默认 carrier 或外部宿主补编译。合法计划须生成非空程序集，并验证 export/入口及真实调用结果。局部 IL 输出和格式校验不证明题意或自举。
