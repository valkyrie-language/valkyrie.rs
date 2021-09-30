//! `CLR` 路线后端公共协议。
//!
//! 这里定义 `CLR` 路线共享的产物口味与命名规则，
//! 由具体的 `clr-backend` 去实现真实 lowering 和二进制编码。

use serde::{Deserialize, Serialize};

use crate::abstractions::ArtifactKind;

/// `CLR` 镜像口味。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClrImageKind {
    /// 带托管入口点的可执行镜像。
    Executable,
    /// 不带入口点的托管动态库。
    DynamicLibrary,
}

impl ClrImageKind {
    /// 根据是否存在入口点推断镜像口味。
    pub fn infer(has_entry_point: bool) -> Self {
        if has_entry_point {
            Self::Executable
        }
        else {
            Self::DynamicLibrary
        }
    }

    /// 返回对应的产物种类。
    pub fn artifact_kind(self) -> ArtifactKind {
        match self {
            Self::Executable => ArtifactKind::Executable,
            Self::DynamicLibrary => ArtifactKind::DynamicLibrary,
        }
    }

    /// 返回推荐的文件扩展名。
    pub fn file_extension(self) -> &'static str {
        match self {
            Self::Executable => "exe",
            Self::DynamicLibrary => "dll",
        }
    }
}
