use super::*;

pub struct ValkyrieInterface {
    /// package∷module∷Interface
    namepath: ValkyrieID,
    /// trait or interfaces
    is_trait: bool,
    /// The functions that are defined in this interfaces
    document: ValkyrieDocument,
}

impl Debug for ValkyrieInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.is_trait {
            true => "ValkyrieTrait",
            false => "ValkyrieInterface",
        };
        let f = &mut f.debug_struct(kind);
        f.field("name", &self.namepath.to_string());
        // f.field("document", &self.document);
        f.finish()
    }
}

impl ValkyrieInterface {
    pub fn new(namepath: ValkyrieID) -> Self {
        Self { namepath, is_trait: false, document: Default::default() }
    }
    pub fn mark_trait(self, is_trait: bool) -> Self {
        Self { is_trait, ..self }
    }
    pub fn with_file(self, file: String) -> Self {
        todo!()
    }
    pub fn with_span(self, span: Range<u32>) -> Self {
        todo!()
    }

    pub fn is_trait(&self) -> bool {
        self.is_trait
    }
    pub fn is_interface(&self) -> bool {
        !self.is_trait
    }
    pub fn namespace(&self) -> &[String] {
        self.namepath.namespace()
    }
    pub fn name(&self) -> ValkyrieString {
        self.namepath.name()
    }
    pub fn full_name(&self) -> &[String] {
        self.namepath.full_name()
    }
}
