use super::*;

impl<T> ValkyrieType for Vec<T>
where
    T: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.collection.List");
        this.mut_generic_types().push(T::static_type());
        Gc::new(this)
    }
}

impl<T> ValkyrieType for BTreeSet<T>
where
    T: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.collection.SortedSet");
        this.mut_generic_types().push(T::static_type());
        Gc::new(this)
    }
}

impl<K, V> ValkyrieType for BTreeMap<K, V>
where
    K: ValkyrieType,
    V: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
        // ValkyrieValue::List(ValkyrieList::from_iter(self.into_iter().map(|v| v.boxed())))
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.collection.SortedMap");
        this.mut_generic_types().push(K::static_type());
        this.mut_generic_types().push(V::static_type());
        Gc::new(this)
    }
}
