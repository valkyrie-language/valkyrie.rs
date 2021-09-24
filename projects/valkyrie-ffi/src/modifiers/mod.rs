use crate::{ValkyrieError, ValkyrieResult};

/// The mutability of a value. Only reference types have this concept, value types are immutable.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FeatureType {
    Allow,
    Warning,
    Deny,
    Forbid,
}

impl Default for FeatureType {
    fn default() -> Self {
        Self::Allow
    }
}

impl FeatureType {
    pub fn allow(&mut self) -> ValkyrieResult<()> {
        match self {
            FeatureType::Forbid => Err(ValkyrieError::custom("License is forbidden")),
            _ => {
                *self = FeatureType::Allow;
                Ok(())
            }
        }
    }
    pub fn warn(&mut self) -> ValkyrieResult<()> {
        match self {
            FeatureType::Forbid => Err(ValkyrieError::custom("License is forbidden")),
            _ => {
                *self = FeatureType::Warning;
                Ok(())
            }
        }
    }
    pub fn deny(&mut self) -> ValkyrieResult<()> {
        match self {
            FeatureType::Forbid => Err(ValkyrieError::custom("License is forbidden")),
            _ => {
                *self = FeatureType::Deny;
                Ok(())
            }
        }
    }
    pub fn forbid(&mut self) -> ValkyrieResult<()> {
        match self {
            FeatureType::Forbid => Err(ValkyrieError::custom("License is forbidden")),
            _ => {
                *self = FeatureType::Forbid;
                Ok(())
            }
        }
    }
}

/// The mutability of a value. Only reference types have this concept, value types are immutable.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MutableType {
    /// The value cannot be changed
    Immutable,
    /// The value can only be change in the constructor
    Once,
    /// The value can be changed at any time
    Mutable,
}

impl Default for MutableType {
    fn default() -> Self {
        Self::Immutable
    }
}

fn mutable_type_to_str(mutable_type: MutableType) -> &'static str {
    match mutable_type {
        MutableType::Immutable => "immutable",
        MutableType::Once => "once",
        MutableType::Mutable => "mutable",
    }
}

/// The laziness and mutability of value.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InitializeType {
    /// This value must be computed at compile time.
    ///
    /// `const`, `constant`
    Constant,
    /// This value can't be computed at compile time, but must be computed at the beginning of the program.
    ///
    /// `imm`, `immutable`
    Immutable,
    /// This value can be set multiple times
    ///
    /// `mut`,`mutable`
    Mutable,
    /// This value must be computed in the constructor
    ///
    /// `readonly`
    Readonly,
    /// This value supports lazy progress at first call, value cannot be written in the future
    ///
    /// `lazy`
    Lazy,
    /// This is a lazy value, throw `UninitializedError` when accessed before set
    ///
    /// `once`
    OnceLate,
    /// This is a lazy value, and can set multiple times
    ///
    /// `late`
    MutableLate,
}

impl Default for InitializeType {
    fn default() -> Self {
        Self::Immutable
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CurryType {
    /// This is not a curry function
    None,
    /// Curry with single assignment
    Curried,
    /// Curry with repeated assignment
    Deferred,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InlineType {
    /// Never inline this function under any circumstances
    Never,
    /// This is an inline function
    Normal,
    /// This is an inline function, but it is not inlined
    Aggressive,
    /// This is an inline function, but it is not inlined
    Always,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AccessType {
    /// This is a public function
    Public,
    /// This is a private function
    Private,
    /// This is a protected function
    Protected,
    /// This is an internal function
    Internal,
}
