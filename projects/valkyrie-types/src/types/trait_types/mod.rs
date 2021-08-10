use super::*;

pub struct ValkyrieTrait {}

impl Hir2Mir for TraitDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        Ok(())
    }
}
