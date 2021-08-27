use crate::{
    helpers::ProgramState,
    traits::YggdrasilNodeExtension,
    utils::{build_annotation_terms, build_constraint, build_modifier_ahead},
};
use nyar_error::Result;
use valkyrie_ast::*;

mod classes;
mod def_fn;
mod enumerate;
mod interface;
mod unions;

mod def_var;
mod extends;

mod constraint;
