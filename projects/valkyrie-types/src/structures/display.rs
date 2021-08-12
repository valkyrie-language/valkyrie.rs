use super::*;

impl Debug for ValkyrieClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.wasi_import {
            Some(name) => {
                let debug = &mut f.debug_struct("Resource");
                debug.field("symbol", &self.symbol).field("name", &name);
                debug.finish()
            }
            None => {
                let debug = &mut f.debug_struct("Class");
                debug.field("symbol", &WrapDisplay::new(&self.symbol)).field("fields", &self.fields.values());
                debug.finish()
            }
        }
    }
}
impl Debug for ValkyrieField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field").field("name", &self.field_name).field("wasi", &self.wasi_alias).finish()
    }
}

impl AddAssign<ValkyrieClass> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieClass) {
        self.items.insert(rhs.symbol.clone(), ModuleItem::Structure(rhs));
    }
}
