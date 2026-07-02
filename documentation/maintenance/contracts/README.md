# Neutral Contracts

`nyar-types::neutral_contract` 提供最小的跨层契约：`Provenance`、`SemanticObservation`、`ArtifactContract`、`PrimitiveDefinition`、`PrimitiveRegistry` 和 `EvidenceStatus`。

这些类型不承载 backend 栈、runtime state 或 artifact index。所有生成字段必须能追溯到 source declaration、definition registry 或显式 contract transformation；缺失 provenance 时 verifier 必须 fail closed。
