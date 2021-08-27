#![allow(dead_code, unused_imports, non_camel_case_types)]
#![allow(missing_docs, rustdoc::missing_crate_level_docs)]
#![allow(clippy::unnecessary_cast)]
#![doc = include_str!("readme.md")]

mod parse_ast;
mod parse_cst;

use core::str::FromStr;
use std::{borrow::Cow, ops::Range, sync::OnceLock};
use yggdrasil_rt::*;

type Input<'i> = Box<State<'i, ValkyrieRule>>;
type Output<'i> = Result<Box<State<'i, ValkyrieRule>>, Box<State<'i, ValkyrieRule>>>;

#[doc = include_str!("railway.min.svg")]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValkyrieParser {}

impl YggdrasilParser for ValkyrieParser {
    type Rule = ValkyrieRule;
    fn parse_cst(input: &str, rule: Self::Rule) -> OutputResult<ValkyrieRule> {
        self::parse_cst::parse_cst(input, rule)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValkyrieRule {
    Program,
    Statement,
    Omit,
    Show,
    EOS,
    EOS_FREE,
    DefineNamespace,
    Main,
    Test,
    Hide,
    OP_NAMESPACE,
    DefineImport,
    ImportBlock,
    ImportTerm,
    ImportAll,
    ImportSpace,
    ImportName,
    ImportAs,
    ImportNameItem,
    DefineConstraint,
    ConstraintParameters,
    ConstraintBlock,
    ConstraintStatement,
    ConstraintImplements,
    WhereBlock,
    WhereBound,
    DefineClass,
    ClassBlock,
    ClassTerm,
    KW_CLASS,
    DefineField,
    ParameterDefault,
    DefineMethod,
    DefineDomain,
    DomainTerm,
    DefineInherit,
    InheritTerm,
    ObjectStatement,
    DefineEnumerate,
    FlagTerm,
    FlagField,
    KW_FLAGS,
    DefineUnion,
    UnionTerm,
    DefineVariant,
    KW_UNION,
    DefineTrait,
    DefineExtends,
    TraitBlock,
    TraitTerm,
    Trait,
    Interface,
    KW_TRAIT,
    DefineFunction,
    DefineLambda,
    FunctionMiddle,
    TypeHint,
    TypeReturn,
    TypeEffect,
    FunctionParameters,
    ParameterItem,
    ParameterItemControl,
    ParameterPair,
    ParameterHint,
    Continuation,
    KW_FUNCTION,
    DefineVariable,
    LetPattern,
    StandardPattern,
    BarePattern,
    BarePatternItem,
    TuplePattern,
    OmitDict,
    OmitList,
    PatternItem,
    TuplePatternItem,
    WhileStatement,
    While,
    Until,
    KW_WHILE,
    ForStatement,
    IfGuard,
    ControlFlow,
    JumpLabel,
    ExpressionRoot,
    MatchExpression,
    SwitchStatement,
    MatchBlock,
    MatchTerms,
    MatchType,
    MatchCase,
    CasePattern,
    MatchWhen,
    MatchElse,
    MatchStatement,
    Match,
    Catch,
    KW_MATCH,
    BIND_L,
    BIND_R,
    DotMatchCall,
    MainExpression,
    MainTerm,
    MainFactor,
    GroupFactor,
    Leading,
    DotClosureCall,
    MainSuffixTerm,
    MainPrefix,
    TypePrefix,
    MainInfix,
    TypeInfix,
    MainSuffix,
    TypeSuffix,
    InlineExpression,
    InlineTerm,
    DotCall,
    InlineSuffixTerm,
    TypeExpression,
    TypeTerm,
    TypeFactor,
    TypeSuffixTerm,
    TryStatement,
    NewStatement,
    NewBlock,
    NewPair,
    NewPairKey,
    DotCallItem,
    InlineTupleCall,
    TupleCall,
    TupleLiteral,
    TupleLiteralStrict,
    TupleTerms,
    TuplePair,
    TupleKey,
    RangeCall,
    RangeLiteral,
    RangeLiteralIndex0,
    RangeLiteralIndex1,
    SubscriptAxis,
    SubscriptOnly,
    SubscriptRange,
    RangeOmit,
    DefineGeneric,
    GenericParameter,
    GenericParameterPair,
    GenericCall,
    GenericHide,
    GenericTerms,
    GenericPair,
    AnnotationHead,
    AnnotationMix,
    AnnotationTerm,
    AnnotationTermMix,
    AttributeList,
    AttributeCall,
    AttributeItem,
    AttributeName,
    ProceduralCall,
    ProceduralName,
    TextLiteral,
    TextRaw,
    Text_L,
    Text_R,
    Text_X,
    TEXT_CONTENT1,
    TEXT_CONTENT2,
    TEXT_CONTENT3,
    TEXT_CONTENT4,
    TEXT_CONTENT5,
    TEXT_CONTENT6,
    ModifierCall,
    ModifierAhead,
    KEYWORDS_STOP,
    IDENTIFIER_STOP,
    Slot,
    SlotItem,
    NamepathFree,
    Namepath,
    Identifier,
    IdentifierBare,
    IdentifierRaw,
    IdentifierRawText,
    Special,
    Number,
    Positive,
    Netative,
    Sign,
    Integer,
    DigitsX,
    Decimal,
    DecimalX,
    PROPORTION,
    NS_CONCAT,
    COLON,
    ARROW1,
    COMMA,
    DOT,
    OP_SLOT,
    OFFSET_L,
    OFFSET_R,
    PROPORTION2,
    OP_IMPORT_ALL,
    OP_AND_THEN,
    OP_BIND,
    KW_CONTROL,
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
    Shebang,
    WhiteSpace,
    SkipSpace,
    Comment,
    StringInterpolations,
    StringInterpolationTerm,
    EscapeCharacter,
    EscapeUnicode,
    EscapeUnicodeCode,
    StringInterpolationSimple,
    StringInterpolationText,
    StringFormatter,
    StringInterpolationComplex,
    StringTemplates,
    StringTemplateTerm,
    ExpressionTemplate,
    ForTemplate,
    ForTemplateBegin,
    ForTemplateElse,
    ForTemplateEnd,
    TEMPLATE_S,
    TEMPLATE_E,
    TEMPLATE_L,
    TEMPLATE_R,
    TEMPLATE_M,
    /// Label for unnamed text literal
    HiddenText,
}

impl YggdrasilRule for ValkyrieRule {
    fn is_ignore(&self) -> bool {
        matches!(self, Self::HiddenText | Self::SkipSpace | Self::Comment)
    }

    fn get_style(&self) -> &'static str {
        match self {
            Self::Program => "",
            Self::Statement => "",
            Self::Omit => "",
            Self::Show => "",
            Self::EOS => "",
            Self::EOS_FREE => "",
            Self::DefineNamespace => "",
            Self::Main => "",
            Self::Test => "",
            Self::Hide => "",
            Self::OP_NAMESPACE => "",
            Self::DefineImport => "",
            Self::ImportBlock => "",
            Self::ImportTerm => "",
            Self::ImportAll => "",
            Self::ImportSpace => "",
            Self::ImportName => "",
            Self::ImportAs => "",
            Self::ImportNameItem => "",
            Self::DefineConstraint => "",
            Self::ConstraintParameters => "",
            Self::ConstraintBlock => "",
            Self::ConstraintStatement => "",
            Self::ConstraintImplements => "",
            Self::WhereBlock => "",
            Self::WhereBound => "",
            Self::DefineClass => "",
            Self::ClassBlock => "",
            Self::ClassTerm => "",
            Self::KW_CLASS => "",
            Self::DefineField => "",
            Self::ParameterDefault => "",
            Self::DefineMethod => "",
            Self::DefineDomain => "",
            Self::DomainTerm => "",
            Self::DefineInherit => "",
            Self::InheritTerm => "",
            Self::ObjectStatement => "",
            Self::DefineEnumerate => "",
            Self::FlagTerm => "",
            Self::FlagField => "",
            Self::KW_FLAGS => "",
            Self::DefineUnion => "",
            Self::UnionTerm => "",
            Self::DefineVariant => "",
            Self::KW_UNION => "",
            Self::DefineTrait => "",
            Self::DefineExtends => "",
            Self::TraitBlock => "",
            Self::TraitTerm => "",
            Self::Trait => "",
            Self::Interface => "",
            Self::KW_TRAIT => "",
            Self::DefineFunction => "",
            Self::DefineLambda => "",
            Self::FunctionMiddle => "",
            Self::TypeHint => "",
            Self::TypeReturn => "",
            Self::TypeEffect => "",
            Self::FunctionParameters => "",
            Self::ParameterItem => "",
            Self::ParameterItemControl => "",
            Self::ParameterPair => "",
            Self::ParameterHint => "",
            Self::Continuation => "",
            Self::KW_FUNCTION => "",
            Self::DefineVariable => "",
            Self::LetPattern => "",
            Self::StandardPattern => "",
            Self::BarePattern => "",
            Self::BarePatternItem => "",
            Self::TuplePattern => "",
            Self::OmitDict => "",
            Self::OmitList => "",
            Self::PatternItem => "",
            Self::TuplePatternItem => "",
            Self::WhileStatement => "",
            Self::While => "",
            Self::Until => "",
            Self::KW_WHILE => "",
            Self::ForStatement => "",
            Self::IfGuard => "",
            Self::ControlFlow => "",
            Self::JumpLabel => "",
            Self::ExpressionRoot => "",
            Self::MatchExpression => "",
            Self::SwitchStatement => "",
            Self::MatchBlock => "",
            Self::MatchTerms => "",
            Self::MatchType => "",
            Self::MatchCase => "",
            Self::CasePattern => "",
            Self::MatchWhen => "",
            Self::MatchElse => "",
            Self::MatchStatement => "",
            Self::Match => "",
            Self::Catch => "",
            Self::KW_MATCH => "",
            Self::BIND_L => "",
            Self::BIND_R => "",
            Self::DotMatchCall => "",
            Self::MainExpression => "",
            Self::MainTerm => "",
            Self::MainFactor => "",
            Self::GroupFactor => "",
            Self::Leading => "",
            Self::DotClosureCall => "",
            Self::MainSuffixTerm => "",
            Self::MainPrefix => "",
            Self::TypePrefix => "",
            Self::MainInfix => "",
            Self::TypeInfix => "",
            Self::MainSuffix => "",
            Self::TypeSuffix => "",
            Self::InlineExpression => "",
            Self::InlineTerm => "",
            Self::DotCall => "",
            Self::InlineSuffixTerm => "",
            Self::TypeExpression => "",
            Self::TypeTerm => "",
            Self::TypeFactor => "",
            Self::TypeSuffixTerm => "",
            Self::TryStatement => "",
            Self::NewStatement => "",
            Self::NewBlock => "",
            Self::NewPair => "",
            Self::NewPairKey => "",
            Self::DotCallItem => "",
            Self::InlineTupleCall => "",
            Self::TupleCall => "",
            Self::TupleLiteral => "",
            Self::TupleLiteralStrict => "",
            Self::TupleTerms => "",
            Self::TuplePair => "",
            Self::TupleKey => "",
            Self::RangeCall => "",
            Self::RangeLiteral => "",
            Self::RangeLiteralIndex0 => "",
            Self::RangeLiteralIndex1 => "",
            Self::SubscriptAxis => "",
            Self::SubscriptOnly => "",
            Self::SubscriptRange => "",
            Self::RangeOmit => "",
            Self::DefineGeneric => "",
            Self::GenericParameter => "",
            Self::GenericParameterPair => "",
            Self::GenericCall => "",
            Self::GenericHide => "",
            Self::GenericTerms => "",
            Self::GenericPair => "",
            Self::AnnotationHead => "",
            Self::AnnotationMix => "",
            Self::AnnotationTerm => "",
            Self::AnnotationTermMix => "",
            Self::AttributeList => "",
            Self::AttributeCall => "",
            Self::AttributeItem => "",
            Self::AttributeName => "",
            Self::ProceduralCall => "",
            Self::ProceduralName => "",
            Self::TextLiteral => "",
            Self::TextRaw => "",
            Self::Text_L => "",
            Self::Text_R => "",
            Self::Text_X => "",
            Self::TEXT_CONTENT1 => "",
            Self::TEXT_CONTENT2 => "",
            Self::TEXT_CONTENT3 => "",
            Self::TEXT_CONTENT4 => "",
            Self::TEXT_CONTENT5 => "",
            Self::TEXT_CONTENT6 => "",
            Self::ModifierCall => "",
            Self::ModifierAhead => "",
            Self::KEYWORDS_STOP => "",
            Self::IDENTIFIER_STOP => "",
            Self::Slot => "",
            Self::SlotItem => "",
            Self::NamepathFree => "",
            Self::Namepath => "",
            Self::Identifier => "",
            Self::IdentifierBare => "",
            Self::IdentifierRaw => "",
            Self::IdentifierRawText => "",
            Self::Special => "",
            Self::Number => "",
            Self::Positive => "",
            Self::Netative => "",
            Self::Sign => "",
            Self::Integer => "",
            Self::DigitsX => "",
            Self::Decimal => "",
            Self::DecimalX => "",
            Self::PROPORTION => "",
            Self::NS_CONCAT => "",
            Self::COLON => "",
            Self::ARROW1 => "",
            Self::COMMA => "",
            Self::DOT => "",
            Self::OP_SLOT => "",
            Self::OFFSET_L => "",
            Self::OFFSET_R => "",
            Self::PROPORTION2 => "",
            Self::OP_IMPORT_ALL => "",
            Self::OP_AND_THEN => "",
            Self::OP_BIND => "",
            Self::KW_CONTROL => "",
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
            Self::Shebang => "",
            Self::WhiteSpace => "",
            Self::SkipSpace => "",
            Self::Comment => "",
            Self::StringInterpolations => "",
            Self::StringInterpolationTerm => "",
            Self::EscapeCharacter => "",
            Self::EscapeUnicode => "",
            Self::EscapeUnicodeCode => "",
            Self::StringInterpolationSimple => "",
            Self::StringInterpolationText => "",
            Self::StringFormatter => "",
            Self::StringInterpolationComplex => "",
            Self::StringTemplates => "",
            Self::StringTemplateTerm => "",
            Self::ExpressionTemplate => "",
            Self::ForTemplate => "",
            Self::ForTemplateBegin => "",
            Self::ForTemplateElse => "",
            Self::ForTemplateEnd => "",
            Self::TEMPLATE_S => "",
            Self::TEMPLATE_E => "",
            Self::TEMPLATE_L => "",
            Self::TEMPLATE_R => "",
            Self::TEMPLATE_M => "",
            _ => "",
        }
    }
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode<'i> {
    DefineNamespace(DefineNamespaceNode<'i>),
    DefineClass(DefineClassNode<'i>),
    DefineUnion(DefineUnionNode<'i>),
    DefineEnumerate(DefineEnumerateNode<'i>),
    DefineTrait(DefineTraitNode<'i>),
    DefineExtends(DefineExtendsNode<'i>),
    DefineFunction(DefineFunctionNode<'i>),
    DefineVariable(DefineVariableNode<'i>),
    DefineImport(DefineImportNode<'i>),
    ControlFlow(ControlFlowNode<'i>),
    WhileStatement(WhileStatementNode<'i>),
    ForStatement(ForStatementNode<'i>),
    ExpressionRoot(ExpressionRootNode<'i>),
    EOS(EosNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OmitNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShowNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EosNode<'i> {
    Omit(OmitNode<'i>),
    Show(ShowNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EosFreeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineNamespaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HideNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpNamespaceNode<'i> {
    Main(MainNode<'i>),
    Test(TestNode<'i>),
    Hide(HideNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineImportNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportTermNode<'i> {
    ImportAll(ImportAllNode<'i>),
    ImportSpace(ImportSpaceNode<'i>),
    ImportName(ImportNameNode<'i>),
    EOS_FREE(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAllNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportSpaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportNameNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportNameItemNode<'i> {
    ProceduralName(ProceduralNameNode<'i>),
    AttributeName(AttributeNameNode<'i>),
    Identifier(IdentifierNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineConstraintNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintParametersNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintImplementsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereBoundNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineClassNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassTermNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    DefineMethod(DefineMethodNode<'i>),
    DefineDomain(DefineDomainNode<'i>),
    DefineField(DefineFieldNode<'i>),
    EOS_FREE(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwClassNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineFieldNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterDefaultNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineMethodNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineDomainNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DomainTermNode<'i> {
    Identifier(IdentifierNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineInheritNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InheritTermNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineEnumerateNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlagTermNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    DefineMethod(DefineMethodNode<'i>),
    FlagField(FlagFieldNode<'i>),
    EOS_FREE(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlagFieldNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwFlagsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineUnionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnionTermNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    DefineMethod(DefineMethodNode<'i>),
    DefineVariant(DefineVariantNode<'i>),
    EOS_FREE(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineVariantNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwUnionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineTraitNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineExtendsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TraitTermNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    DefineMethod(DefineMethodNode<'i>),
    DefineField(DefineFieldNode<'i>),
    EOS_FREE(EosNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterfaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwTraitNode<'i> {
    Trait(TraitNode<'i>),
    Interface(InterfaceNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineFunctionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineLambdaNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionMiddleNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeHintNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeReturnNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeEffectNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionParametersNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterItemNode<'i> {
    ParameterItemControl(ParameterItemControlNode<'i>),
    ParameterPair(ParameterPairNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterItemControlNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterPairNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterHintNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContinuationNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwFunctionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineVariableNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LetPatternNode<'i> {
    StandardPattern(StandardPatternNode<'i>),
    BarePattern(BarePatternNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StandardPatternNode<'i> {
    TuplePattern(TuplePatternNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BarePatternNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BarePatternItemNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OmitDictNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OmitListNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternItemNode<'i> {
    TuplePatternItem(TuplePatternItemNode<'i>),
    OmitDict(OmitDictNode<'i>),
    OmitList(OmitListNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternItemNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhileStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhileNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UntilNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwWhileNode<'i> {
    While(WhileNode<'i>),
    Until(UntilNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfGuardNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlFlowNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpLabelNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionRootNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchExpressionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatchTermsNode<'i> {
    MatchType(MatchTypeNode<'i>),
    MatchCase(MatchCaseNode<'i>),
    MatchWhen(MatchWhenNode<'i>),
    MatchElse(MatchElseNode<'i>),
    COMMA(CommaNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchTypeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchCaseNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CasePatternNode<'i> {
    StandardPattern(StandardPatternNode<'i>),
    Namepath(NamepathNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchWhenNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchElseNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CatchNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwMatchNode<'i> {
    Match(MatchNode<'i>),
    Catch(CatchNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindLNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindRNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotMatchCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainExpressionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainTermNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainFactorNode<'i> {
    SwitchStatement(SwitchStatementNode<'i>),
    TryStatement(TryStatementNode<'i>),
    MatchExpression(MatchExpressionNode<'i>),
    DefineLambda(DefineLambdaNode<'i>),
    ObjectStatement(ObjectStatementNode<'i>),
    NewStatement(NewStatementNode<'i>),
    GroupFactor(GroupFactorNode<'i>),
    Leading(LeadingNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupFactorNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LeadingNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    TupleLiteralStrict(TupleLiteralStrictNode<'i>),
    RangeLiteral(RangeLiteralNode<'i>),
    TextLiteral(TextLiteralNode<'i>),
    Slot(SlotNode<'i>),
    Number(NumberNode<'i>),
    Special(SpecialNode<'i>),
    Namepath(NamepathNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotClosureCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainSuffixTermNode<'i> {
    DotMatchCall(DotMatchCallNode<'i>),
    DotClosureCall(DotClosureCallNode<'i>),
    TupleCall(TupleCallNode<'i>),
    InlineSuffixTerm(InlineSuffixTermNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainPrefixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypePrefixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainInfixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeInfixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainSuffixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeSuffixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineExpressionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineTermNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InlineSuffixTermNode<'i> {
    MainSuffix(MainSuffixNode<'i>),
    DotCall(DotCallNode<'i>),
    InlineTupleCall(InlineTupleCallNode<'i>),
    RangeCall(RangeCallNode<'i>),
    GenericCall(GenericCallNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeExpressionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeTermNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeFactorNode<'i> {
    TypeExpression(TypeExpressionNode<'i>),
    Leading(LeadingNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeSuffixTermNode<'i> {
    GenericHide(GenericHideNode<'i>),
    TypeSuffix(TypeSuffixNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TryStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewPairNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NewPairKeyNode<'i> {
    Identifier(IdentifierNode<'i>),
    TextRaw(TextRawNode<'i>),
    RangeLiteral(RangeLiteralNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DotCallItemNode<'i> {
    Namepath(NamepathNode<'i>),
    Integer(IntegerNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineTupleCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleLiteralNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleLiteralStrictNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleTermsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePairNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TupleKeyNode<'i> {
    Identifier(IdentifierNode<'i>),
    Integer(IntegerNode<'i>),
    TextRaw(TextRawNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeLiteralNode<'i> {
    RangeLiteralIndex0(RangeLiteralIndex0Node<'i>),
    RangeLiteralIndex1(RangeLiteralIndex1Node<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeLiteralIndex0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeLiteralIndex1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubscriptAxisNode<'i> {
    SubscriptRange(SubscriptRangeNode<'i>),
    SubscriptOnly(SubscriptOnlyNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubscriptOnlyNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubscriptRangeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeOmitNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineGenericNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParameterNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParameterPairNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericHideNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericTermsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericPairNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnnotationHeadNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnnotationMixNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationTermNode<'i> {
    AttributeList(AttributeListNode<'i>),
    AttributeCall(AttributeCallNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationTermMixNode<'i> {
    AttributeList(AttributeListNode<'i>),
    AttributeCall(AttributeCallNode<'i>),
    ProceduralCall(ProceduralCallNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeListNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeItemNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeNameNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralNameNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLiteralNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextRawNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextRNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextXNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent2Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent3Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent4Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent5Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent6Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierAheadNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeywordsStopNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierStopNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SlotItemNode<'i> {
    Integer(IntegerNode<'i>),
    Identifier(IdentifierNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamepathFreeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamepathNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdentifierNode<'i> {
    IdentifierBare(IdentifierBareNode<'i>),
    IdentifierRaw(IdentifierRawNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierBareNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierRawNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierRawTextNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpecialNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NumberNode<'i> {
    DecimalX(DecimalXNode<'i>),
    Decimal(DecimalNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositiveNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NetativeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignNode<'i> {
    Positive(PositiveNode<'i>),
    Netative(NetativeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DigitsXNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecimalNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecimalXNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProportionNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NsConcatNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColonNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arrow1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommaNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpSlotNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetLNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetRNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proportion2Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpImportAllNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpAndThenNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpBindNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwControlNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNamespaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImportNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwConstraintNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhereNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImplementsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwExtendsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInheritsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwForNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEndNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLetNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNewNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwObjectNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLambdaNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIfNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwSwitchNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTryNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTypeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwCaseNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhenNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwElseNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNotNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwAsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShebangNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhiteSpaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkipSpaceNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringInterpolationTermNode<'i> {
    EscapeUnicode(EscapeUnicodeNode<'i>),
    EscapeCharacter(EscapeCharacterNode<'i>),
    StringInterpolationSimple(StringInterpolationSimpleNode<'i>),
    StringInterpolationComplex(StringInterpolationComplexNode<'i>),
    StringInterpolationText(StringInterpolationTextNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeCharacterNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeUnicodeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeUnicodeCodeNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationSimpleNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationTextNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringFormatterNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationComplexNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringTemplatesNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringTemplateTermNode<'i> {
    ForTemplate(ForTemplateNode<'i>),
    ExpressionTemplate(ExpressionTemplateNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionTemplateNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateBeginNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateElseNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateEndNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateSNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateENode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateLNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateRNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateMNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
