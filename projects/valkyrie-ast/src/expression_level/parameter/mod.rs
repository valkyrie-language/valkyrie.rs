mod display;

use super::*;

/// The kind of the parameter node
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterKind {
    /// `a: Type = null`
    #[default]
    Expression,
    /// `T: Trait = ()`
    Generic,
}

/// `micro f(t: Type = default)` or `class F⦓T: Trait = Default⦔`
///
/// ```vk
/// (self, a: Type = default, < ,b: Type = default, ..c: Type, ...d: Type = default>)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParametersList {
    /// The kind of the parameter node
    pub kind: ParameterKind,
    /// The raw string of the number.
    pub this: Option<ParameterTerm>,
    /// Positional only parameters, who appear in left side of `<`
    pub positional: Vec<ParameterTerm>,
    /// Mixed parameters, who appear in the middle of `<` and `>`
    pub mixed: Vec<ParameterTerm>,
    /// Named only parameters, who appear in right side of `>`
    pub named: Vec<ParameterTerm>,
}

/// `<, a: Type = default, >`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParameterMixed {
    /// `positional, 「, mixed`
    LMark(SourceSpan),
    /// `mixed, 」, named`
    RMark(SourceSpan),
    /// `a: Type = default`
    Term(ParameterTerm),
}

/// `#annotation modifier a: Type = default`
///
/// ```vk
/// #annotation modifier term: Type
/// #annotation modifier ..list: Type
/// #annotation modifier ...dict: Type
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterTerm {
    /// The modifiers apply on the parameter
    pub annotations: AnnotationNode,
    /// Unpack level
    pub unpack: u8,
    /// The name
    pub key: IdentifierNode,
    /// The type boundary of the parameter
    pub bound: Option<ExpressionKind>,
    /// The default value of the parameter
    pub default: Option<ExpressionKind>,
}

impl ParametersList {
    /// Create a new parameter list
    pub fn new(kind: ParameterKind) -> Self {
        Self { kind, this: None, positional: vec![], mixed: vec![], named: vec![] }
    }

    ///
    ///
    /// ```vk
    /// (a, b, c)
    /// (<a, b, c>
    /// (a, < , b, c)
    /// ```
    #[must_use = "Error information must be collected"]
    pub fn resolve(&mut self, unsolved: Vec<ParameterMixed>) -> Vec<NyarError> {
        let mut pass_left = true;
        let mut pass_right = false;
        let mut errors = vec![];
        let mut positional = vec![];
        let mut mixed = vec![];
        let mut named = vec![];
        // if has left, start with position only, otherwise mixed
        for item in unsolved.iter() {
            match item {
                ParameterMixed::LMark(_) => {
                    pass_left = false;
                    break;
                }
                _ => {}
            }
        }
        for (index, item) in unsolved.into_iter().enumerate() {
            if index == 0 {
                match item {
                    ParameterMixed::Term(s) if s.is_self() => {
                        self.this = Some(s);
                        continue;
                    }
                    _ => {}
                }
            }
            match item {
                ParameterMixed::LMark(s) => {
                    if pass_left {
                        errors.push(NyarError::syntax_error("dup `「 `", s))
                    }
                    pass_left = true;
                }
                ParameterMixed::RMark(s) => {
                    if pass_right {
                        errors.push(NyarError::syntax_error("dup `」`", s))
                    }
                    pass_right = true;
                }
                ParameterMixed::Term(term) => {
                    if term.is_self() {
                        errors.push(NyarError::syntax_error("self must first", term.key.span))
                    }

                    if pass_right {
                        named.push(term);
                    }
                    else if pass_left {
                        mixed.push(term);
                    }
                    else {
                        positional.push(term);
                    }
                }
            }
        }

        errors
    }
    /// Get the terms in [ParametersList]
    pub fn terms(&self) -> impl Iterator<Item = &ParameterTerm> {
        self.this.iter().chain(self.positional.iter()).chain(self.mixed.iter()).chain(self.named.iter())
    }
}
impl ParameterTerm {
    /// Check if the parameter is a self parameter
    pub fn is_self(&self) -> bool {
        self.key.name.as_ref().eq("self")
    }
}
