use super::*;
use crate::{ValkyrieID, ValkyrieString};

#[derive(Clone, Debug)]
pub struct ValkyrieEnumerate {
    /// package∷module::Interface
    name: ValkyrieID,
    terms: BTreeMap<String, ValkyrieEnumerateItem>,
}

#[derive(Clone, Debug)]
pub struct ValkyrieEnumerateItem {
    name: ValkyrieID,
    /// Can be a progress or a value
    value: Option<ExpressionNode>,
}

impl ValkyrieEnumerate {
    pub fn new(namepath: ValkyrieID) -> Self {
        Self { name: namepath, terms: Default::default() }
    }
    pub fn name(&self) -> ValkyrieString {
        self.name.name()
    }
    pub fn namespace(&self) -> &[String] {
        self.name.namespace()
    }
}

impl ValkyrieEnumerateItem {
    pub fn new(name: ValkyrieID) -> Self {
        Self { name, value: None }
    }
}

impl Valhalla {
    pub(crate) fn load_flags(&mut self, o: FlagDeclaration) -> Validation<()> {
        match o.kind {
            FlagKind::Enumerate => self.load_enum(o),
            FlagKind::Flags => self.load_flag(o),
        }
    }
    fn load_enum(&mut self, o: FlagDeclaration) -> Validation<()> {
        let name = self.load_id(o.name);

        let def = ValkyrieEnumerate { name, terms: Default::default() };

        self.items.insert(def.name.to_string(), ValhallaItem::Enumerate(def));
        Success { value: (), diagnostics: vec![] }
    }
    fn load_flag(&mut self, o: FlagDeclaration) -> Validation<()> {
        todo!()
    }
}
