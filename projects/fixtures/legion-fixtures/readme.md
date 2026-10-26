# legion-fixtures

Legion 工程测试语料，提供项目运行、自举入口、runtime smoke 和文本输出等样本。测试工具可通过 Rust API 获取语料位置，逐个打开其中的工程。

```rust
use legion_fixtures::root;
let corpus = root().join("cmds_run");
```

该 crate 提供语料根目录，工程执行由调用它的 harness 或编译器测试负责。
