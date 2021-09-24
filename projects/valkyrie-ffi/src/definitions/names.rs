use std::fmt::{Debug, Formatter};

/// A unique identifier used to query the valkyrie object
#[derive(Clone, Eq, Hash)]
pub struct ValkyrieName {
    name: String,
}

impl ValkyrieName {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

impl Debug for ValkyrieName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl AsRef<str> for ValkyrieName {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl<T> PartialEq<T> for ValkyrieName
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.name.eq(other.as_ref())
    }
}
