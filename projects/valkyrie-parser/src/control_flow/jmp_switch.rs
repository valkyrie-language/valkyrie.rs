use super::*;
use crate::traits::YggdrasilNodeExtension;

impl<'i> crate::SwitchStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<SwitchStatement> {
        Ok(SwitchStatement { patterns: self.match_block().build(ctx), span: self.get_range32() })
    }
}
