# pipeline

这里放编译主链的入口与阶段编排。

## 职责
- 提供 `ValkyrieRoot -> HIR` 以及后续阶段的明确入口。
- 让各阶段边界显式可见，避免逻辑散落在 CLI 或 runtime。

## 禁止
- 不把 pipeline 目录写成新的统一 `god ir`。
- 不在这里偷偷持有目标运行时细节。
