#![feature(lazy_cell)]
#![feature(const_trait_impl)]

mod codegen;

pub use crate::{codegen::*, helpers::ProgramContext};
mod atomic;
mod control_flow;
// mod control_flow;
// mod declarations;
mod declarations;
mod expression;
mod helpers;
mod patterns;
mod statements;
mod string_like;
mod utils;
// mod table;
// mod traits;
// mod helpers;
// mod validation;
//
// pub use crate::{statements::ReplRoot, traits::ThisParser};
// pub use valkyrie_ast::ProgramRoot;
pub use crate::string_like::{big_number::NumberBuilder, formatted::StringFormatterBuilder};
