# interpreter tests

这里放运行时与集成冒烟测试。

## 职责
- 覆盖真实 fixture 的执行结果。
- 作为 `CLR` 主线之外的反过拟合护栏。
- 优先覆盖 `test.io`、`test.io_smoke`、`test.control_flow_smoke`、`test.control_flow` 这类基础样例。
