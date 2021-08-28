#![allow(dead_code, unused_imports, non_camel_case_types)]
#![allow(missing_docs, rustdoc::missing_crate_level_docs)]
#![allow(clippy::unnecessary_cast)]
#![doc = include_str!("readme.md")]

mod parse_ast;
mod parse_cst;

use core::str::FromStr;
use std::{borrow::Cow, ops::Range, sync::OnceLock};
use yggdrasil_rt::*;

type Input<'i> = Box<State<'i, Rule>>;
type Output<'i> = Result<Box<State<'i, Rule>>, Box<State<'i, Rule>>>;

#[doc = include_str!("railway.min.svg")]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parser {}

impl YggdrasilParser for Parser {
    type Rule = Rule;
    fn parse_cst(input: &str, rule: Self::Rule) -> OutputResult<Rule> {
        self::parse_cst::parse_cst(input, rule)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rule {
    BIND_L,
    BIND_R,
    PROPORTION,
    NS_CONCAT,
    COLON,
    ARROW1,
    COMMA,
    DOT,
    OP_SLOT,
    OFFSET_L,
    OFFSET_R,
    OP_IMPORT_ALL,
    OP_AND_THEN,
    OP_BIND,
    KW_NAMESPACE,
    KW_IMPORT,
    KW_CONSTRAINT,
    KW_WHERE,
    KW_IMPLEMENTS,
    KW_EXTENDS,
    KW_INHERITS,
    KW_FOR,
    KW_END,
    KW_LET,
    KW_NEW,
    KW_OBJECT,
    KW_LAMBDA,
    KW_IF,
    KW_SWITCH,
    KW_TRY,
    KW_TYPE,
    KW_CASE,
    KW_WHEN,
    KW_ELSE,
    KW_NOT,
    KW_IN,
    KW_IS,
    KW_AS,
    TEMPLATE_L,
    TEMPLATE_R,
    TEMPLATE_M,
    /// Label for unnamed text literal
    HiddenText,
}

impl YggdrasilRule for Rule {
    fn is_ignore(&self) -> bool {
        matches!(self, Self::HiddenText)
    }

    fn get_style(&self) -> &'static str {
        match self {
            Self::BIND_L => "",
            Self::BIND_R => "",
            Self::PROPORTION => "",
            Self::NS_CONCAT => "",
            Self::COLON => "",
            Self::ARROW1 => "",
            Self::COMMA => "",
            Self::DOT => "",
            Self::OP_SLOT => "",
            Self::OFFSET_L => "",
            Self::OFFSET_R => "",
            Self::OP_IMPORT_ALL => "",
            Self::OP_AND_THEN => "",
            Self::OP_BIND => "",
            Self::KW_NAMESPACE => "",
            Self::KW_IMPORT => "",
            Self::KW_CONSTRAINT => "",
            Self::KW_WHERE => "",
            Self::KW_IMPLEMENTS => "",
            Self::KW_EXTENDS => "",
            Self::KW_INHERITS => "",
            Self::KW_FOR => "",
            Self::KW_END => "",
            Self::KW_LET => "",
            Self::KW_NEW => "",
            Self::KW_OBJECT => "",
            Self::KW_LAMBDA => "",
            Self::KW_IF => "",
            Self::KW_SWITCH => "",
            Self::KW_TRY => "",
            Self::KW_TYPE => "",
            Self::KW_CASE => "",
            Self::KW_WHEN => "",
            Self::KW_ELSE => "",
            Self::KW_NOT => "",
            Self::KW_IN => "",
            Self::KW_IS => "",
            Self::KW_AS => "",
            Self::TEMPLATE_L => "",
            Self::TEMPLATE_R => "",
            Self::TEMPLATE_M => "",
            _ => "",
        }
    }
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindLNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindRNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProportionNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NsConcatNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColonNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arrow1Node<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommaNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpSlotNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetLNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetRNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpImportAllNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpAndThenNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpBindNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNamespaceNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImportNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwConstraintNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhereNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImplementsNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwExtendsNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInheritsNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwForNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEndNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLetNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNewNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwObjectNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLambdaNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIfNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwSwitchNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTryNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTypeNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwCaseNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhenNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwElseNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNotNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIsNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwAsNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateLNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateRNode<'i> {
    pair: TokenPair<'i, Rule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateMNode<'i> {
    pair: TokenPair<'i, Rule>,
}
