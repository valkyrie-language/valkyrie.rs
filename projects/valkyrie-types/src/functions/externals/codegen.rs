use super::*;

impl ConvertTo<ExternalType> for ValkyrieExternalFunction {
    fn convert(&self) -> ExternalType {
        let mut item = ExternalType::new(self.bind_module.clone(), self.bind_name.clone());

        item
    }
}
