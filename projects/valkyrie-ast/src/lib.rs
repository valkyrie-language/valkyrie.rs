#![no_std]
#![allow(unused_imports)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/91894079")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/91894079")]

extern crate alloc;

mod control_flow;
mod expression_level;
pub mod helper;
mod package_level;
mod patterns;
mod string_like;
pub(crate) mod utils;

pub use crate::{
    control_flow::{
        control::{ControlKind, ControlNode, LabelNode},
        do_catch::{MatchCallNode, MatchKind, MatchStatement},
        do_try::TryStatement,
        jmp_guard::{GuardPattern, GuardStatement},
        jmp_if::{BreakStatement, ElseStatement, IfBranchNode, IfStatement, JumpStatement},
        jmp_switch::SwitchStatement,
        loop_for::{ForBarePattern, ForLoop},
        loop_pure::{LoopContinuation, LoopStatement},
        loop_while::{WhileConditionNode, WhileLoop, WhileLoopKind},
    },
    expression_level::{
        annotations::{AnnotationNode, AttributeKind, AttributeList, AttributeTerm, ModifierList, ProceduralNode},
        argument::{ArgumentKey, ArgumentTerm, ArgumentsList},
        call_apply::ApplyCallNode,
        call_dot::{DotCallNode, DotCallTerm},
        call_generic::{GenericCallNode, GenericCallTerm},
        call_subscript::SubscriptCallNode,
        ctor::{CollectorTerm, ConstructNewNode},
        lambda::{ClosureCallNode, LambdaNode},
        number::NumberLiteralNode,
        operators::{BinaryNode, LogicMatrix, OperatorNode, UnaryNode, ValkyrieOperator},
        parameter::{ParameterKind, ParameterMixed, ParameterTerm, ParametersList},
        range::{RangeKind, RangeNode, RangeTermNode},
        symbol::{BooleanNode, IdentifierNode, LambdaSlotItem, LambdaSlotNode, NamePathNode, NullNode, OutputNode},
        tuple::{TupleKind, TupleNode},
        ExpressionContext, ExpressionKind, ExpressionNode, TypingExpression,
    },
    package_level::{
        classes::{
            ClassDeclaration, ClassKind, ClassTerm, ConstructObjectNode, DomainDeclaration, FieldDeclaration, MethodDeclaration,
        },
        constraints::{ConstraintDeclaration, ConstraintTerm},
        documentation::DocumentationList,
        flags::{EncodeDeclaration, FlagDeclaration, FlagKind, FlagTerm},
        function::{FunctionDeclaration, FunctionKind, FunctionReturnNode, StatementBlock},
        guarantee::{EffectTypeNode, GuaranteeNode},
        import::{
            ImportAliasItem, ImportAliasNode, ImportAllNode, ImportGroupNode, ImportKind, ImportResolvedItem, ImportState,
            ImportStatement, ImportTermNode,
        },
        labeled::{GotoStatement, LabelStatement},
        let_bind::{LetBindNode, VariableDeclaration},
        namespace::{NamespaceDeclaration, NamespaceKind},
        program::ProgramRoot,
        statements::{StatementContext, StatementKind},
        traits::{ExtendsStatement, TraitDeclaration, TraitKind, TraitTerm},
        unions::{UnionDeclaration, UnionTerm, VariantDeclaration},
    },
    patterns::{
        ArrayPatternNode, ClassPatternNode, IdentifierPattern, ImplicitCaseNode, PatternBranch, PatternCaseNode,
        PatternCondition, PatternNode, PatternTypeNode, PatternWhenNode, PatternsList, TuplePatternNode, UnionPatternNode,
    },
    string_like::{
        string_formatter::{FormatterNode, FormatterTerm},
        string_literal::{StringLiteralNode, StringTextNode},
        string_template::{
            TemplateCloseNode, TemplateCommentNode, TemplateInlineNode, TemplateLineType, TemplateNode, TemplateOpenNode,
        },
    },
};
