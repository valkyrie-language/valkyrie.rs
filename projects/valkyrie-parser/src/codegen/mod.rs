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
    EOS,
    EOS_FREE,
    DefineNamespace,
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
    PatternItem,
    TuplePatternItem,
    WhileStatement,
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
    KW_MATCH,
    BIND_L,
    BIND_R,
    DotMatchCall,
    MainExpression,
    MainTerm,
    MainFactor,
    GroupFactor,
    Leading,
    MainSuffixTerm,
    MainPrefix,
    TypePrefix,
    MainInfix,
    TypeInfix,
    MainSuffix,
    TypeSuffix,
    InlineExpression,
    InlineTerm,
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
    DotCall,
    DotCallItem,
    DotClosureCall,
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
            Self::EOS => "",
            Self::EOS_FREE => "",
            Self::DefineNamespace => "",
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
            Self::PatternItem => "",
            Self::TuplePatternItem => "",
            Self::WhileStatement => "",
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
            Self::KW_MATCH => "",
            Self::BIND_L => "",
            Self::BIND_R => "",
            Self::DotMatchCall => "",
            Self::MainExpression => "",
            Self::MainTerm => "",
            Self::MainFactor => "",
            Self::GroupFactor => "",
            Self::Leading => "",
            Self::MainSuffixTerm => "",
            Self::MainPrefix => "",
            Self::TypePrefix => "",
            Self::MainInfix => "",
            Self::TypeInfix => "",
            Self::MainSuffix => "",
            Self::TypeSuffix => "",
            Self::InlineExpression => "",
            Self::InlineTerm => "",
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
            Self::DotCall => "",
            Self::DotCallItem => "",
            Self::DotClosureCall => "",
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
pub struct ProgramNode {
    pub shebang: Option<ShebangNode>,
    pub statement: Vec<StatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode {
    ControlFlow(ControlFlowNode),
    DefineClass(DefineClassNode),
    DefineEnumerate(DefineEnumerateNode),
    DefineExtends(DefineExtendsNode),
    DefineFunction(DefineFunctionNode),
    DefineImport(DefineImportNode),
    DefineNamespace(DefineNamespaceNode),
    DefineTrait(DefineTraitNode),
    DefineUnion(DefineUnionNode),
    DefineVariable(DefineVariableNode),
    Eos(EosNode),
    ExpressionRoot(ExpressionRootNode),
    ForStatement(ForStatementNode),
    WhileStatement(WhileStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EosNode {
    Omit,
    Show,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EosFreeNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineNamespaceNode {
    pub namepath_free: NamepathFreeNode,
    pub op_namespace: Option<OpNamespaceNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpNamespaceNode {
    Hide,
    Main,
    Test,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineImportNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub eos_free: Option<EosFreeNode>,
    pub import_block: Option<ImportBlockNode>,
    pub import_term: Option<ImportTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportBlockNode {
    pub import_term: Vec<ImportTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportTermNode {
    EosFree(EosFreeNode),
    ImportAll(ImportAllNode),
    ImportName(ImportNameNode),
    ImportSpace(ImportSpaceNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAllNode {
    pub path: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportSpaceNode {
    pub body: ImportBlockNode,
    pub path: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportNameNode {
    pub alias: ImportAsNode,
    pub item: ImportNameItemNode,
    pub path: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAsNode {
    pub alias: Option<ImportNameItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportNameItemNode {
    AttributeName(AttributeNameNode),
    Identifier(IdentifierNode),
    ProceduralName(ProceduralNameNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineConstraintNode {
    pub annotation_head: AnnotationHeadNode,
    pub constraint_block: ConstraintBlockNode,
    pub constraint_parameters: Option<ConstraintParametersNode>,
    pub kw_constraint: KwConstraintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintParametersNode {
    pub identifier: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintBlockNode {
    pub constraint_implements: Vec<ConstraintImplementsNode>,
    pub constraint_statement: Vec<ConstraintStatementNode>,
    pub eos_free: Vec<EosFreeNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintStatementNode {
    pub where_block: WhereBlockNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintImplementsNode {
    pub kw_implements: KwImplementsNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereBlockNode {
    pub kw_where: KwWhereNode,
    pub where_bound: Vec<WhereBoundNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereBoundNode {
    pub eos_free: EosFreeNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineClassNode {
    pub annotation_head: AnnotationHeadNode,
    pub class_block: ClassBlockNode,
    pub define_constraint: Option<DefineConstraintNode>,
    pub define_generic: Option<DefineGenericNode>,
    pub define_inherit: Option<DefineInheritNode>,
    pub identifier: IdentifierNode,
    pub kw_class: KwClassNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassBlockNode {
    pub class_term: Vec<ClassTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassTermNode {
    DefineDomain(DefineDomainNode),
    DefineField(DefineFieldNode),
    DefineMethod(DefineMethodNode),
    EosFree(EosFreeNode),
    ProceduralCall(ProceduralCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwClassNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineFieldNode {
    pub annotation_mix: AnnotationMixNode,
    pub identifier: IdentifierNode,
    pub parameter_default: ParameterDefaultNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterDefaultNode {
    pub main_expression: Option<MainExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineMethodNode {
    pub annotation_mix: AnnotationMixNode,
    pub continuation: Option<ContinuationNode>,
    pub function_middle: FunctionMiddleNode,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineDomainNode {
    pub annotation_mix: AnnotationMixNode,
    pub domain_term: DomainTermNode,
    pub statement: Vec<StatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DomainTermNode {
    Identifier(IdentifierNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineInheritNode {
    pub inherit_term: Vec<InheritTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InheritTermNode {
    pub annotation_mix: AnnotationMixNode,
    pub type_expression: TypeExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectStatementNode {
    pub class_block: ClassBlockNode,
    pub define_inherit: Option<DefineInheritNode>,
    pub kw_object: KwObjectNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineEnumerateNode {
    pub annotation_head: AnnotationHeadNode,
    pub define_inherit: Option<DefineInheritNode>,
    pub flag_term: Vec<FlagTermNode>,
    pub identifier: IdentifierNode,
    pub kw_flags: KwFlagsNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlagTermNode {
    DefineMethod(DefineMethodNode),
    EosFree(EosFreeNode),
    FlagField(FlagFieldNode),
    ProceduralCall(ProceduralCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlagFieldNode {
    pub identifier: IdentifierNode,
    pub parameter_default: ParameterDefaultNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwFlagsNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineUnionNode {
    pub annotation_head: AnnotationHeadNode,
    pub define_constraint: Option<DefineConstraintNode>,
    pub define_generic: Option<DefineGenericNode>,
    pub define_inherit: Option<DefineInheritNode>,
    pub identifier: IdentifierNode,
    pub kw_union: KwUnionNode,
    pub type_hint: TypeHintNode,
    pub union_term: Vec<UnionTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnionTermNode {
    DefineMethod(DefineMethodNode),
    DefineVariant(DefineVariantNode),
    EosFree(EosFreeNode),
    ProceduralCall(ProceduralCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineVariantNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub class_block: Option<ClassBlockNode>,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwUnionNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineTraitNode {
    pub annotation_head: AnnotationHeadNode,
    pub define_constraint: Option<DefineConstraintNode>,
    pub define_generic: Option<DefineGenericNode>,
    pub define_inherit: Option<DefineInheritNode>,
    pub identifier: IdentifierNode,
    pub kw_trait: KwTraitNode,
    pub trait_block: TraitBlockNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineExtendsNode {
    pub annotation_head: AnnotationHeadNode,
    pub define_constraint: Option<DefineConstraintNode>,
    pub kw_extends: KwExtendsNode,
    pub trait_block: TraitBlockNode,
    pub type_expression: TypeExpressionNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitBlockNode {
    pub trait_term: Vec<TraitTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TraitTermNode {
    DefineField(DefineFieldNode),
    DefineMethod(DefineMethodNode),
    EosFree(EosFreeNode),
    ProceduralCall(ProceduralCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwTraitNode {
    Interface,
    Trait,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineFunctionNode {
    pub annotation_head: AnnotationHeadNode,
    pub continuation: ContinuationNode,
    pub function_middle: FunctionMiddleNode,
    pub identifier: IdentifierNode,
    pub kw_function: KwFunctionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineLambdaNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub continuation: ContinuationNode,
    pub function_middle: FunctionMiddleNode,
    pub kw_lambda: KwLambdaNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionMiddleNode {
    pub define_generic: Option<DefineGenericNode>,
    pub function_parameters: FunctionParametersNode,
    pub type_effect: Option<TypeEffectNode>,
    pub type_return: Option<TypeReturnNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeHintNode {
    pub hint: Option<TypeExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeReturnNode {
    pub arrow_1: Arrow1Node,
    pub type_expression: TypeExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeEffectNode {
    pub type_expression: TypeExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionParametersNode {
    pub parameter_item: Vec<ParameterItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterItemNode {
    ParameterItemControl(ParameterItemControlNode),
    ParameterPair(ParameterPairNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterItemControlNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterPairNode {
    pub identifier: IdentifierNode,
    pub modifier_ahead: Vec<ModifierAheadNode>,
    pub parameter_default: ParameterDefaultNode,
    pub parameter_hint: Option<ParameterHintNode>,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterHintNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContinuationNode {
    pub statement: Vec<StatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwFunctionNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineVariableNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub kw_let: KwLetNode,
    pub let_pattern: LetPatternNode,
    pub parameter_default: ParameterDefaultNode,
    pub type_hint: TypeHintNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LetPatternNode {
    BarePattern(BarePatternNode),
    StandardPattern(StandardPatternNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StandardPatternNode {
    TuplePattern(TuplePatternNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BarePatternNode {
    pub bare_pattern_item: Vec<BarePatternItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BarePatternItemNode {
    pub identifier: IdentifierNode,
    pub modifier_ahead: Vec<ModifierAheadNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternNode {
    pub namepath: Option<NamepathNode>,
    pub pattern_item: Vec<PatternItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternItemNode {
    OmitDict,
    OmitList,
    TuplePatternItem(TuplePatternItemNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternItemNode {
    pub annotation_mix: AnnotationMixNode,
    pub colon: Option<ColonNode>,
    pub identifier: IdentifierNode,
    pub parameter_hint: Option<ParameterHintNode>,
    pub standard_pattern: Option<StandardPatternNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhileStatementNode {
    pub continuation: ContinuationNode,
    pub inline_expression: Option<InlineExpressionNode>,
    pub kw_while: KwWhileNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwWhileNode {
    Until,
    While,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForStatementNode {
    pub continuation: ContinuationNode,
    pub if_guard: IfGuardNode,
    pub inline_expression: Option<InlineExpressionNode>,
    pub kw_for: KwForNode,
    pub kw_in: KwInNode,
    pub let_pattern: LetPatternNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfGuardNode {
    pub condition: Option<InlineExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlFlowNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub jump_label: JumpLabelNode,
    pub kw_control: KwControlNode,
    pub main_expression: Option<MainExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpLabelNode {
    pub identifier: Option<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionRootNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub eos: Option<EosNode>,
    pub main_expression: MainExpressionNode,
    pub op_and_then: Option<OpAndThenNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchExpressionNode {
    pub bind_l: Option<BindLNode>,
    pub bind_r: Option<BindRNode>,
    pub identifier: Option<IdentifierNode>,
    pub inline_expression: InlineExpressionNode,
    pub kw_match: KwMatchNode,
    pub match_block: MatchBlockNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchStatementNode {
    pub kw_switch: KwSwitchNode,
    pub match_block: MatchBlockNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchBlockNode {
    pub match_terms: Vec<MatchTermsNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatchTermsNode {
    Comma(CommaNode),
    MatchCase(MatchCaseNode),
    MatchElse(MatchElseNode),
    MatchType(MatchTypeNode),
    MatchWhen(MatchWhenNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchTypeNode {
    pub if_guard: IfGuardNode,
    pub kw_type: KwTypeNode,
    pub match_statement: Vec<MatchStatementNode>,
    pub type_expression: TypeExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchCaseNode {
    pub case_pattern: CasePatternNode,
    pub if_guard: IfGuardNode,
    pub kw_case: KwCaseNode,
    pub match_statement: Vec<MatchStatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CasePatternNode {
    Namepath(NamepathNode),
    StandardPattern(StandardPatternNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchWhenNode {
    pub inline_expression: InlineExpressionNode,
    pub kw_when: KwWhenNode,
    pub match_statement: Vec<MatchStatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchElseNode {
    pub kw_else: KwElseNode,
    pub match_statement: Vec<MatchStatementNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchStatementNode {
    pub statement: StatementNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwMatchNode {
    Catch,
    Match,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindLNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BindRNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotMatchCallNode {
    pub bind_r: Option<BindRNode>,
    pub identifier: Option<IdentifierNode>,
    pub kw_match: KwMatchNode,
    pub match_block: MatchBlockNode,
    pub op_and_then: Option<OpAndThenNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainExpressionNode {
    pub main_infix: Vec<MainInfixNode>,
    pub main_term: Vec<MainTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainTermNode {
    pub main_factor: MainFactorNode,
    pub main_prefix: Vec<MainPrefixNode>,
    pub main_suffix_term: Vec<MainSuffixTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainFactorNode {
    DefineLambda(DefineLambdaNode),
    GroupFactor(GroupFactorNode),
    Leading(LeadingNode),
    MatchExpression(MatchExpressionNode),
    NewStatement(NewStatementNode),
    ObjectStatement(ObjectStatementNode),
    SwitchStatement(SwitchStatementNode),
    TryStatement(TryStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupFactorNode {
    pub main_expression: MainExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LeadingNode {
    Namepath(NamepathNode),
    Number(NumberNode),
    ProceduralCall(ProceduralCallNode),
    RangeLiteral(RangeLiteralNode),
    Slot(SlotNode),
    Special(SpecialNode),
    TextLiteral(TextLiteralNode),
    TupleLiteralStrict(TupleLiteralStrictNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainSuffixTermNode {
    DotClosureCall(DotClosureCallNode),
    DotMatchCall(DotMatchCallNode),
    InlineSuffixTerm(InlineSuffixTermNode),
    TupleCall(TupleCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainPrefixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypePrefixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainInfixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeInfixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainSuffixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeSuffixNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineExpressionNode {
    pub inline_term: Vec<InlineTermNode>,
    pub main_infix: Vec<MainInfixNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineTermNode {
    pub inline_suffix_term: Vec<InlineSuffixTermNode>,
    pub main_factor: MainFactorNode,
    pub main_prefix: Vec<MainPrefixNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InlineSuffixTermNode {
    DotCall(DotCallNode),
    GenericCall(GenericCallNode),
    InlineTupleCall(InlineTupleCallNode),
    MainSuffix(MainSuffixNode),
    RangeCall(RangeCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeExpressionNode {
    pub type_infix: Vec<TypeInfixNode>,
    pub type_term: Vec<TypeTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeTermNode {
    pub main_factor: MainFactorNode,
    pub type_prefix: Vec<TypePrefixNode>,
    pub type_suffix_term: Vec<TypeSuffixTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeFactorNode {
    Leading(LeadingNode),
    TypeExpression(TypeExpressionNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeSuffixTermNode {
    GenericHide(GenericHideNode),
    TypeSuffix(TypeSuffixNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TryStatementNode {
    pub continuation: ContinuationNode,
    pub kw_try: KwTryNode,
    pub type_expression: Option<TypeExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewStatementNode {
    pub generic_hide: Option<GenericHideNode>,
    pub kw_new: KwNewNode,
    pub modifier_ahead: Vec<ModifierAheadNode>,
    pub namepath: NamepathNode,
    pub new_block: Option<NewBlockNode>,
    pub tuple_literal: Option<TupleLiteralNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewBlockNode {
    pub eos_free: Vec<EosFreeNode>,
    pub new_pair: Vec<NewPairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewPairNode {
    pub colon: Option<ColonNode>,
    pub main_expression: MainExpressionNode,
    pub new_pair_key: Option<NewPairKeyNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NewPairKeyNode {
    Identifier(IdentifierNode),
    RangeLiteral(RangeLiteralNode),
    TextRaw(TextRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotCallNode {
    pub dot_call_item: DotCallItemNode,
    pub op_and_then: Option<OpAndThenNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DotCallItemNode {
    Integer(IntegerNode),
    Namepath(NamepathNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotClosureCallNode {
    pub continuation: ContinuationNode,
    pub op_and_then: Option<OpAndThenNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineTupleCallNode {
    pub op_and_then: Option<OpAndThenNode>,
    pub tuple_literal: TupleLiteralNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleCallNode {
    pub continuation: Option<ContinuationNode>,
    pub op_and_then: Option<OpAndThenNode>,
    pub tuple_literal: Option<TupleLiteralNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleLiteralNode {
    pub tuple_terms: TupleTermsNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleLiteralStrictNode {
    pub tuple_pair: Vec<TuplePairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleTermsNode {
    pub tuple_pair: Vec<TuplePairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePairNode {
    pub colon: Option<ColonNode>,
    pub main_expression: MainExpressionNode,
    pub tuple_key: Option<TupleKeyNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TupleKeyNode {
    Identifier(IdentifierNode),
    Integer(IntegerNode),
    TextRaw(TextRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeCallNode {
    pub op_and_then: Option<OpAndThenNode>,
    pub range_literal: RangeLiteralNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeLiteralNode {
    RangeLiteralIndex0(RangeLiteralIndex0Node),
    RangeLiteralIndex1(RangeLiteralIndex1Node),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeLiteralIndex0Node {
    pub subscript_axis: Vec<SubscriptAxisNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeLiteralIndex1Node {
    pub subscript_axis: Vec<SubscriptAxisNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubscriptAxisNode {
    SubscriptOnly(SubscriptOnlyNode),
    SubscriptRange(SubscriptRangeNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubscriptOnlyNode {
    pub index: MainExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubscriptRangeNode {
    pub head: Option<MainExpressionNode>,
    pub step: Option<MainExpressionNode>,
    pub tail: Option<MainExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeOmitNode {
    pub colon: Vec<ColonNode>,
    pub proportion: Option<ProportionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineGenericNode {
    pub generic_parameter: GenericParameterNode,
    pub proportion: Option<ProportionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParameterNode {
    pub generic_parameter_pair: Vec<GenericParameterPairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParameterPairNode {
    pub identifier: IdentifierNode,
    pub bound: Option<TypeExpressionNode>,
    pub default: Option<TypeExpressionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericCallNode {
    pub generic_terms: GenericTermsNode,
    pub namepath: Option<NamepathNode>,
    pub op_and_then: Option<OpAndThenNode>,
    pub proportion: Vec<ProportionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericHideNode {
    pub generic_terms: GenericTermsNode,
    pub proportion: Option<ProportionNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericTermsNode {
    pub generic_pair: Vec<GenericPairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericPairNode {
    pub colon: Option<ColonNode>,
    pub identifier: Option<IdentifierNode>,
    pub type_expression: TypeExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnnotationHeadNode {
    pub annotation_term: Vec<AnnotationTermNode>,
    pub modifier_call: Vec<ModifierCallNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnnotationMixNode {
    pub annotation_term_mix: Vec<AnnotationTermMixNode>,
    pub modifier_ahead: Vec<ModifierAheadNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationTermNode {
    AttributeCall(AttributeCallNode),
    AttributeList(AttributeListNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationTermMixNode {
    AttributeCall(AttributeCallNode),
    AttributeList(AttributeListNode),
    ProceduralCall(ProceduralCallNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeListNode {
    pub attribute_item: Vec<AttributeItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeCallNode {
    pub attribute_item: AttributeItemNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeItemNode {
    pub continuation: Option<ContinuationNode>,
    pub identifier: Vec<IdentifierNode>,
    pub namepath: NamepathNode,
    pub tuple_literal: Option<TupleLiteralNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeNameNode {
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralCallNode {
    pub continuation: Option<ContinuationNode>,
    pub namepath: NamepathNode,
    pub tuple_literal: Option<TupleLiteralNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralNameNode {
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLiteralNode {
    pub identifier: Option<IdentifierNode>,
    pub text_raw: TextRawNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextRawNode {
    pub text_content_1: Option<TextContent1Node>,
    pub text_content_2: Option<TextContent2Node>,
    pub text_content_3: Option<TextContent3Node>,
    pub text_content_4: Option<TextContent4Node>,
    pub text_content_5: Option<TextContent5Node>,
    pub text_content_6: Option<TextContent6Node>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextRNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextXNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent1Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent2Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent3Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent4Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent5Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextContent6Node {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierCallNode {
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierAheadNode {
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeywordsStopNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierStopNode {
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotNode {
    pub op_slot: OpSlotNode,
    pub slot_item: Option<SlotItemNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SlotItemNode {
    Identifier(IdentifierNode),
    Integer(IntegerNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamepathFreeNode {
    pub identifier: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamepathNode {
    pub identifier: Vec<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdentifierNode {
    IdentifierBare(IdentifierBareNode),
    IdentifierRaw(IdentifierRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierBareNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierRawNode {
    pub identifier_raw_text: IdentifierRawTextNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierRawTextNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpecialNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NumberNode {
    Decimal(DecimalNode),
    DecimalX(DecimalXNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignNode {
    Netative,
    Positive,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DigitsXNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecimalNode {
    pub dot: Option<DotNode>,
    pub lhs: IntegerNode,
    pub rhs: Option<IntegerNode>,
    pub shift: Option<IntegerNode>,
    pub sign: Option<SignNode>,
    pub unit: Option<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecimalXNode {
    pub base: IntegerNode,
    pub dot: Option<DotNode>,
    pub lhs: DigitsXNode,
    pub rhs: Option<DigitsXNode>,
    pub shift: Option<IntegerNode>,
    pub sign: Option<SignNode>,
    pub unit: Option<IdentifierNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProportionNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NsConcatNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColonNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arrow1Node {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommaNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpSlotNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetLNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetRNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proportion2Node {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpImportAllNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpAndThenNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpBindNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwControlNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNamespaceNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImportNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwConstraintNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhereNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwImplementsNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwExtendsNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInheritsNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwForNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEndNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLetNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNewNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwObjectNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLambdaNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIfNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwSwitchNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTryNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTypeNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwCaseNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhenNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwElseNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwNotNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwInNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIsNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwAsNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShebangNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhiteSpaceNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkipSpaceNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationsNode {
    pub string_interpolation_term: Vec<StringInterpolationTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringInterpolationTermNode {
    EscapeCharacter(EscapeCharacterNode),
    EscapeUnicode(EscapeUnicodeNode),
    StringInterpolationComplex(StringInterpolationComplexNode),
    StringInterpolationSimple(StringInterpolationSimpleNode),
    StringInterpolationText(StringInterpolationTextNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeCharacterNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeUnicodeNode {
    pub code: EscapeUnicodeCodeNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapeUnicodeCodeNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationSimpleNode {
    pub main_expression: MainExpressionNode,
    pub string_formatter: Option<StringFormatterNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationTextNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringFormatterNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringInterpolationComplexNode {
    pub main_expression: MainExpressionNode,
    pub tuple_pair: Vec<TuplePairNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringTemplatesNode {
    pub string_template_term: Vec<StringTemplateTermNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringTemplateTermNode {
    ExpressionTemplate(ExpressionTemplateNode),
    ForTemplate(ForTemplateNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionTemplateNode {
    pub main_expression: MainExpressionNode,
    pub template_e: TemplateENode,
    pub template_s: TemplateSNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateNode {
    pub for_template_begin: ForTemplateBeginNode,
    pub for_template_else: Option<ForTemplateElseNode>,
    pub for_template_end: ForTemplateEndNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateBeginNode {
    pub if_guard: IfGuardNode,
    pub inline_expression: Option<InlineExpressionNode>,
    pub kw_for: KwForNode,
    pub kw_in: KwInNode,
    pub let_pattern: LetPatternNode,
    pub template_e: TemplateENode,
    pub template_s: TemplateSNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateElseNode {
    pub kw_else: KwElseNode,
    pub template_e: TemplateENode,
    pub template_s: TemplateSNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForTemplateEndNode {
    pub kw_end: KwEndNode,
    pub kw_for: Option<KwForNode>,
    pub template_e: TemplateENode,
    pub template_s: TemplateSNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateSNode {
    pub template_m: Option<TemplateMNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateENode {
    pub template_m: Option<TemplateMNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateLNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateRNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateMNode {
    pub span: Range<u32>,
}
