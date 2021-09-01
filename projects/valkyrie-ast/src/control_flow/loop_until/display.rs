use super::*;
#[cfg(feature = "pretty-print")]
impl PrettyPrint for LoopUntil {
    /// ```vk
    /// # inline style
    /// while a || b || c { ... }
    ///
    /// # block style
    /// while a
    ///     || b && c
    ///     && d || e
    /// {
    ///    ...
    /// }
    /// ```
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(4);
        terms += theme.keyword("while");
        terms += " ";
        terms += self.condition.pretty(theme);
        terms += self.then.pretty(theme);
        terms.into()
    }
}

#[cfg(feature = "lispify")]
impl Lispify for LoopUntil {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        let mut lisp = Lisp::new(self.then.terms.len() + 1);
        match self.keyword {
            WhileLoopKind::While => {
                lisp += Lisp::keyword("while");
            }
            WhileLoopKind::Until => {
                lisp += Lisp::keyword("until");
            }
        }
        lisp += self.condition.lispify();
        for term in &self.then.terms {
            lisp += term.lispify();
        }
        lisp
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for UntilConditionNode {
    /// ```vk
    /// # inline style
    /// a || b || c
    ///
    /// # block style
    ///
    /// a
    ///   || b && c
    ///   && d || e
    /// ```
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        match self {
            UntilConditionNode::NotCase(e) => theme.keyword("case"),
            UntilConditionNode::Expression(e) => e.pretty(theme),
        }
    }
}

#[cfg(feature = "lispify")]
impl Lispify for UntilConditionNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        match self {
            UntilConditionNode::Unconditional => Lisp::keyword("true"),
            UntilConditionNode::NotCase => Lisp::keyword("case"),
            UntilConditionNode::Expression(v) => v.lispify(),
        }
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for OtherwiseStatement {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(10);
        terms += PrettyTree::Hardline;
        terms += theme.keyword("otherwise");
        terms += " ";
        terms += "{";
        terms += PrettyTree::Hardline;
        terms += theme.join(self.terms.clone(), PrettyTree::Hardline).indent(4);
        terms += PrettyTree::Hardline;
        terms += "}";
        terms.into()
    }
}
