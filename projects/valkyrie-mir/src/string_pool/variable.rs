use std::fmt::{Display, Formatter};
use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    /// The raw name of the variable.
    name_key: Spur,
    /// The unique index of the variable.
    hidden_index: u32,
    location: Location,
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        if self.hidden_index != 0 {
            f.write_char('_')?;
            f.write_str(&self.hidden_index.to_string())?;
        }
        Ok(())
    }
}

impl Variable {
    pub fn new(name: Spur, location: Location) -> Self {
        Self { name_key: name, hidden_index: 0, location }
    }
    pub fn get_name_index(&self) -> u32 {
        self.hidden_index
    }
    pub fn get_name_key(&self) -> Spur {
        self.name_key
    }
    pub fn set_name_index(&mut self, index: u32) {
        self.hidden_index = index;
    }
}

impl AsRef<str> for Variable  {
    fn as_ref(&self) -> &str {
        STRING_POOL.decode_string(&self.name_key)
    }
}