use super::*;

impl AddAssign<ValkyrieFunction> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieFunction) {
        self.items.insert(rhs.function_name.clone(), ModuleItem::Function(rhs));
    }
}
