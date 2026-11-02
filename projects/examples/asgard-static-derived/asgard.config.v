using asgard;

# 事件派生状态示例的 Asgard 配置入口（唯一入口，普通 V 模块，最后有效表达式返回配置对象）。
# 作者形态为规范规定的 `define_config({...})`，不使用旧文本 DSL；未声明可选项按合同默认值补全。
define_config({
    artifact: {
        target: "static",
        entry: "source/index.awsl",
        output: "dist",
        base: "/"
    }
})
