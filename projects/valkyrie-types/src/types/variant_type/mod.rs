use super::*;
use shredder::Scanner;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieVariantType {
    namepath: ValkyrieID,
    generics: Vec<Gc<ValkyrieMetaType>>,
    variants: Vec<ValkyrieStructure>,
}

unsafe impl GcSafe for ValkyrieVariantType {}

unsafe impl Scan for ValkyrieVariantType {
    fn scan(&self, scanner: &mut Scanner<'_>) {
        todo!()
    }
}

impl ValkyrieVariantType {
    pub fn new(namepath: &str) -> Self {
        todo!()
    }
    pub fn mut_generics(&mut self) -> &mut Vec<Gc<ValkyrieMetaType>> {
        &mut self.generics
    }
}

impl Default for ValkyrieVariantType {
    fn default() -> Self {
        todo!()
    }
}
