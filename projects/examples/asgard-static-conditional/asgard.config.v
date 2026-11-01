using asgard;

# 条件渲染形态示例的 Asgard 配置入口（03 §4：唯一入口，普通 V 模块）。
# 作者形态为规范规定的 `define_config({...})`，字段与取值形状对齐结构校验合同
# `resolve_static_config`，未声明的可选项按合同默认值补全。
# 注意：配置求值尚未接入 CLI，本文件当前只作为存在性准入与作者面合同形态存在。
define_config({
    artifact: {
        target: "static",
        entry: "source/index.awsl",
        output: "dist",
        base: "/"
    }
})
