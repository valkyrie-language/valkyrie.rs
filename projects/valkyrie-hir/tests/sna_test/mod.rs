use std::fmt::Write;
use valkyrie_hir::{RenameContext, SNAError, SingleNameAssignment,  IndentFormat, IndentContext};
use valkyrie_types::{Location, Variable, STRING_POOL};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expression {
    Let(Box<LetAssign>),
    Variable(Variable),
    Value(i64),
    Apply(Box<FunctionApply>),
    Block(Box<Block>),
    If(Box<IfStatement>),
}

/// let name = init
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LetAssign {
    name: Variable,
    assign: Expression,
}

/// f(a, b, c)(d, e, f)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FunctionApply {
    base: Expression,
    args: Vec<Expression>,
}
/// if cond { a } else { b }
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IfStatement {
    condition: Expression,
    then_branch: Block,
    else_branch: Option<Block>,
}
/// {a; b; c}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    statements: Vec<Expression>,
}

impl SingleNameAssignment for Expression {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        match self {
            Expression::Let(let_assign) => let_assign.rename(ctx),
            Expression::Variable(variable) => variable.rename(ctx),
            Expression::Value(_) => Ok(()),
            Expression::Apply(function_apply) => function_apply.rename(ctx),
            Expression::Block(expressions) => expressions.rename(ctx),
            Expression::If(if_statement) => if_statement.rename(ctx),
        }
    }
}

impl SingleNameAssignment for LetAssign {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.assign.rename(ctx)?;
        let index = ctx.fresh_index(self.name.get_name_key());
        self.name.set_name_index(index);
        ctx.add_to_current_scope(self.name.get_name_key(), self.name.clone());
        Ok(())
    }
}



impl SingleNameAssignment for FunctionApply {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.base.rename(ctx)?;
        for arg in &mut self.args {
            arg.rename(ctx)?;
        }
        Ok(())
    }
}

impl SingleNameAssignment for IfStatement {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        self.condition.rename(ctx)?;
        self.then_branch.rename(ctx)?;
        if let Some(else_branch) = &mut self.else_branch {
            else_branch.rename(ctx)?;
        }
        Ok(())
    }
}

impl SingleNameAssignment for Block {
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError> {
        ctx.push_scope();
        for expr in self.statements.iter_mut() {
            expr.rename(ctx)?;
        }
        ctx.pop_scope();
        Ok(())
    }
}

impl IndentFormat for Expression {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        match self {
            Expression::Let(v) => v.indent_format(context),
            Expression::Variable(v) => v.indent_format(context),
            Expression::Apply(v) => v.indent_format(context),
            Expression::Block(v) => {
                v.indent_format(context)
            }
            Expression::If(v) => v.indent_format(context),
            Expression::Value(v) => v.indent_format(context),
        }
    }
}




impl IndentFormat for LetAssign {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.add_text("let ")?;
        self.name.indent_format(context)?;
        context.add_text(" = ")?;
        self.assign.indent_format(context)?;
        context.new_line()
    }
}

impl IndentFormat for FunctionApply {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        self.base.indent_format(context)?;
        context.add_text("(")?;
        for (index, arg) in self.args.iter().enumerate() {
            if index != 0 {
                context.add_text(", ")?;
            }
            arg.indent_format(context)?;
        }
        context.add_text(")")?;
        Ok(())
    }
}

impl IndentFormat for IfStatement {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.add_text("if ")?;
        self.condition.indent_format(context)?;
        context.new_line()?;
        context.indent("{")?;
        self.then_branch.indent_format(context)?;
        context.dedent("}")?;
        if let Some(else_branch) = &self.else_branch {
            context.add_text("else ")?;
            else_branch.indent_format(context)?;
        }
        Ok(())
    }
}

impl IndentFormat for Block {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.indent("{")?;
        for expr in self.statements.iter() {
            context.new_line()?;
            expr.indent_format(context)?;
        }
        context.dedent("}")
    }
}
#[test]
fn test_block() {
    let mut ctx = RenameContext::default();
    let a = STRING_POOL.encode_static("a");
    let loc = Location::default();

    let raw = Block {
        statements: vec![
            Expression::Let(Box::new(LetAssign {
                name: Variable::new(a, loc),
                assign: Expression::Value(0),
            })),
            Expression::Let(Box::new(LetAssign {
                name: Variable::new(a, loc),
                assign: Expression::Variable(Variable::new(a, loc)),
            })),
            Expression::Variable(Variable::new(a, loc)),
        ],
    };

    let mut new = raw.clone();
    new.rename(&mut ctx).unwrap();

    raw.indent_display().unwrap();
    new.indent_display().unwrap();
}
