use super::*;

impl Debug for ValkyrieResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let debug = &mut f.debug_struct("Resource");
        debug.field("symbol", &self.resource_name).field("name", &self.wasi_import.name);
        debug.finish()
    }
}

impl Debug for ValkyrieClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let debug = &mut f.debug_struct("Class");
        debug.field("symbol", &WrapDisplay::new(&self.class_name)).field("fields", &self.fields.values());
        debug.finish()
    }
}

impl Debug for ValkyrieField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field").field("name", &self.field_name).field("wasi", &self.wasi_alias).finish()
    }
}

impl AddAssign<ValkyrieClass> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieClass) {
        self.items.insert(rhs.class_name.clone(), ModuleItem::Structure(rhs));
    }
}
impl AddAssign<ValkyrieResource> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieResource) {
        self.items.insert(rhs.resource_name.clone(), ModuleItem::Resource(rhs));
    }
}
