use valkyrie_error::NyarError;
use valkyrie_ast::{helper::NumberInterpreter, NumberLiteralNode};

pub struct NumberBuilder {}

impl NumberInterpreter for NumberBuilder {
    type Output = ();

    fn interpret(&mut self, n: &NumberLiteralNode) -> Result<Self::Output, NyarError> {
        todo!()
    }
}
