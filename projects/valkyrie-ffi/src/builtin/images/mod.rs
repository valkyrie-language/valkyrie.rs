use super::*;
use image::RgbaImage;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValkyrieImage {
    rgba: RgbaImage,
}

impl ValkyrieType for ValkyrieImage {
    fn boxed(self) -> ValkyrieValue {
        todo!()
    }

    fn dynamic_type(&self) -> Gc<ValkyrieMetaType> {
        // TODO: pixel parameter
        let this = ValkyrieMetaType::new("std.image.Image");
        // this.set_namepath("std.collection.SortedMap");
        // this.mut_generic_types().push(K::static_type());
        // this.mut_generic_types().push(V::static_type());
        Gc::new(this)
    }
}
