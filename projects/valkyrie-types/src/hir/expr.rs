//! Expression, capture, and capture mode definitions for HIR.

use super::{HirBlock, HirFunction, HirGeneric, HirIdentifier, HirLiteral, HirMatchArm, HirParam, HirPattern, HirType};
use crate::{Identifier, NamePath, SourceSpan};

/// An expression in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirExpr {
    /// The kind of the expression.
    pub kind: HirExprKind,
    /// The source span for error reporting.
    pub span: SourceSpan,
}

/// Capture mode for variables in anonymous classes and closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CaptureMode {
    /// Capture by value (for primitive types).
    ByValue,
    /// Capture by reference (for reference types).
    ByReference,
}

/// Storage location hint for captured variables.
///
/// This enum indicates where a captured variable should be stored,
/// based on escape analysis results. The storage hint helps the
/// code generator choose the optimal storage strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CaptureStorage {
    /// Store on the stack for non-escaping captures.
    ///
    /// When an anonymous class does not escape its defining scope,
    /// captured variables can be stored on the stack for better
    /// performance and cache locality.
    #[default]
    Stack,
    /// Store on the heap for escaping captures.
    ///
    /// When an anonymous class escapes its defining scope (e.g.,
    /// returned from a function or stored in a data structure),
    /// captured variables must be stored on the heap to ensure
    /// they remain valid after the defining scope exits.
    Heap,
}

/// A captured variable in an anonymous class or closure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirCapture {
    /// The identifier of the captured variable.
    pub identifier: HirIdentifier,
    /// The type of the captured variable.
    pub ty: HirType,
    /// How the variable is captured.
    pub mode: CaptureMode,
    /// Whether the capture is mutable.
    pub is_mutable: bool,
    /// Storage hint based on escape analysis.
    ///
    /// This field is populated during escape analysis and indicates
    /// whether the captured variable should be stored on the stack
    /// or heap based on whether the anonymous class escapes.
    pub storage_hint: CaptureStorage,
}

