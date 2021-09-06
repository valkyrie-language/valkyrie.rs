use crate::Identifier;

pub trait IntoField {
    fn as_field_name(&self) -> Identifier;

    fn into_field_data(self) -> ValkyrieFieldData
    where
        Self: Sized,
    {
        let name = self.as_field_name();
        ValkyrieFieldData { name }
    }
}

pub struct ValkyrieFieldData {
    pub name: Identifier,
}
