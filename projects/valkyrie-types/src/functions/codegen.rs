use super::*;
use nyar_wasm::FunctionType;

impl ConvertTo<FunctionType> for ValkyrieFunction {
    fn convert(&self) -> FunctionType {
        let mut item = FunctionType::new(self.name.to_string());

        item
    }
}
