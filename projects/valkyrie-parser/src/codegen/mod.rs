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
    ARROW1,
    ANNOTATION_HEAD,
    ANNOTATION_MIX,
    ANNOTATION_TERM,
    ANNOTATION_TERM_MIX,
    ATTRIBUTE_BELOW_CALL,
    ATTRIBUTE_BELOW_MARK,
    ATTRIBUTE_ITEM,
    ATTRIBUTE_NAME,
    BIND_L,
    BIND_R,
    BARE_PATTERN,
    BARE_PATTERN_ITEM,
    COLON,
    COMMA,
    CASE_PATTERN,
    CLASS_BLOCK,
    CLASS_TERM,
    COMMENT,
    CONSTRAINT_BLOCK,
    CONSTRAINT_IMPLEMENTS,
    CONSTRAINT_PARAMETERS,
    CONSTRAINT_STATEMENT,
    CONTINUATION,
    CONTROL_FLOW,
    DOT,
    DECIMAL,
    DECIMAL_X,
    DEFINE_CLASS,
    DEFINE_CONSTRAINT,
    DEFINE_DOMAIN,
    DEFINE_ENUMERATE,
    DEFINE_EXTENDS,
    DEFINE_FIELD,
    DEFINE_FUNCTION,
    DEFINE_GENERIC,
    DEFINE_IMPORT,
    DEFINE_INHERIT,
    DEFINE_LAMBDA,
    DEFINE_METHOD,
    DEFINE_NAMESPACE,
    DEFINE_TRAIT,
    DEFINE_UNION,
    DEFINE_VARIABLE,
    DEFINE_VARIANT,
    DIGITS_X,
    DOMAIN_TERM,
    DOT_CALL,
    DOT_CALL_ITEM,
    DOT_CLOSURE_CALL,
    DOT_MATCH_CALL,
    EOS,
    EOS_FREE,
    EOS0,
    EOS1,
    ESCAPE_CHARACTER,
    ESCAPE_UNICODE,
    ESCAPE_UNICODE_CODE,
    EXPRESSION_ROOT,
    EXPRESSION_TEMPLATE,
    FLAG_FIELD,
    FLAG_TERM,
    FOR_TEMPLATE,
    FOR_TEMPLATE_BEGIN,
    FOR_TEMPLATE_ELSE,
    FOR_TEMPLATE_END,
    FUNCTION_MIDDLE,
    FUNCTION_PARAMETERS,
    GENERIC_CALL,
    GENERIC_HIDE,
    GENERIC_PAIR,
    GENERIC_PARAMETER,
    GENERIC_PARAMETER_PAIR,
    GENERIC_TERMS,
    GROUP_FACTOR,
    IDENTIFIER_STOP,
    IDENTIFIER,
    IDENTIFIER_BARE,
    IDENTIFIER_RAW,
    IDENTIFIER_RAW_TEXT,
    IF_GUARD,
    IMPORT_ALL,
    IMPORT_AS,
    IMPORT_BLOCK,
    IMPORT_NAME,
    IMPORT_NAME_ITEM,
    IMPORT_SPACE,
    IMPORT_TERM,
    INHERIT_TERM,
    INLINE_EXPRESSION,
    INLINE_SUFFIX_TERM,
    INLINE_SUFFIX_TERM0,
    INLINE_SUFFIX_TERM1,
    INLINE_TERM,
    INLINE_TUPLE_CALL,
    INTEGER,
    JUMP_LABEL,
    KEYWORDS_STOP,
    KW_AS,
    KW_CASE,
    KW_CLASS,
    KW_CONSTRAINT,
    KW_CONTROL,
    KW_EACH,
    KW_ELSE,
    KW_END,
    KW_ENUMERATE,
    KW_EXTENDS,
    KW_FLAGS,
    KW_FUNCTION,
    KW_IF,
    KW_IMPLEMENTS,
    KW_IMPORT,
    KW_IN,
    KW_INHERITS,
    KW_IS,
    KW_LAMBDA,
    KW_LET,
    KW_LOOP,
    KW_MATCH,
    KW_NAMESPACE,
    KW_NEW,
    KW_NOT,
    KW_OBJECT,
    KW_SWITCH,
    KW_TRAIT,
    KW_TRY,
    KW_TYPE,
    KW_UNION,
    KW_UNTIL,
    KW_WHEN,
    KW_WHERE,
    KW_WHILE,
    KW_MATCH0,
    KW_MATCH1,
    KW_TRAIT0,
    KW_TRAIT1,
    LEADING,
    LET_PATTERN,
    LOOP_EACH_STATEMENT,
    LOOP_STATEMENT,
    LOOP_UNTIL_STATEMENT,
    LOOP_WHILE_STATEMENT,
    MAIN_EXPRESSION,
    MAIN_FACTOR,
    MAIN_INFIX,
    MAIN_PREFIX,
    MAIN_SUFFIX,
    MAIN_SUFFIX_TERM,
    MAIN_SUFFIX_TERM0,
    MAIN_SUFFIX_TERM1,
    MAIN_TERM,
    MATCH_BLOCK,
    MATCH_CASE,
    MATCH_ELSE,
    MATCH_EXPRESSION,
    MATCH_STATEMENT,
    MATCH_TERMS,
    MATCH_TYPE,
    MATCH_WHEN,
    MODIFIER_AHEAD,
    MODIFIER_CALL,
    NS_CONCAT,
    NAMEPATH,
    NAMEPATH_FREE,
    NEW_BLOCK,
    NEW_PAIR,
    NEW_PAIR_KEY,
    NEW_STATEMENT,
    NUMBER,
    OFFSET_L,
    OFFSET_R,
    OP_AND_THEN,
    OP_BIND,
    OP_IMPORT_ALL,
    OP_NAMESPACE,
    OP_SLOT,
    OBJECT_STATEMENT,
    OP_NAMESPACE0,
    OP_NAMESPACE1,
    OP_NAMESPACE2,
    PROPORTION,
    PROPORTION2,
    PARAMETER_DEFAULT,
    PARAMETER_HINT,
    PARAMETER_ITEM,
    PARAMETER_ITEM_CONTROL,
    PARAMETER_PAIR,
    PATTERN_ITEM,
    PATTERN_ITEM1,
    PATTERN_ITEM2,
    PROCEDURAL_CALL,
    PROCEDURAL_NAME,
    PROGRAM,
    RANGE_CALL,
    RANGE_LITERAL,
    RANGE_LITERAL_INDEX0,
    RANGE_LITERAL_INDEX1,
    RANGE_OMIT,
    SHEBANG,
    SIGN,
    SIGN0,
    SIGN1,
    SKIP_SPACE,
    SLOT,
    SLOT_ITEM,
    SPECIAL,
    STANDARD_PATTERN,
    STATEMENT,
    STRING_FORMATTER,
    STRING_INTERPOLATION_COMPLEX,
    STRING_INTERPOLATION_SIMPLE,
    STRING_INTERPOLATION_TERM,
    STRING_INTERPOLATION_TEXT,
    STRING_INTERPOLATIONS,
    STRING_TEMPLATE_TERM,
    STRING_TEMPLATES,
    SUBSCRIPT_AXIS,
    SUBSCRIPT_ONLY,
    SUBSCRIPT_RANGE,
    SWITCH_STATEMENT,
    TEMPLATE_E,
    TEMPLATE_L,
    TEMPLATE_M,
    TEMPLATE_R,
    TEMPLATE_S,
    TEXT_CONTENT1,
    TEXT_CONTENT2,
    TEXT_CONTENT3,
    TEXT_CONTENT4,
    TEXT_CONTENT5,
    TEXT_CONTENT6,
    TEXT_LITERAL,
    TEXT_RAW,
    TEXT_L,
    TEXT_R,
    TEXT_X,
    TRAIT_BLOCK,
    TRAIT_TERM,
    TRY_STATEMENT,
    TUPLE_CALL,
    TUPLE_KEY,
    TUPLE_LITERAL,
    TUPLE_LITERAL_STRICT,
    TUPLE_PAIR,
    TUPLE_PATTERN,
    TUPLE_PATTERN_ITEM,
    TUPLE_TERMS,
    TYPE_EFFECT,
    TYPE_EXPRESSION,
    TYPE_FACTOR,
    TYPE_FACTOR0,
    TYPE_HINT,
    TYPE_INFIX,
    TYPE_PREFIX,
    TYPE_RETURN,
    TYPE_SUFFIX,
    TYPE_SUFFIX_TERM,
    TYPE_TERM,
    UNION_TERM,
    WHERE_BLOCK,
    WHERE_BOUND,
    WHITE_SPACE,
    /// Label for unnamed text literal
    HiddenText,
}

