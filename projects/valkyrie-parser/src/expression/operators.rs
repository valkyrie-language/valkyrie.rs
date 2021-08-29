use super::*;
use crate::traits::YggdrasilNodeExtension;
use yggdrasil_rt::YggdrasilNode;
impl<'i> crate::MainPrefixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            "!" => Not,
            "+" => Positive,
            "-" => Negative,
            "&" => Box,
            "*" => Unbox,
            "⅟" => Reciprocal,
            "√" => Roots(2),
            "∛" => Roots(3),
            "∜" => Roots(4),
            ".." => Unpack { level: 2 },
            "..." => Unpack { level: 3 },
            _ => unimplemented!("{} is a unknown prefix operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}
impl<'i> crate::TypePrefixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            "!" => Not,
            "+" => CovariantType,
            "-" => ContravariantType,
            "&" => Box,
            _ => unimplemented!("{} is a unknown prefix operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}

impl<'i> crate::InlineSuffixTerm0Node<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        todo!()
    }
}

impl<'i> crate::MainInfixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use valkyrie_ast::LogicMatrix;
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            s if s.starts_with("is") => Is { negative: s.ends_with("not") },
            s if s.ends_with("in") => In { negative: s.ends_with("not") },
            "as" => As { r#try: false },
            "as?" => As { r#try: true },
            "∈" | "∊" => In { negative: false },
            "∉" => In { negative: true },
            "∋" => Contains { negative: false },
            "∌" => Contains { negative: true },
            "+" => Plus,
            "+=" => PlusAssign,
            "-" => Minus,
            "-=" => MinusAssign,
            "*" => Multiply,
            "/" => Divide,
            "⁒" | "٪" | "%%" => Modulo,
            "%" => Remainder,
            "÷" | "/%" => DivideRemainder,
            "^" => Power,
            "=" => Assign { monadic: false },
            "?=" => Assign { monadic: true },
            "==" => Equal { negative: false },
            "≠" | "!=" => Equal { negative: true },
            "≡" | "===" => StrictlyEqual { negative: false },
            "≢" | "!==" | "=!=" => StrictlyEqual { negative: true },
            ">" => Greater { equal: false },
            "≥" | ">=" => Greater { equal: true },
            "≫" | ">>" => MuchGreater,
            "⋙" | ">>>" => VeryMuchGreater,
            ">>=" => Placeholder,
            "<" => Less { equal: false },
            "≤" | "<=" => Less { equal: true },
            "≪" | "<<" => MuchLess,
            "⋘" | "<<<" => VeryMuchLess,
            "<<=" => Placeholder,
            // logic operators
            "∧" | "&&" => LogicMatrix::And.into(),
            "⊼" => LogicMatrix::Nand.into(),
            "⩟" => LogicMatrix::Xnor.into(), // aka. xand
            "∨" | "||" => LogicMatrix::Or.into(),
            "⊽" => LogicMatrix::Nor.into(),
            "⊻" => LogicMatrix::Xor.into(),
            // range
            "..<" => RangeTo { equal: false },
            "..=" => RangeTo { equal: true },
            // list operator
            "⇴" | "⨵" | "⊕" | "⟴" => Map,

            _ => unimplemented!("{} is a unknown infix operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}
impl<'i> crate::TypeInfixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            "+" => Plus,
            "->" => CovariantType,
            _ => unimplemented!("{} is a unknown infix type operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}
impl<'i> crate::MainSuffixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            "!" => QuickRaise,
            "℃" => Celsius,
            "℉" => Fahrenheit,
            "⁒" | "٪" => DivideByDecimal { power: 1 },
            "%" => DivideByDecimal { power: 2 },
            "‰" => DivideByDecimal { power: 3 },
            "‱" => DivideByDecimal { power: 4 },
            _ => unimplemented!("{} is a unknown suffix operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}

impl<'i> crate::TypeSuffixNode<'i> {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.get_str() {
            "!" => QuickRaise,
            "?" => Celsius,
            _ => unimplemented!("{} is a unknown type suffix operator", self.get_str()),
        };
        OperatorNode { kind: o, span: self.get_range32() }
    }
}
