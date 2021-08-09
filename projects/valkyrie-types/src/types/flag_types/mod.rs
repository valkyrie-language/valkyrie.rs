use super::*;

pub struct FlagTypes {}

impl Hir2Mir for FlagDeclaration {
    type Output = ();

    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output> {
        Ok(())
    }
}
