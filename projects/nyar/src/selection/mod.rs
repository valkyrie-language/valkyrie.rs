#![doc = include_str!("readme.md")]

use serde::{Deserialize, Serialize};

use crate::{
    abstractions::{BackendInputKind, BinaryTarget},
    packaging::TargetLane,
};

/// 候选后端元信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCandidate {
    /// 后端名。
    pub name: String,
    /// 所属路线。
    pub lane: TargetLane,
    /// 接受的输入种类。
    pub input_kind: BackendInputKind,
    /// 支持目标。
    pub target: BinaryTarget,
    /// 选择优先级，越大越优先。
    pub priority: u16,
}

/// 简单后端选择器。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BackendSelector {
    /// 注册的候选后端。
    pub candidates: Vec<BackendCandidate>,
}

impl BackendSelector {
    /// 注册一个候选后端。
    pub fn register(&mut self, candidate: BackendCandidate) {
        self.candidates.push(candidate);
    }

    /// 为指定路线和输入选择优先级最高的后端。
    pub fn select(&self, lane: TargetLane, input_kind: BackendInputKind) -> Option<&BackendCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.lane == lane && candidate.input_kind == input_kind)
            .max_by_key(|candidate| candidate.priority)
    }
}
