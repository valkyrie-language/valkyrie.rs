use crate::{
    expression_level::{
        identifier::{IdentifierNode, NamepathNode},
        string::StringLiteralNode,
    },
    utils::small_range,
    NumberLiteralNode, ValkyrieOperator,
};
use std::{
    fmt::{Display, Formatter, Write},
    ops::Range,
};
mod arithmetic;
pub mod decimal;
pub mod identifier;
pub mod operators;
pub mod string;
