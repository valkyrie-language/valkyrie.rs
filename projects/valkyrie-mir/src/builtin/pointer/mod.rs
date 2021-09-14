use super::*;

impl<T> ValkyrieType for Arc<T>
where
    T: ValkyrieType,
{
    #[track_caller]
    fn boxed(self) -> ValkyrieValue {
        panic!("Gc<T> can't be not boxed")
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        self.as_ref().dynamic_type()
    }
}