/// The kind of an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirExprKind {
    /// A literal value.
    Literal(HirLiteral),
    /// A variable reference.
    Variable(HirIdentifier),
    /// A path expression (e.g., module::item).
    Path(NamePath),
    /// A function call.
    Call {
        /// The callee expression.
        callee: Box<HirExpr>,
        /// The arguments.
        args: Vec<HirExpr>,
    },
    /// A constructor call.
    Construct {
        /// The type name.
        name: Identifier,
        /// The arguments.
        args: Vec<HirExpr>,
    },
    /// Field initialization expression.
    ///
    /// Represents a named field initialization in object construction.
    ///
    /// ```v
    /// Point { x: 10, y: 20 }
    /// Point { x, y }  // shorthand: x and y are local variables
    /// ```
    FieldInit {
        /// The field name.
        name: Identifier,
        /// The field value.
        value: Box<HirExpr>,
    },
    /// Subscript expression.
    Subscript {
        /// The object expression.
        object: Box<HirExpr>,
        /// The index expression.
        index: Box<HirExpr>,
    },
    /// 数组元素赋值表达式：`object[index] = value`。
    ///
    /// 将 `value` 写入 `object` 数组的 `index` 位置。
    /// 结果类型为 `Unit`。
    StoreSubscript {
        /// 被写入的数组对象。
        object: Box<HirExpr>,
        /// 索引表达式。
        index: Box<HirExpr>,
        /// 要写入的值。
        value: Box<HirExpr>,
    },
    /// 数组创建表达式：`new [ElementType](length)`。
    ///
    /// 创建一个指定元素类型和长度的一维零基数组。
    /// `element_type` 是数组元素类型，`length` 是数组长度表达式。
    /// 结果类型为 `Array<ElementType>`。
    ArrayNew {
        /// 数组元素类型。
        element_type: HirType,
        /// 数组长度表达式。
        length: Box<HirExpr>,
    },
    /// 字段访问表达式：`object.field`。
    ///
    /// 读取 `object` 对象的 `field` 字段值。
    FieldAccess {
        /// 被访问字段的对象表达式。
        object: Box<HirExpr>,
        /// 字段名。
        field: Identifier,
    },
    /// 字段赋值表达式：`object.field = value`。
    ///
    /// 将 `value` 写入 `object` 对象的 `field` 字段。
    /// 结果类型为 `Unit`。
    StoreField {
        /// 被写入字段的对象表达式。
        object: Box<HirExpr>,
        /// 字段名。
        field: Identifier,
        /// 要写入的值。
        value: Box<HirExpr>,
    },
    /// Explicit generic application on a value path or callable.
    GenericApply {
        /// The callee expression before applying generics.
        callee: Box<HirExpr>,
        /// The generic arguments.
        arguments: Vec<HirType>,
    },
    /// A block expression.
    Block(Box<HirBlock>),
    /// A lambda expression.
    Lambda {
        /// Generic parameters.
        generics: Vec<HirGeneric>,
        /// Parameters.
        params: Vec<HirParam>,
        /// Return type.
        return_type: HirType,
        /// The body.
        body: Box<HirBlock>,
    },
    /// Anonymous class expression.
    ///
    /// Represents an inline class definition that can implement traits.
    ///
    /// ```v
    /// let obj = class { x: 10, y: 20 }
    /// let impl_trait = class: Trait { ... }
    /// ```
    AnonymousClass {
        /// Parent traits or classes to implement/extend.
        parents: Vec<NamePath>,
        /// Fields with their initial values.
        fields: Vec<(Identifier, Box<HirExpr>)>,
        /// Methods defined in the anonymous class.
        methods: Vec<HirFunction>,
        /// Variables captured from the enclosing scope.
        captures: Vec<HirCapture>,
        /// Generated class name for the anonymous class.
        class_name: Option<Identifier>,
    },
    /// An if expression.
    If {
        /// The condition.
        condition: Box<HirExpr>,
        /// The then branch.
        then_branch: Box<HirBlock>,
        /// The else branch.
        else_branch: Option<Box<HirBlock>>,
    },
    /// A match expression.
    Match {
        /// The scrutinee.
        scrutinee: Box<HirExpr>,
        /// The match arms.
        arms: Vec<HirMatchArm>,
    },
    /// A loop expression.
    Loop {
        /// Optional label for break/continue.
        label: Option<Identifier>,
        /// Optional pattern for iterator loop.
        pattern: Option<HirPattern>,
        /// Optional iterator source for `loop pattern in source`.
        iterator: Option<Box<HirExpr>>,
        /// Optional condition for while loop.
        condition: Option<Box<HirExpr>>,
        /// The loop body.
        body: Box<HirBlock>,
    },
    /// A return expression.
    Return(Option<Box<HirExpr>>),
    /// A variable assignment expression: `target = value`.
    ///
    /// 对可变变量的赋值，更新绑定指向新值。
    Assign {
        /// The target variable name.
        target: Identifier,
        /// The value to assign.
        value: Box<HirExpr>,
    },
    /// A break expression.
    Break {
        /// Optional label.
        label: Option<Identifier>,
        /// Optional value (for labeled breaks).
        expr: Option<Box<HirExpr>>,
    },
    /// A continue expression.
    Continue {
        /// Optional label.
        label: Option<Identifier>,
    },
    /// A yield expression.
    Yield(Option<Box<HirExpr>>),
    /// A yield from expression.
    YieldFrom(Box<HirExpr>),
    /// A raise (throw) expression.
    Raise(Box<HirExpr>),
    /// A resume expression.
    ///
    /// Resumes execution from an effect handler with a value.
    /// Only valid inside a catch block.
    Resume(Box<HirExpr>),
    /// A catch (try) expression.
    Catch {
        /// The expression to try.
        expr: Box<HirExpr>,
        /// The catch arms.
        arms: Vec<HirMatchArm>,
    },
    /// With expression for functional record updates.
    ///
    /// Creates a new record by copying an existing one and updating specified fields.
    With {
        /// The base expression to copy from.
        base: Box<HirExpr>,
        /// Field updates to apply.
        updates: Vec<(Identifier, HirExpr)>,
    },
    /// Super call expression for constructor chaining.
    ///
    /// Represents a call to a parent class constructor within a subclass constructor.
    ///
    /// ```v
    /// class Derived(Base) {
    ///     initiate(mut self, x: i32, y: i32) {
    ///         super.initiate(x)  // Call parent constructor
    ///         self.y = y
    ///     }
    /// }
    /// ```
    SuperCall {
        /// The parent class name or alias for renamed inheritance.
        ///
        /// In single inheritance, this is typically None (calls the single parent).
        /// In renamed inheritance, this specifies which parent to call:
        /// ```v
        /// class Child(primary: ParentA, secondary: ParentB) {
        ///     initiate(mut self) {
        ///         super.primary.initiate()  // alias: "primary"
        ///         super.secondary.initiate()  // alias: "secondary"
        ///     }
        /// }
        /// ```
        parent_alias: Option<Identifier>,
        /// The method name to call (usually "initiate").
        method: Identifier,
        /// Arguments passed to the parent constructor.
        args: Vec<HirExpr>,
    },
}
