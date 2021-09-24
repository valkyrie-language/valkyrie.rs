use super::*;

impl ValkyrieType for ValkyrieDict {
    fn boxed(self) -> ValkyrieValue {
        ValkyrieValue::Dict(self)
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.List");

        Gc::new(this)
    }
}

impl ValkyrieType for () {
    fn boxed(self) -> ValkyrieValue {
        ValkyrieValue::Dict(Default::default())
    }

    fn static_type() -> Gc<ValkyrieMetaType> {
        primitive_type("std.primitive.Tuple")
    }
    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        primitive_type("std.primitive.Tuple")
    }
}

impl<T1> ValkyrieType for (T1,)
where
    T1: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.Tuple");
        this.mut_generic_types().push(T1::static_type());
        Gc::new(this)
    }
}

impl<T1, T2> ValkyrieType for (T1, T2)
where
    T1: ValkyrieType,
    T2: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.Tuple");
        this.mut_generic_types().push(T1::static_type());
        this.mut_generic_types().push(T2::static_type());
        Gc::new(this)
    }
}

impl<T1, T2, T3> ValkyrieType for (T1, T2, T3)
where
    T1: ValkyrieType,
    T2: ValkyrieType,
    T3: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.Tuple");
        this.mut_generic_types().push(T1::static_type());
        this.mut_generic_types().push(T2::static_type());
        this.mut_generic_types().push(T3::static_type());
        Gc::new(this)
    }
}

impl<T1, T2, T3, T4> ValkyrieType for (T1, T2, T3, T4)
where
    T1: ValkyrieType,
    T2: ValkyrieType,
    T3: ValkyrieType,
    T4: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.Tuple");
        this.mut_generic_types().push(T1::static_type());
        this.mut_generic_types().push(T2::static_type());
        this.mut_generic_types().push(T3::static_type());
        this.mut_generic_types().push(T4::static_type());
        Gc::new(this)
    }
}

impl<T1, T2, T3, T4, T5> ValkyrieType for (T1, T2, T3, T4, T5)
where
    T1: ValkyrieType,
    T2: ValkyrieType,
    T3: ValkyrieType,
    T4: ValkyrieType,
    T5: ValkyrieType,
{
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        let mut this = ValkyrieMetaType::default();
        this.set_namepath("std.primitive.Tuple");
        this.mut_generic_types().push(T1::static_type());
        this.mut_generic_types().push(T2::static_type());
        this.mut_generic_types().push(T3::static_type());
        this.mut_generic_types().push(T4::static_type());
        this.mut_generic_types().push(T5::static_type());
        Gc::new(this)
    }
}
