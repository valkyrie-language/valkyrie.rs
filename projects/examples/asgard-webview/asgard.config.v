using asgard;

# 桌面 WebView 示例的 Asgard 配置入口：普通 V 模块，最后一个表达式返回有类型的
# `AsgardConfig`。字段集合由 `asgard` SDK 定义，这里只声明本示例需要的项。
define_config(AsgardConfig {
    artifact: ArtifactConfig {
        target: Some(ArtifactTarget.Static)
        entry: "source/index.w"
        output: Some("dist")
        base: Some("/")
        chunks: None
    }
    assets: None
    build: None
    serve: None
    dev: None
})
