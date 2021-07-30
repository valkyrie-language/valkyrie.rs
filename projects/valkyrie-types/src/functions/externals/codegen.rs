use super::*;
use nyar_wasm::{ArrayType, WasmSymbol, WasmType};

impl ConvertTo<ExternalType> for ValkyrieExternalFunction {
    fn convert(&self) -> ExternalType {
        let mut item = ExternalType::new(self.bind_module.clone(), self.bind_name.clone());
        item += WasmSymbol::from(self.name.to_string());
        for param in self.inputs.iter() {
            match param {
                ParameterTerm::LMark => {}
                ParameterTerm::RMark => {}
                ParameterTerm::Single { key, bound, .. } => {
                    let mut p = WasmParameter::new(key.name.as_str());
                    match bound {
                        Some(s) => p.type_hint = s.convert(),
                        None => {
                            panic!("Run type infer first")
                        }
                    }
                    item += p
                }
                // `..args: Type`
                ParameterTerm::UnpackList { key, bound, .. } => {
                    let mut p = WasmParameter::new(key.name.as_str());
                    let array = match bound {
                        Some(s) => ArrayType::new(format!("{}.{}", self.name, key.name), s.convert()),
                        None => ArrayType::new(format!("{}.{}", self.name, key.name), WasmType::Any { nullable: true }),
                    };
                    p.type_hint = WasmType::Array(Box::new(array));
                    item += p
                }
                ParameterTerm::UnpackDict { .. } => {
                    unimplemented!()
                }
            }
        }
        item
    }
}
