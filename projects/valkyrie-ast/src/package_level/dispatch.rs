use super::*;
use indentation::{IndentDisplay, IndentFormatter};

impl IndentDisplay for StatementNode {
    fn indent_fmt(&self, f: &mut IndentFormatter) -> core::fmt::Result {
        self.r#type.indent_fmt(f)?;
        if self.eos {
            f.write_str(";")?;
        }
        Ok(())
    }
}

impl IndentDisplay for StatementType {
    fn indent_fmt(&self, f: &mut IndentFormatter) -> core::fmt::Result {
        match self {
            StatementType::Nothing => f.write_str(";;"),
            StatementType::Namespace(_) => {
                todo!()
            }
            StatementType::Import(_) => {
                todo!()
            }
            StatementType::Class(_) => {
                todo!()
            }
            StatementType::While(v) => v.indent_fmt(f),
            StatementType::For(_) => {
                todo!()
            }
            StatementType::Expression(v) => v.indent_fmt(f),
        }
    }
}

impl Display for StatementNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        IndentFormatter::wrap(self, f)
    }
}

impl Display for StatementType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        IndentFormatter::wrap(self, f)
    }
}

impl From<NamespaceDeclarationNode> for StatementType {
    fn from(value: NamespaceDeclarationNode) -> Self {
        Self::Namespace(Box::new(value))
    }
}

impl From<ImportStatementNode> for StatementType {
    fn from(value: ImportStatementNode) -> Self {
        Self::Import(Box::new(value))
    }
}

impl From<ClassDeclarationNode> for StatementType {
    fn from(value: ClassDeclarationNode) -> Self {
        StatementType::Class(Box::new(value))
    }
}

impl From<WhileLoopNode> for StatementType {
    fn from(value: WhileLoopNode) -> Self {
        StatementType::While(Box::new(value))
    }
}

impl From<ForLoopNode> for StatementType {
    fn from(value: ForLoopNode) -> Self {
        StatementType::For(Box::new(value))
    }
}

impl From<ExpressionNode> for StatementType {
    fn from(value: ExpressionNode) -> Self {
        StatementType::Expression(Box::new(value))
    }
}
