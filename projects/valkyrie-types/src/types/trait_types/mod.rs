use super::*;

pub struct ValkyrieTrait {}

impl Hir2Mir for TraitDeclaration {
    type Output = ();

    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output> {
        Ok(())
    }
}
