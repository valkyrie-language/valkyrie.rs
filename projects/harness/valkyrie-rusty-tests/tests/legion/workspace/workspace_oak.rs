//! Migrated from `src/oak.rs` (`tests`).

use data_text_core::{TextError, TextErrorKind};
use rusty_valkyrie::legion::workspace::oak::*;
use std::{fs, path::Path};

#[test]
fn source_point_span_preserves_oak_offset() {
    let error = TextError::new(TextErrorKind::SyntaxError { message: "expected field".to_string(), offset: 7, source_id: None });
    assert_eq!(source_point_span(&error), Some(7..8));
}
