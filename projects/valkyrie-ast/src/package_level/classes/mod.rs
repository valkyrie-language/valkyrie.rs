use super::*;
use nyar_error::SourceSpan;

mod display;

mod iters;

/// `class A { }, structure V { }`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassKind {
    /// A class with reference types, type erasure
    Class,
    /// A class with value type, specialized type
    Structure,
}

#[doc = include_str!("readme.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDeclaration {
    /// The name of the class.
    pub name: IdentifierNode,
    /// The kind of class
    pub kind: ClassKind,
    /// The document of the class
    pub annotations: AnnotationNode,
    /// The parameter arguments of the class.
    pub generic: Option<ParametersList>,
    /// The super class of the class.
    pub inherits: Option<String>,
    /// The traits that the class implements.
    pub implements: Vec<String>,
    /// All fields declared in this block, exclude extensions.
    pub terms: Vec<ClassTerm>,
    /// The range of the number.
    pub span: Range<u32>,
}

impl Debug for ClassDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let w = &mut match self.kind {
            ClassKind::Class => f.debug_struct("DefineClass"),
            ClassKind::Structure => f.debug_struct("DefineStructure"),
        };
        // if !self.constraint.is_empty() {
        //     w.field("constraint", &self.constraint);
        // }
        if !self.annotations.is_empty() {
            w.field("annotations", &self.annotations);
        }
        w.field("name", &WrapDisplay::new(&self.name));
        if let Some(s) = &self.generic {
            w.field("generic", s);
        }
        if let Some(s) = &self.inherits {
            w.field("inherits", s);
        }
        if !self.implements.is_empty() {
            w.field("implements", &self.implements);
        }
        if !self.terms.is_empty() {
            w.field("terms", &self.terms);
        }
        w.field("span", &self.span);
        w.finish()
    }
}

/// Valid terms in the class statements
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassTerm {
    /// `@expand {}`
    Macro(ProceduralNode),
    /// `field: Type = default`
    Field(FieldDeclaration),
    /// `method()`
    Method(MethodDeclaration),
    /// `domain { }`
    Domain(DomainDeclaration),
}

/// `object: Trait { ... }`
///
/// Construct an anonymous object
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstructObjectNode {
    /// The super class of the class.
    pub base_classes: Option<String>,
    /// The traits that the class implements.
    pub bounds: Option<ExpressionKind>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `field: Type = default`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldDeclaration {
    /// The name of this field
    pub name: IdentifierNode,
    /// The modifiers of the declaration.
    pub annotations: AnnotationNode,
    /// The type hint of this field
    pub typing: Option<ExpressionKind>,
    /// The default value of this field
    pub default: Option<ExpressionKind>,
    /// The range of the declaration.
    pub span: Range<u32>,
}

/// `#attribute modifier Trait::method(): Return / Effect { ... }`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MethodDeclaration {
    /// The method name which may associated with a trait.
    pub name: IdentifierNode,
    /// The modifiers of the node.
    pub annotations: AnnotationNode,
    /// Thy type parameters of this function
    pub generics: ParametersList,
    /// Thy value parameters of this function
    pub parameters: ParametersList,
    /// `: ReturnType / [EffectType]`
    pub returns: FunctionReturnNode,
    /// `{ body }`
    pub body: Option<StatementBlock>,
    /// The range of the declaration.
    pub span: Range<u32>,
}

impl Debug for MethodDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let w = &mut f.debug_struct("Method");
        if !self.annotations.is_empty() {
            w.field("annotations", &self.annotations);
        }
        w.field("name", &WrapDisplay::new(&self.name));
        if self.generics.terms().count() != 0 {
            w.field("generics", &self.generics);
        }
        if self.parameters.terms().count() != 0 {
            w.field("parameters", &self.parameters);
        }
        if let Some(s) = &self.returns.typing {
            w.field("returns", s);
        }
        if let Some(s) = &self.body {
            w.field("body", s);
        }
        w.field("span", &self.span);
        w.finish()
    }
}

/// `domain { field; method(); domain {} }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DomainDeclaration {
    /// The annotations of the domain
    pub annotations: AnnotationNode,
    /// The range of the declaration.
    pub body: Vec<ClassTerm>,
    /// The range of the declaration.
    pub span: Range<u32>,
}

impl ValkyrieNode for ConstructObjectNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

// impl ClassDeclare {
//     pub fn get_namepath(&self) -> Iter<'_, ValkyrieIdentifier> {
//         self.namepath.iter()
//     }
//     pub fn mut_namepath(&mut self) -> &mut Vec<ValkyrieIdentifier> {
//         &mut self.namepath
//     }
//     pub fn get_modifiers(&self) -> Iter<'_, ValkyrieIdentifier> {
//         self.modifiers.iter()
//     }
//     pub fn mut_modifiers(&mut self) -> &mut Vec<ValkyrieIdentifier> {
//         &mut self.modifiers
//     }
//     pub fn get_statement(&self) -> Iter<'_, ValkyrieASTNode> {
//         self.statements.iter()
//     }
//     pub fn mut_statement(&mut self) -> &mut Vec<ValkyrieASTNode> {
//         &mut self.statements
//     }
//     pub fn to_node(self, file: SourceID, range: &Range<usize>) -> ValkyrieASTNode {
//         ValkyrieASTKind::Class(box self).to_node(file, range)
//     }
// }
