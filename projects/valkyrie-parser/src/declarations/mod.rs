use crate::{
    helpers::ProgramState,
    traits::YggdrasilNodeExtension,
    utils::{build_annotation_terms, build_constraint, build_modifier_ahead},
};
use valkyrie_ast::*;
use valkyrie_types::Result;

mod classes;
mod def_fn;
mod enumerate;
mod interface;
mod unions;

mod def_var;
mod extends;

mod constraint;
