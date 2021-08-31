use super::*;
use core::fmt::Debug;

impl Debug for StatementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nothing => f.write_str("Statement::Nothing"),
            Self::Document(node) => Debug::fmt(node, f),
            Self::Annotation(node) => Debug::fmt(node, f),
            Self::Namespace(node) => Debug::fmt(node, f),
            Self::Import(node) => Debug::fmt(node, f),
            Self::Class(node) => Debug::fmt(node, f),
            Self::Union(node) => Debug::fmt(node, f),
            Self::Enumerate(node) => Debug::fmt(node, f),
            Self::Trait(node) => Debug::fmt(node, f),
            Self::Extends(node) => Debug::fmt(node, f),
            Self::Function(node) => Debug::fmt(node, f),
            Self::Variable(node) => Debug::fmt(node, f),
            Self::Loop(node) => Debug::fmt(node, f),
            Self::While(node) => Debug::fmt(node, f),
            Self::Until(node) => Debug::fmt(node, f),
            Self::Each(node) => Debug::fmt(node, f),
            Self::Guard(node) => Debug::fmt(node, f),
            Self::Control(node) => Debug::fmt(node, f),
            Self::Expression(node) => Debug::fmt(&node.body, f),
        }
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for StatementKind {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        match self {
            Self::Nothing => ";;".into(),
            Self::Annotation(node) => node.pretty(theme),
            Self::Namespace(node) => node.pretty(theme),
            Self::Import(node) => node.pretty(theme),
            Self::Class(node) => node.pretty(theme),
            Self::Tagged(node) => node.pretty(theme),
            Self::Variant(node) => node.pretty(theme),
            Self::Union(node) => node.pretty(theme),
            Self::UnionField(node) => node.pretty(theme),
            Self::Enumerate(node) => node.pretty(theme),
            Self::EnumerateField(node) => node.pretty(theme),
            Self::Function(node) => node.pretty(theme),
            Self::While(node) => node.pretty(theme),
            Self::Each(node) => node.pretty(theme),
            Self::Expression(node) => node.pretty(theme),
            Self::Control(node) => node.pretty(theme),
            Self::Document(node) => node.pretty(theme),
            Self::Variable(node) => node.pretty(theme),
            Self::Guard(node) => node.pretty(theme),
            Self::Flags(node) => node.pretty(theme),
            Self::Trait(node) => node.pretty(theme),
            Self::Extends(node) => node.pretty(theme),
        }
    }
}

#[cfg(feature = "lispify")]
impl Lispify for StatementKind {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        match self {
            Self::Nothing => Lisp::default(),
            Self::Namespace(v) => v.lispify(),
            Self::Import(v) => todo!(),
            Self::While(v) => v.lispify(),
            Self::Each(v) => v.lispify(),
            Self::Class(v) => v.lispify(),
            Self::Expression(v) => v.lispify(),
            Self::Function(v) => v.lispify(),
            Self::Control(v) => todo!(),
            Self::Document(v) => todo!(),
            Self::Variable(v) => todo!(),
            Self::Guard(v) => v.lispify(),
            Self::Flags(v) => v.lispify(),
            Self::EnumerateField(v) => todo!(),
            Self::Tagged(v) => todo!(),
            Self::Variant(v) => todo!(),
            Self::Union(v) => v.lispify(),
            Self::Enumerate(v) => todo!(),
            Self::UnionField(v) => todo!(),
            Self::Annotation(v) => todo!(),
            Self::Trait(v) => v.lispify(),
            Self::Extends(v) => v.lispify(),
        }
    }
}
