using asgard;

# 桌面 WebView 示例的 Asgard 配置入口（03 §4：唯一入口，普通 V 模块，最后有效表达式返回配置对象）。
# 首轮 `artifact.target` 只支持 `static`（03 §6），桌面差异在 target 策略与宿主打包，不在配置拼写；
# 作者形态为规范规定的 `define_config({...})`，字段与取值形状对齐结构校验合同
# `resolve_static_config`，未声明的可选项按合同默认值补全。
# 注意：配置求值（C0/C2）尚未接入 CLI，本文件当前只作为存在性准入与作者面合同形态存在；
# 桌面 WebView 的打包启动/宿主桥自动化仍待宿主选型（06 §3），不以静态预渲染冒充桌面交付。
define_config({
    artifact: {
        target: "static",
        entry: "source/index.awsl",
        output: "dist",
        base: "/"
    }
})
