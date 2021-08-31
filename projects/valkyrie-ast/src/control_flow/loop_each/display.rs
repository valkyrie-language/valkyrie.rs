use super::*;
#[cfg(feature = "pretty-print")]
impl PrettyPrint for LoopEach {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(4);
        terms += theme.keyword("for");
        terms += " ";
        terms += self.pattern.pretty(theme);
        terms += " ";
        terms += theme.keyword("∈");
        terms += " ";
        terms += self.iterator.pretty(theme);
        if let Some(s) = &self.condition {
            terms += " ";
            terms += theme.keyword("if");
            terms += " ";
            terms += s.pretty(theme);
        }
        terms += self.body.pretty(theme);
        terms.into()
    }
}
#[cfg(feature = "lispify")]
impl Lispify for LoopEach {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        let mut lisp = Lisp::new(10);
        lisp += Lisp::keyword("loop/for");
        lisp += Lisp::keyword("iterator") + self.iterator.lispify();
        // lisp += Lisp::keyword("pattern") + self.pattern.lispify();
        if let Some(cond) = &self.condition {
            lisp += Lisp::keyword("condition");
            lisp += cond.lispify();
        }
        lisp += self.body.terms.iter().map(|s| s.lispify()).collect::<Lisp>();
        lisp
    }
}
