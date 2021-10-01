#![doc = include_str!("readme.md")]

pub mod context;
pub mod r#enum;
pub mod expr;
pub mod function;
pub mod identifier;
pub mod r#impl;
pub mod module;
pub mod property;
pub mod statement;
pub mod r#struct;
pub mod r#trait;
pub mod type_family;
pub mod types;
pub mod visibility;
pub mod widget;

pub use types::{AccessLevel, *};

pub use context::RenameContext;
pub use expr::{CaptureMode, CaptureStorage, HirCallableDomain, HirCapture, HirExpr, HirExprKind, HirResolvedCall};
pub use function::HirFunction;
pub use identifier::HirIdentifier;
pub use module::{HirDocumentation, HirModule};
pub use property::HirProperty;
pub use r#enum::{HirEnum, HirFlagMember, HirFlags, HirVariant};
pub use r#impl::{HirDerive, HirImpl, HirWhereConstraint};
pub use r#struct::{AbstractPropertyRequirement, HirField, HirParent, HirStruct, InheritancePermission};
pub use r#trait::{HirAssociatedConst, HirAssociatedType, HirTrait};
pub use statement::{HirArgument, HirAttribute, HirBlock, HirMatchArm, HirStatement, HirStatementKind};
pub use type_family::{HirTypeFamily, HirTypeFunction};
pub use visibility::HirVisibility;
pub use widget::{HirSingleton, HirWidget, HirWidgetLifecycle};
