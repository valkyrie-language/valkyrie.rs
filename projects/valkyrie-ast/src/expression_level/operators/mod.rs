use super::*;
use core::ptr::eq;

mod display;

mod logic;

/// All builtin operator in valkyrie language
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValkyrieOperator {
    /// unassigned operator symbol
    Placeholder,
    /// prefix operator: `!`
    Not,
    /// prefix operator: `+`
    Positive,
    /// prefix operator: `-`
    Negative,
    /// prefix type operator: `+`
    CovariantType,
    /// prefix type operator: `-`
    ContravariantType,
    /// prefix operator: `&`
    Box,
    /// prefix operator: `*`
    Unbox,
    /// prefix operator: `.., ...`
    Unpack {
        /// unpack level
        level: u8,
    },
    /// prefix operator: `⅟`
    Reciprocal,
    /// prefix operator: `√`, `∛`, `∜`     
    Roots(u8),
    /// infix operator: `=`
    Assign {
        /// monadic assign: `?=`
        monadic: bool,
    },
    /// binary operator: `+`
    Plus,
    /// binary operator: `+=`
    PlusAssign,
    /// binary operator: `++`
    Concat,
    /// binary operator: `<`
    Less {
        /// binary operator: `<=`, `⩽`
        equal: bool,
    },
    /// binary operator: `>`
    Greater {
        /// binary operator: `>=`, `⩾`
        equal: bool,
    },
    /// binary operator: `≫`, `>>`
    MuchGreater,
    /// binary operator:
    MuchLess,
    /// binary operator: `⋙`, `>>>`
    VeryMuchGreater,
    /// binary operator:
    VeryMuchLess,
    /// binary operator: `≡`, `≢`
    Equal {
        /// negative operator `!=`
        negative: bool,
    },
    /// binary operator: `≡`, `≢`
    StrictlyEqual {
        /// negative operator `=!=`
        negative: bool,
    },
    /// binary operator: `..<, ..=`
    RangeTo {
        /// inclusive operator: `..=`
        equal: bool,
    },

    /// binary operator: `as, as?, as!`
    As {
        /// `as?`
        r#try: bool,
    },
    /// binary operator: `⊑, ⋢, is, is not`
    Is {
        /// negative operator: `⋢, is not`
        negative: bool,
    },
    /// binary operator: `∈`, `∉`, `in`, `not in`
    In {
        /// negative operator: `∉, not in`
        negative: bool,
    },
    /// binary operator: `∋`, `∌`    
    Contains {
        /// negative operator: `∉, not in`
        negative: bool,
    },
    /// binary operator: `-`
    Minus,
    /// binary operator: `-=`
    MinusAssign,
    /// binary operator: `*`
    Multiply,
    /// binary operator: `/`
    Divide,
    /// binary operator: `%`
    Remainder,
    /// binary operator: `٪, ⁒, %%`
    Modulo,
    /// binary operator: `÷, /%`
    DivideRemainder,
    /// binary operator: `/_`
    DivideFloor,
    /// binary operator: `^`
    Power,
    /// binary operator: `√`
    Surd,
    /// binary operator: `&,  |`
    LogicMatrix {
        /// binary operator: `&,  |`
        mask: LogicMatrix,
    },
    /// binary operator: `/@`
    Map,
    /// suffix operator: `?`
    Optional,
    /// suffix operator: `!`
    QuickRaise,
    /// suffix operator: `℃`
    Celsius,
    /// suffix operator: `℉`
    Fahrenheit,
    /// suffix operator: `⁒, %, ‰, ‱`
    DivideByDecimal {
        /// decimal power: `a⁒, b%, c‰, d‱`
        power: u8,
    },
    /// suffix operator: `ᵀ`, `\^T`, `\transpose`
    Transpose,
    /// suffix operator: `ᴴ`, `\^H`, `\conjugate_transpose
    Transjugate,
    ///  suffix operator: `\hat`
    Hermitian,
}

