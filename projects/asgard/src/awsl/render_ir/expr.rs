//! Render expression side-table records.

use super::ids::{RenderBindingId, RenderEventId, RenderExprId};

/// Expression record in canonical RenderIR.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderExpr {
    pub kind: RenderExprKind,
    pub source: String,
    pub binding_refs: Vec<RenderBindingId>,
    pub value_kind: RenderValueKind,
    pub purity: RenderPurity,
    pub memoizable: bool,
    pub event_call: Option<RenderEventShape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderExprKind {
    Template,
    BindingInit,
    EventHandler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderValueKind {
    Void,
    Bool,
    I32,
    Utf8,
    List,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPurity {
    Pure,
    Impure,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderEventShape {
    pub event_id: RenderEventId,
    pub arg_expr: Option<String>,
}

/// Script / template binding record.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderBinding {
    pub name: String,
    pub init_expr: RenderExprId,
    pub reactive: bool,
    pub sig_var: String,
}

/// Event handler record.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderEvent {
    pub name: String,
    pub handler_expr: RenderExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDiagnostic {
    pub message: String,
    pub span: Option<std::ops::Range<usize>>,
}
