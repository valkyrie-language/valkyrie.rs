use super::*;

impl ConvertTo<FieldType> for ValkyrieField {
    fn convert(&self) -> FieldType {
        let mut field = FieldType::new(self.symbol.to_string());
        field.readonly = self.get_readonly();
        match &self.typing {
            Some(s) => field.set_type(s.convert()),
            None => panic!("Run type infer first"),
        }
        field
    }
}
