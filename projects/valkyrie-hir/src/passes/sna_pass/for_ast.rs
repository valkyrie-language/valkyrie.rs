use crate::{RenameContext, SNAError, SingleNameAssignment};
use lasso::Spur;
use valkyrie_ast::{
    ArgumentTerm, ArgumentsList, ExpressionKind, IdentifierNode, LetBindNode, NamePathNode, StatementBlock, StatementKind,
    TupleNode,
};
use valkyrie_types::Variable;

impl SingleNameAssignment for StatementBlock {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        ctx.push_scope();
        for term in self.terms.iter_mut() {
            match term.rename(ctx) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        ctx.pop_scope();
        Ok(())
    }
}

impl SingleNameAssignment for StatementKind {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        match self {
            StatementKind::Variable(v) => v.rename(ctx),
            _ => Ok(()),
        }
    }
}
impl SingleNameAssignment for LetBindNode {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        if let Some(o) = self.body.as_mut() {
            o.rename(ctx)?
        }

        // let index = ctx.fresh_index(self.name.get_name_key());
        // self.name.set_name_index(index);
        // ctx.add_to_current_scope(self.name.get_name_key(), self.name.clone());

        Ok(())
    }
}

impl SingleNameAssignment for ExpressionKind {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        match self {
            ExpressionKind::Symbol(v) => v.rename(ctx),
            ExpressionKind::Tuple(v) => v.rename(ctx),
            // ExpressionKind::Array(v) => { v.rename(ctx)}
            _ => Ok(()),
        }
    }
}

impl SingleNameAssignment for TupleNode {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.terms.rename(ctx)
    }
}

impl SingleNameAssignment for ArgumentsList {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        for i in self.terms.iter_mut() {
            i.rename(ctx)?
        }
        Ok(())
    }
}

impl SingleNameAssignment for ArgumentTerm {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.value.rename(ctx)
    }
}
impl SingleNameAssignment for NamePathNode {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        match self.path.first_mut() {
            Some(s) => s.rename(ctx),
            None => Err(SNAError::EmptyPath { location: ctx.file.with_range(self.span.get_range()) }),
        }
    }
}
impl SingleNameAssignment for IdentifierNode {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.shadow_index = ctx.get_current_index(&Spur::from(self.name), self.span.get_range())?;
        Ok(())
    }
}
impl SingleNameAssignment for Variable {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        let index = ctx.get_current_index(&self.get_name_key(), Default::default())?;
        self.set_name_index(index);
        Ok(())
    }
}