/// Variants of all logical operators.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogicMatrix {
    /// 1. always false
    False = 0b0,
    /// 2. p and q            ∧
    And = 0b1,
    /// 3. p and not q        >
    AndNot = 0b10,
    /// 4. p                  ⊣
    P = 0b11,
    /// 5. not p and q        <
    NotAnd = 0b100,
    /// 6. q                  ⊢
    Q = 0b101,
    /// 7. xor                ≠
    Xor = 0b110,
    /// 8. p or q             ∨
    Or = 0b111,
    /// 9. not p and not q    ⍱
    Nor = 0b1000,
    /// 10. p == q
    Xnor = 0b1001,
    /// 11. not q              ~⊢
    NotQ = 0b1010,
    /// 12. p or not q         ≥
    OrNot = 0b1011,
    /// 13. not p              ~⊣
    NotP = 0b1100,
    /// 14. not p or q         ≤
    NotOr = 0b1101,
    /// 15. not p or not q     ⍲
    Nand = 0b1110,
    /// 16. always true
    True = 0b1111,
}

impl From<LogicMatrix> for ValkyrieOperator {
    fn from(logic: LogicMatrix) -> Self {
        Self::LogicMatrix { mask: logic }
    }
}

impl ValkyrieOperator {
    /// Get the precedence of the operator, larger number means higher precedence.
    pub fn precedence(&self) -> Precedence {
        let n = match self {
            Self::Placeholder => 0,
            //
            Self::Concat => 14000,
            Self::As { .. } => 14000,
            Self::Is { .. } => 14000,
            Self::In { .. } => 14000,
            Self::Contains { .. } => 14000,
            Self::Assign { .. } => 14000,
            Self::RangeTo { .. } => 14000,
            // prefix - 3
            Self::PlusAssign => 14100,
            Self::MinusAssign => 14100,
            // infix - 3
            Self::LogicMatrix { .. } => 14700,
            Self::Equal { .. } => 14700,
            Self::StrictlyEqual { .. } => 14700,
            Self::Map => 14700,
            // infix - 2
            Self::Less { .. } | Self::Greater { .. } => 14800,
            // infix - 1
            Self::MuchLess | Self::MuchGreater => 14900,
            Self::VeryMuchLess | Self::VeryMuchGreater => 14950,
            // infix + 0
            Self::Plus => 15000,
            Self::Minus => 15000,
            // infix + 1
            Self::Multiply => 15100,
            Self::Divide => 15100,
            Self::Remainder => 15100,
            Self::Modulo => 15100,
            Self::DivideRemainder => 15100,
            Self::DivideFloor => 15100,
            // infix + 2
            Self::Power => 15200,
            Self::Surd => 15200,
            // prefix + 0
            Self::Not => 25000,
            Self::Positive | Self::Negative => 25000,
            Self::CovariantType | Self::ContravariantType => 25000,
            Self::Box => 25000,
            Self::Unbox => 25000,
            Self::Unpack { .. } => 25000,
            Self::Reciprocal => 25000,
            Self::Roots(_) => 25000,
            // postfix + 0
            Self::Optional => 45000,
            Self::QuickRaise => 45000,
            Self::Celsius => 45000,
            Self::Fahrenheit => 45000,
            Self::Transpose => 45000,
            Self::Transjugate => 45000,
            Self::Hermitian => 45000,
            Self::DivideByDecimal { .. } => 45000,
        };
        Precedence(n)
    }
    /// Get the normalised string representation of the operator.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Placeholder => "???",
            Self::Not => "!",
            Self::Concat => "++",
            Self::Positive => "+",
            Self::Negative => "-",
            Self::CovariantType => "+",
            Self::ContravariantType => "-",
            Self::Plus => "+",
            Self::PlusAssign => "+=",
            Self::Minus => "-",
            Self::MinusAssign => "-=",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
            Self::Modulo => "⁒",
            Self::DivideRemainder => "÷",
            Self::DivideFloor => "/_",
            Self::Power => "^",
            Self::Surd => "√",
            Self::Optional => "?",
            Self::QuickRaise => "!",
            Self::Celsius => "℃",
            Self::Fahrenheit => "℉",
            Self::Transpose => "ᵀ",
            Self::Transjugate => "ᴴ",
            Self::Hermitian => "Hermitian",
            Self::Box => "&",
            Self::Unbox => "*",
            Self::Unpack { level } => match level {
                2 => "⁑",
                _ => "⁂",
            },
            Self::Greater { equal } => match equal {
                true => "⩾",
                false => ">",
            },
            Self::MuchGreater => "≫",
            Self::VeryMuchGreater => "⋙",
            Self::Less { equal } => match equal {
                true => "⩽",
                false => "<",
            },
            Self::MuchLess => "≪",
            Self::VeryMuchLess => "⋘",
            Self::As { r#try } => match r#try {
                true => "as?",
                false => "as",
            },
            Self::Is { negative } => match negative {
                true => "⋢",
                false => "⊑",
            },
            Self::In { negative } => match negative {
                true => "∉",
                false => "∈",
            },
            Self::Contains { negative } => match negative {
                true => "∌",
                false => "∋",
            },
            Self::RangeTo { equal } => match equal {
                true => "..=",
                false => "..<",
            },
            Self::Equal { negative } => match negative {
                true => "≠",
                false => "==",
            },
            Self::StrictlyEqual { negative } => match negative {
                true => "≢",
                false => "≡",
            },
            Self::Reciprocal => "⅟",
            Self::Roots(v) => match v {
                3 => "∛",
                4 => "∜",
                _ => "√",
            },
            Self::DivideByDecimal { power } => match power {
                1 => "⁒",
                3 => "‰",
                4 => "‱",
                _ => "%",
            },
            Self::Assign { monadic } => match monadic {
                true => "?=",
                false => "=",
            },
            Self::LogicMatrix { mask } => mask.as_str(),
            Self::Map => "/@",
        }
    }
    /// Get the associativity of the operator.
    pub fn associativity(&self) -> Associativity {
        match self {
            ValkyrieOperator::Power => Associativity::Right,
            ValkyrieOperator::Surd => Associativity::Right,
            _ => Associativity::Left,
        }
    }

    /// user input arguments
    pub fn accept_arguments(&self) -> usize {
        match self {
            Self::Plus | Self::Multiply => 0,
            _ => 1,
        }
    }
    /// if this operatr can be override
    pub fn overrideable(&self) -> bool {
        match self {
            Self::Equal { negative: true } => false,
            Self::StrictlyEqual { negative: true } => false,
            _ => true,
        }
    }
}

/// An expression node with unary operator and base expression
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnaryNode {
    ///   The operator of the node
    pub operator: OperatorNode,
    ///   The expression that the operator is applied to
    pub base: ExpressionKind,
}
/// An expression node with binary operator and two expressions
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinaryNode {
    ///  The operator of the node
    pub infix: OperatorNode,
    ///  The left hand side of the node
    pub lhs: ExpressionKind,
    ///  The right hand side of the node
    pub rhs: ExpressionKind,
}

/// An operator node with kind and range
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OperatorNode {
    /// The valid kind of operator
    pub kind: ValkyrieOperator,
    /// The range of the node
    pub span: Range<u32>,
}
impl ValkyrieNode for UnaryNode {
    fn get_range(&self) -> Range<u32> {
        Range { start: self.operator.span.start, end: self.base.get_end() }
    }
}
impl ValkyrieNode for BinaryNode {
    fn get_range(&self) -> Range<u32> {
        let head = self.lhs.get_range().start;
        let tail = self.rhs.get_range().end;
        head..tail
    }
}
