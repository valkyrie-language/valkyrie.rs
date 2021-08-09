use super::*;

// noinspection DuplicatedCode
#[cfg(feature = "pretty-print")]
impl PrettyPrint for GenericCallNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(3);
        terms += "⦓";
        // terms += theme.join(self.terms.clone(), ", ");
        terms += "⦔";
        terms.into()
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for GenericCallTerm {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(3);
        // if let Some(k) = &self.term.key {
        //     terms += theme.generic(k.name.to_owned());
        //     terms += ": ";
        // }
        // terms += self.term.value.pretty(theme);
        terms.into()
    }
}

// noinspection DuplicatedCode
#[cfg(feature = "pretty-print")]
impl PrettyPrint for ParametersList {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(3);
        terms += "⦓";
        // terms += theme.join(self.terms.clone(), ", ");
        terms += "⦔";
        terms.into()
    }
}
#[cfg(feature = "lispify")]
impl Lispify for ParametersList {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        todo!()
    }
}
