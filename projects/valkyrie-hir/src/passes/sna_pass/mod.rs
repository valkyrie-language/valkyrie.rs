use lasso::Spur;
use std::{collections::HashMap, ops::Range};
use valkyrie_types::{FileName, Location, STRING_POOL, Variable};

mod for_ast;

/// Give each function in the expression a unique name.
pub trait SingleNameAssignment {
    ///
    /// Before renaming:
    ///
    /// ```
    /// let mut a = 1;
    /// {
    ///     a = 2;
    ///     let mut a = 3;
    ///     {
    ///         let a = a;
    ///         a // a = 3
    ///     }
    ///     a = 4
    /// }
    /// a // 2
    /// ```
    ///
    /// After renaming:
    ///
    /// ```
    /// let mut a_1 = 1;
    /// {
    ///     a_1 = 2;
    ///     let mut a_2 = 3;
    ///     {
    ///         let a_3 = a_2;
    ///         a_3 // a = 3
    ///     }
    ///     a_2 = 4
    /// }
    /// a_1 // 2
    /// ```
    fn rename(&mut self, ctx: &mut RenameContext) -> Result<(), SNAError>;
}



// The SNAError enum represents different types of errors that can occur
// during the single name assignment process.
#[derive(Debug)]
pub enum SNAError {
    EmptyPath {
        location: Location
    },
    // The Undefined error indicates that a variable is undefined.
    Undefined { variable: Spur, location: Location },
}

// The RenameContext struct is used to manage the renaming of variables
// during the single name assignment process.
#[derive(Default)]
pub struct RenameContext {
    file: FileName,
    // The counter is a HashMap that keeps track of the number of times
    // a variable name has been used, allowing for the generation of
    // unique variable names.
    counter: HashMap<Spur, u32>,
    // The scope_stack is a Vec of HashMaps, where each HashMap represents
    // a scope in the code. This is used to keep track of the current
    // variable names and their corresponding renamed versions.
    scope_stack: Vec<HashMap<Spur, u32>>,
    errors: Vec<SNAError>,
}

impl RenameContext {
    // Create a new index when `let-bind`
    pub fn fresh_index(&mut self, base: Spur) -> u32 {
        let count = self.counter.entry(base).or_insert(0);
        *count += 1;
        *count
    }

    // The push_scope method adds a new scope to the scope_stack.
    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    // The pop_scope method removes the top scope from the scope_stack.
    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    // The get_current_index method retrieves the current name for the
    // given variable name, searching through the scope_stack in reverse
    // order to find the most recent definition.
    pub fn get_current_index(&self, name: &Spur, range: Range<u32>) -> Result<u32, SNAError> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(renamed) = scope.get(name) {
                return Ok(renamed.clone());
            }
        }
        Err(SNAError::Undefined { variable: *name, location: self.file.with_range(range) })
    }

    // The add_to_current_scope method adds a new variable name mapping
    // to the current scope.
    pub fn add_to_current_scope(&mut self, original: Spur, renamed: Variable) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(original, renamed.get_name_index());
        }
    }
    pub fn finish(self) -> Vec<SNAError> {
        self.errors
    }
}
