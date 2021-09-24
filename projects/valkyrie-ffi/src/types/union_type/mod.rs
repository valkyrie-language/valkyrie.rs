use super::*;

pub struct ValkyrieUnionType {
    terms: Vec<ValkyrieMetaType>,
}

impl Default for ValkyrieUnionType {
    fn default() -> Self {
        todo!()
    }
}

impl ValkyrieType for ValkyrieUnionType {
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        todo!()
    }
}