impl YggdrasilRule for ValkyrieRule {
    fn is_ignore(&self) -> bool {
        matches!(self, Self::HiddenText | Self::SKIP_SPACE | Self::COMMENT)
    }

    fn get_style(&self) -> &'static str {
        match self {
            Self::PROGRAM => "",
            Self::STATEMENT => "",
            Self::EOS => "",
            Self::EOS_FREE => "",
            Self::DEFINE_NAMESPACE => "",
            Self::OP_NAMESPACE => "",
            Self::DEFINE_IMPORT => "",
            Self::IMPORT_BLOCK => "",
            Self::IMPORT_TERM => "",
            Self::IMPORT_ALL => "",
            Self::IMPORT_SPACE => "",
            Self::IMPORT_NAME => "",
            Self::IMPORT_AS => "",
            Self::IMPORT_NAME_ITEM => "",
            Self::DEFINE_CONSTRAINT => "",
            Self::CONSTRAINT_PARAMETERS => "",
            Self::CONSTRAINT_BLOCK => "",
            Self::CONSTRAINT_STATEMENT => "",
            Self::CONSTRAINT_IMPLEMENTS => "",
            Self::WHERE_BLOCK => "",
            Self::WHERE_BOUND => "",
            Self::DEFINE_CLASS => "",
            Self::CLASS_BLOCK => "",
            Self::CLASS_TERM => "",
            Self::KW_CLASS => "",
            Self::DEFINE_FIELD => "",
            Self::PARAMETER_DEFAULT => "",
            Self::DEFINE_METHOD => "",
            Self::DEFINE_DOMAIN => "",
            Self::DOMAIN_TERM => "",
            Self::DEFINE_INHERIT => "",
            Self::INHERIT_TERM => "",
            Self::OBJECT_STATEMENT => "",
            Self::DEFINE_ENUMERATE => "",
            Self::FLAG_TERM => "",
            Self::FLAG_FIELD => "",
            Self::DEFINE_UNION => "",
            Self::UNION_TERM => "",
            Self::DEFINE_VARIANT => "",
            Self::KW_UNION => "",
            Self::DEFINE_TRAIT => "",
            Self::TRAIT_BLOCK => "",
            Self::TRAIT_TERM => "",
            Self::KW_TRAIT => "",
            Self::DEFINE_EXTENDS => "",
            Self::DEFINE_FUNCTION => "",
            Self::DEFINE_LAMBDA => "",
            Self::FUNCTION_MIDDLE => "",
            Self::TYPE_HINT => "",
            Self::TYPE_RETURN => "",
            Self::TYPE_EFFECT => "",
            Self::FUNCTION_PARAMETERS => "",
            Self::PARAMETER_ITEM => "",
            Self::PARAMETER_ITEM_CONTROL => "",
            Self::PARAMETER_PAIR => "",
            Self::PARAMETER_HINT => "",
            Self::CONTINUATION => "",
            Self::KW_FUNCTION => "",
            Self::DEFINE_VARIABLE => "",
            Self::LET_PATTERN => "",
            Self::STANDARD_PATTERN => "",
            Self::BARE_PATTERN => "",
            Self::BARE_PATTERN_ITEM => "",
            Self::TUPLE_PATTERN => "",
            Self::PATTERN_ITEM => "",
            Self::TUPLE_PATTERN_ITEM => "",
            Self::LOOP_STATEMENT => "",
            Self::LOOP_WHILE_STATEMENT => "",
            Self::LOOP_UNTIL_STATEMENT => "",
            Self::LOOP_EACH_STATEMENT => "",
            Self::IF_GUARD => "",
            Self::CONTROL_FLOW => "",
            Self::JUMP_LABEL => "",
            Self::EXPRESSION_ROOT => "",
            Self::MATCH_EXPRESSION => "",
            Self::SWITCH_STATEMENT => "",
            Self::MATCH_BLOCK => "",
            Self::MATCH_TERMS => "",
            Self::MATCH_TYPE => "",
            Self::MATCH_CASE => "",
            Self::CASE_PATTERN => "",
            Self::MATCH_WHEN => "",
            Self::MATCH_ELSE => "",
            Self::MATCH_STATEMENT => "",
            Self::KW_MATCH => "",
            Self::BIND_L => "",
            Self::BIND_R => "",
            Self::DOT_MATCH_CALL => "",
            Self::MAIN_EXPRESSION => "",
            Self::MAIN_TERM => "",
            Self::MAIN_FACTOR => "",
            Self::GROUP_FACTOR => "",
            Self::LEADING => "",
            Self::MAIN_SUFFIX_TERM => "",
            Self::MAIN_PREFIX => "",
            Self::TYPE_PREFIX => "",
            Self::MAIN_INFIX => "",
            Self::TYPE_INFIX => "",
            Self::MAIN_SUFFIX => "",
            Self::TYPE_SUFFIX => "",
            Self::INLINE_EXPRESSION => "",
            Self::INLINE_TERM => "",
            Self::INLINE_SUFFIX_TERM => "",
            Self::TYPE_EXPRESSION => "",
            Self::TYPE_TERM => "",
            Self::TYPE_FACTOR => "",
            Self::TYPE_SUFFIX_TERM => "",
            Self::TRY_STATEMENT => "",
            Self::NEW_STATEMENT => "",
            Self::NEW_BLOCK => "",
            Self::NEW_PAIR => "",
            Self::NEW_PAIR_KEY => "",
            Self::DOT_CALL => "",
            Self::DOT_CALL_ITEM => "",
            Self::DOT_CLOSURE_CALL => "",
            Self::INLINE_TUPLE_CALL => "",
            Self::TUPLE_CALL => "",
            Self::TUPLE_LITERAL => "",
            Self::TUPLE_LITERAL_STRICT => "",
            Self::TUPLE_TERMS => "",
            Self::TUPLE_PAIR => "",
            Self::TUPLE_KEY => "",
            Self::RANGE_CALL => "",
            Self::RANGE_LITERAL => "",
            Self::RANGE_LITERAL_INDEX0 => "",
            Self::RANGE_LITERAL_INDEX1 => "",
            Self::SUBSCRIPT_AXIS => "",
            Self::SUBSCRIPT_ONLY => "",
            Self::SUBSCRIPT_RANGE => "",
            Self::RANGE_OMIT => "",
            Self::DEFINE_GENERIC => "",
            Self::GENERIC_PARAMETER => "",
            Self::GENERIC_PARAMETER_PAIR => "",
            Self::GENERIC_CALL => "",
            Self::GENERIC_HIDE => "",
            Self::GENERIC_TERMS => "",
            Self::GENERIC_PAIR => "",
            Self::ANNOTATION_HEAD => "",
            Self::ANNOTATION_MIX => "",
            Self::ANNOTATION_TERM => "",
            Self::ANNOTATION_TERM_MIX => "",
            Self::ATTRIBUTE_BELOW_CALL => "",
            Self::ATTRIBUTE_BELOW_MARK => "",
            Self::ATTRIBUTE_ITEM => "",
            Self::ATTRIBUTE_NAME => "",
            Self::PROCEDURAL_CALL => "",
            Self::PROCEDURAL_NAME => "",
            Self::TEXT_LITERAL => "",
            Self::TEXT_RAW => "",
            Self::TEXT_L => "",
            Self::TEXT_R => "",
            Self::TEXT_X => "",
            Self::TEXT_CONTENT1 => "",
            Self::TEXT_CONTENT2 => "",
            Self::TEXT_CONTENT3 => "",
            Self::TEXT_CONTENT4 => "",
            Self::TEXT_CONTENT5 => "",
            Self::TEXT_CONTENT6 => "",
            Self::MODIFIER_CALL => "",
            Self::MODIFIER_AHEAD => "",
            Self::KEYWORDS_STOP => "",
            Self::IDENTIFIER_STOP => "",
            Self::SLOT => "",
            Self::SLOT_ITEM => "",
            Self::NAMEPATH_FREE => "",
            Self::NAMEPATH => "",
            Self::IDENTIFIER => "",
            Self::IDENTIFIER_BARE => "",
            Self::IDENTIFIER_RAW => "",
            Self::IDENTIFIER_RAW_TEXT => "",
            Self::SPECIAL => "",
            Self::NUMBER => "",
            Self::SIGN => "",
            Self::INTEGER => "",
            Self::DIGITS_X => "",
            Self::DECIMAL => "",
            Self::DECIMAL_X => "",
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
            Self::KW_ENUMERATE => "",
            Self::KW_FLAGS => "",
            Self::KW_LOOP => "",
            Self::KW_EACH => "",
            Self::KW_WHILE => "",
            Self::KW_UNTIL => "",
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
            Self::KW_END => "",
            Self::SHEBANG => "",
            Self::WHITE_SPACE => "",
            Self::SKIP_SPACE => "",
            Self::COMMENT => "",
            Self::STRING_INTERPOLATIONS => "",
            Self::STRING_INTERPOLATION_TERM => "",
            Self::ESCAPE_CHARACTER => "",
            Self::ESCAPE_UNICODE => "",
            Self::ESCAPE_UNICODE_CODE => "",
            Self::STRING_INTERPOLATION_SIMPLE => "",
            Self::STRING_INTERPOLATION_TEXT => "",
            Self::STRING_FORMATTER => "",
            Self::STRING_INTERPOLATION_COMPLEX => "",
            Self::STRING_TEMPLATES => "",
            Self::STRING_TEMPLATE_TERM => "",
            Self::EXPRESSION_TEMPLATE => "",
            Self::FOR_TEMPLATE => "",
            Self::FOR_TEMPLATE_BEGIN => "",
            Self::FOR_TEMPLATE_ELSE => "",
            Self::FOR_TEMPLATE_END => "",
            Self::TEMPLATE_S => "",
            Self::TEMPLATE_E => "",
            Self::TEMPLATE_L => "",
            Self::TEMPLATE_R => "",
            Self::TEMPLATE_M => "",
            Self::EOS0 => "",
            Self::EOS1 => "",
            Self::OP_NAMESPACE0 => "",
            Self::OP_NAMESPACE1 => "",
            Self::OP_NAMESPACE2 => "",
            Self::KW_TRAIT0 => "",
            Self::KW_TRAIT1 => "",
            Self::PATTERN_ITEM1 => "",
            Self::PATTERN_ITEM2 => "",
            Self::KW_MATCH0 => "",
            Self::KW_MATCH1 => "",
            Self::MAIN_SUFFIX_TERM0 => "",
            Self::MAIN_SUFFIX_TERM1 => "",
            Self::INLINE_SUFFIX_TERM0 => "",
            Self::INLINE_SUFFIX_TERM1 => "",
            Self::TYPE_FACTOR0 => "",
            Self::SIGN0 => "",
            Self::SIGN1 => "",
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
    LoopEachStatement(LoopEachStatementNode<'i>),
    LoopWhileStatement(LoopWhileStatementNode<'i>),
    LoopUntilStatement(LoopUntilStatementNode<'i>),
    LoopStatement(LoopStatementNode<'i>),
    ExpressionRoot(ExpressionRootNode<'i>),
    Eos(EosNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EosNode<'i> {
    Omit(Eos0Node<'i>),
    Show(Eos1Node<'i>),
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
pub enum OpNamespaceNode<'i> {
    Main(OpNamespace0Node<'i>),
    Test(OpNamespace1Node<'i>),
    Hide(OpNamespace2Node<'i>),
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
    EosFree(EosFreeNode<'i>),
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
    EosFree(EosFreeNode<'i>),
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
    EosFree(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlagFieldNode<'i> {
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
    EosFree(EosFreeNode<'i>),
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
pub struct TraitBlockNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TraitTermNode<'i> {
    ProceduralCall(ProceduralCallNode<'i>),
    DefineMethod(DefineMethodNode<'i>),
    DefineField(DefineFieldNode<'i>),
    EosFree(EosFreeNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KwTraitNode<'i> {
    Trait(KwTrait0Node<'i>),
    Interface(KwTrait1Node<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefineExtendsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
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
pub enum PatternItemNode<'i> {
    TuplePatternItem(TuplePatternItemNode<'i>),
    OmitDict(PatternItem1Node<'i>),
    OmitList(PatternItem2Node<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternItemNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopWhileStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopUntilStatementNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopEachStatementNode<'i> {
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
    Comma(CommaNode<'i>),
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
pub enum KwMatchNode<'i> {
    Match(KwMatch0Node<'i>),
    Catch(KwMatch1Node<'i>),
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
pub enum MainSuffixTermNode<'i> {
    MainSuffixTerm0(MainSuffixTerm0Node<'i>),
    MainSuffixTerm1(MainSuffixTerm1Node<'i>),
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
pub enum InlineSuffixTermNode<'i> {
    MainSuffix(InlineSuffixTerm0Node<'i>),
    DotCall(InlineSuffixTerm1Node<'i>),
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
    TypeExpression(TypeFactor0Node<'i>),
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
pub struct DotCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DotCallItemNode<'i> {
    Namepath(NamepathNode<'i>),
    Integer(IntegerNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotClosureCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
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
    AttributeBelowCall(AttributeBelowCallNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationTermMixNode<'i> {
    AttributeBelowCall(AttributeBelowCallNode<'i>),
    ProceduralCall(ProceduralCallNode<'i>),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeBelowCallNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeBelowMarkNode<'i> {
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
pub enum SignNode<'i> {
    Positive(Sign0Node<'i>),
    Netative(Sign1Node<'i>),
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
pub struct KwEnumerateNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwFlagsNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwLoopNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEachNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwWhileNode<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwUntilNode<'i> {
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
pub struct KwEndNode<'i> {
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
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eos0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eos1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNamespace0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNamespace1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNamespace2Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTrait0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTrait1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternItem1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternItem2Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwMatch0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwMatch1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainSuffixTerm0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainSuffixTerm1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineSuffixTerm0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineSuffixTerm1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeFactor0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sign0Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sign1Node<'i> {
    pair: TokenPair<'i, ValkyrieRule>,
}
