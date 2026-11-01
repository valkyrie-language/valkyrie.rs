using asgard;

# 纯静态示例的 Asgard 配置入口（03 §4：唯一入口，普通 V 模块，最后有效表达式返回配置对象）。
# 作者形态为规范规定的 `define_config({...})`，参数是 Von 对象字面量；不使用旧
# `define_config(asgard) {}` 文本 DSL。字段名与取值形状对齐结构校验合同
# `resolve_static_config`（根：artifact/assets/build/serve/dev；artifact：target/entry/output/base/chunks），
# 未声明的可选项按合同默认值补全，不写 `Some`/`None` 包装，不引用尚未冻结的 SDK 类型拼写。
# 注意：配置求值（C0/C2）尚未接入 CLI，本文件当前只作为存在性准入与作者面合同形态存在；
# 入口/输出/基址的生效证据仍以配置驱动构建接入为准（见 harness `asgard_static` 缺口证据用例）。
define_config({
    artifact: {
        target: "static",
        entry: "source/index.awsl",
        output: "dist",
        base: "/"
    }
})
