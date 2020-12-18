use crate::{
    helpers::{ignore, parse_any_name_path, parse_eos, parse_name_join_dot},
    traits::ThisParser,
    utils::{get_span, parse_expression_node},
};
use lispify::Lisp;
use pex::{helpers::CommentLine, BracketPattern, ParseResult, ParseState, Regex, StopBecause};
use std::sync::LazyLock;
use valkyrie_ast::{
    AnnotationList, AnnotationNode, ApplyCallNode, ClassDeclaration, ControlNode, DocumentationNode, EnumerateDeclaration,
    ExpressionContext, ExpressionNode, FlagsDeclaration, ForLoop, FunctionDeclaration, GenericCallNode, GuardLetStatement,
    GuardStatement, IdentifierNode, ImportAliasNode, ImportGroupNode, ImportStatement, ImportTermNode, LambdaArgumentNode,
    LambdaNode, LetBindNode, NamePathNode, NamespaceDeclaration, NamespaceKind, NewConstructNode, PatternExpressionType,
    ProgramRoot, StatementNode, StatementType, TableTermNode, TaggedDeclaration, TypingExpression, UnionDeclaration, WhileLoop,
};

mod annotation;
mod def_var;
mod documentation;
mod import;
mod lambda;
mod new;

pub struct ReplRoot {
    pub statements: Vec<StatementNode>,
}

impl ThisParser for ReplRoot {
    fn parse(input: ParseState) -> ParseResult<Self> {
        input.match_repeats(|s| parse_statement_node(s, false)).map_inner(|s| ReplRoot { statements: s })
    }

    fn as_lisp(&self) -> Lisp {
        unreachable!()
    }
}

impl ThisParser for ProgramRoot {
    fn parse(input: ParseState) -> ParseResult<Self> {
        input.match_repeats(|s| parse_statement_node(s, false)).map_inner(|s| ProgramRoot { statements: s })
    }

    fn as_lisp(&self) -> Lisp {
        unreachable!()
    }
}

impl ThisParser for StatementNode {
    /// - [term](ExpressionType::parse)
    fn parse(input: ParseState) -> ParseResult<Self> {
        parse_statement_node(input, false)
    }

    fn as_lisp(&self) -> Lisp {
        self.r#type.as_lisp()
    }
}

pub fn parse_statement_node(input: ParseState, repl: bool) -> ParseResult<StatementNode> {
    let parser = match repl {
        true => parse_repl_statements,
        false => StatementType::parse,
    };
    let (state, expr) = input.skip(ignore).match_fn(parser)?;
    let (state, eos) = parse_eos(state)?;
    state.finish(StatementNode { r#type: expr, end_semicolon: eos, span: get_span(input, state) })
}

impl ThisParser for StatementType {
    fn parse(input: ParseState) -> ParseResult<Self> {
        input
            .begin_choice()
            .choose(|s| DocumentationNode::parse(s).map_into())
            .choose(|s| AnnotationNode::parse(s).map_into())
            .choose(|s| AnnotationList::parse(s).map_into())
            .choose(|s| NamespaceDeclaration::parse(s).map_into())
            .choose(|s| ImportStatement::parse(s).map_into())
            .choose(|s| ClassDeclaration::parse(s).map_into())
            .choose(|s| UnionDeclaration::parse(s).map_into())
            .choose(|s| TaggedDeclaration::parse(s).map_into())
            .choose(|s| EnumerateDeclaration::parse(s).map_into())
            .choose(|s| FlagsDeclaration::parse(s).map_into())
            .choose(function_with_head)
            .choose_from(LetBindNode::parse)
            .choose_from(GuardLetStatement::parse)
            .choose_from(GuardStatement::parse)
            .choose_from(WhileLoop::parse)
            .choose_from(ForLoop::parse)
            .choose_from(ControlNode::parse)
            .choose(|s| parse_expression_node(s, ExpressionContext::in_free()).map_into())
            .end_choice()
    }

    fn as_lisp(&self) -> Lisp {
        match self {
            Self::Nothing => Lisp::default(),
            Self::Namespace(v) => v.as_lisp(),
            Self::Import(v) => v.as_lisp(),
            Self::While(v) => v.as_lisp(),
            Self::For(v) => v.as_lisp(),
            Self::Class(v) => v.as_lisp(),
            Self::ClassField(v) => v.as_lisp(),
            Self::ClassMethod(v) => v.as_lisp(),
            Self::Expression(v) => v.as_lisp(),
            Self::Function(v) => v.as_lisp(),
            Self::Control(v) => v.as_lisp(),
            Self::Document(v) => v.as_lisp(),
            Self::LetBind(v) => v.as_lisp(),
            Self::Guard(v) => v.as_lisp(),
            Self::Flags(v) => v.as_lisp(),
            Self::EnumerateField(v) => v.as_lisp(),
            Self::Tagged(v) => v.as_lisp(),
            Self::Variant(v) => v.as_lisp(),
            Self::Union(v) => v.as_lisp(),
            Self::Enumerate(v) => v.as_lisp(),
            Self::UnionField(v) => v.as_lisp(),
            Self::Annotation(v) => v.as_lisp(),
            Self::GuardLet(v) => v.as_lisp(),
        }
    }
}

pub fn parse_repl_statements(input: ParseState) -> ParseResult<StatementType> {
    input
        .begin_choice()
        .choose_from(DocumentationNode::parse)
        .choose(|s| AnnotationNode::parse(s).map_into())
        .choose(|s| AnnotationList::parse(s).map_into())
        .choose(|s| NamespaceDeclaration::parse(s).map_into())
        .choose(|s| ImportStatement::parse(s).map_into())
        .choose(|s| ClassDeclaration::parse(s).map_into())
        .choose(|s| UnionDeclaration::parse(s).map_into())
        .choose(|s| TaggedDeclaration::parse(s).map_into())
        .choose(|s| EnumerateDeclaration::parse(s).map_into())
        .choose(|s| FlagsDeclaration::parse(s).map_into())
        .choose_from(FunctionDeclaration::parse)
        .choose_from(LetBindNode::parse)
        .choose_from(GuardLetStatement::parse)
        .choose_from(GuardStatement::parse)
        .choose_from(WhileLoop::parse)
        .choose_from(ForLoop::parse)
        .choose_from(ControlNode::parse)
        .choose(|s| parse_expression_node(s, ExpressionContext::in_free()).map_into())
        .end_choice()
}

fn function_with_head(input: ParseState) -> ParseResult<StatementType> {
    let (state, mods) = input.match_repeats(|s| {
        let (state, _) = s.skip(ignore).match_negative(FunctionDeclaration::parse, "KW_FUNCTION")?;
        IdentifierNode::parse(state)
    })?;
    let (state, mut func) = state.skip(ignore).match_fn(FunctionDeclaration::parse)?;
    func.modifiers = mods;
    state.finish(StatementType::Function(Box::new(func)))
}
