use super::*;

use polars::prelude::LazyFrame;

#[derive(Clone)]
pub struct ValkyrieDataFrame {
    any_frame: LazyFrame,
}

impl Debug for ValkyrieDataFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ValkyrieType for ValkyrieDataFrame {
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        todo!()
    }
}
