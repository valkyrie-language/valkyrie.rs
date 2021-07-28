#![allow(unused)]

use crate::ProgramNode;
use nyar_error::{Failure, FileCache, FileID, NyarError, Success, Validation};
use std::{ops::AddAssign, str::FromStr};
use valkyrie_ast::ProgramRoot;

pub struct ProgramContext {
    pub file: FileID,
}

pub(crate) struct ProgramState {
    pub file: FileID,
    errors: Vec<NyarError>,
}

impl AddAssign<NyarError> for ProgramState {
    fn add_assign(&mut self, rhs: NyarError) {
        self.errors.push(rhs)
    }
}

pub(crate) struct LooperState {
    pub id: usize,
    pub stack: Vec<usize>,
}

impl ProgramState {
    pub fn new(file: FileID) -> Self {
        Self { file, errors: vec![] }
    }
    pub fn add_error<E>(&mut self, error: E)
    where
        E: Into<NyarError>,
    {
        self.errors.push(error.into())
    }
}

impl ProgramContext {
    pub fn parse(&self, cache: &mut FileCache) -> Validation<ProgramRoot> {
        let text = cache.fetch(&self.file)?.to_string();
        let mut state = ProgramState::new(self.file);
        match ProgramNode::from_str(&text) {
            Ok(o) => match o.build(&mut state) {
                Ok(value) => Success { value, diagnostics: state.errors },
                Err(e) => Failure { fatal: e.with_file(self.file), diagnostics: state.errors },
            },
            Err(e) => Failure { fatal: NyarError::from(e).with_file(self.file), diagnostics: state.errors },
        }
    }
    pub fn parse_custom(&self, text: &str) -> Validation<ProgramRoot> {
        let mut state = ProgramState::new(self.file);
        match ProgramNode::from_str(&text) {
            Ok(o) => match o.build(&mut state) {
                Ok(value) => Success { value, diagnostics: state.errors },
                Err(e) => Failure { fatal: e.with_file(self.file), diagnostics: state.errors },
            },
            Err(e) => Failure { fatal: NyarError::from(e).with_file(self.file), diagnostics: state.errors },
        }
    }
}
