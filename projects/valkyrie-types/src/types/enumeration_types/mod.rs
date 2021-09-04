use super::*;

#[derive(Debug)]
pub struct ValkyrieEnumeration {
    pub enumeration_name: Identifier,
    pub enumerations: IndexMap<Arc<str>, ValkyrieSemanticNumber>,
}

impl AddAssign<ValkyrieEnumeration> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieEnumeration) {
        self.items.insert(rhs.enumeration_name.clone(), NamespaceItem::Enums(rhs));
    }
}

impl Mir2Lir for ValkyrieEnumeration {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut enumerations = Vec::with_capacity(self.enumerations.len());
        for value in self.enumerations.values() {
            enumerations.push(value.to_lir(graph, context)?);
        }
        *graph += WasiEnumeration { symbol: self.enumeration_name.clone(), enumerations };

        Ok(())
    }
}
