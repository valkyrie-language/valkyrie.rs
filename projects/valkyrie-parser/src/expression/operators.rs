use super::*;
impl crate::MainPrefixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
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
            _ => unimplemented!("{} is a unknown prefix operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}
impl crate::TypePrefixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
            "!" => Not,
            "+" => CovariantType,
            "-" => ContravariantType,
            "&" => Box,
            _ => unimplemented!("{} is a unknown prefix operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}
impl crate::MainInfixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use valkyrie_ast::LogicMatrix;
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
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

            _ => unimplemented!("{} is a unknown infix operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}
impl crate::TypeInfixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
            "+" => Plus,
            "->" => CovariantType,
            _ => unimplemented!("{} is a unknown infix type operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}
impl crate::MainSuffixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
            "!" => QuickRaise,
            "℃" => Celsius,
            "℉" => Fahrenheit,
            "⁒" | "٪" => DivideByDecimal { power: 1 },
            "%" => DivideByDecimal { power: 2 },
            "‰" => DivideByDecimal { power: 3 },
            "‱" => DivideByDecimal { power: 4 },
            _ => unimplemented!("{} is a unknown suffix operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}

impl crate::TypeSuffixNode {
    pub fn as_operator(&self) -> OperatorNode {
        use ValkyrieOperator::*;
        let o = match self.text.as_str() {
            "!" => QuickRaise,
            "?" => Celsius,
            _ => unimplemented!("{} is a unknown type suffix operator", self.text),
        };
        OperatorNode { kind: o, span: self.span.clone() }
    }
}
