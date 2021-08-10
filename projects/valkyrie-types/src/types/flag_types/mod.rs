use super::*;

pub struct FlagTypes {}

impl Hir2Mir for FlagDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        Ok(())
    }
}
