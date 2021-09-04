use super::*;

impl AddAssign<ValkyrieImportFunction> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieImportFunction) {
        self.items.insert(rhs.function_name.clone(), ModuleItem::External(rhs));
    }
}

impl AddAssign<ValkyrieNativeFunction> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieNativeFunction) {
        self.items.insert(rhs.function_name.clone(), ModuleItem::Function(rhs));
    }
}
