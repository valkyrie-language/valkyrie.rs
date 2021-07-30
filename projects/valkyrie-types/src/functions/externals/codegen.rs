use super::*;
use nyar_wasm::{ArrayType, ExternalFunctionType, WasmSymbol, WasmType};
use std::str::FromStr;

impl ConvertTo<ExternalFunctionType> for ValkyrieExternalFunction {
    fn convert(&self) -> ExternalFunctionType {
        let external = WasmExternalName::from_str(&self.bind_module).unwrap();
        let mut item = ExternalFunctionType::new(external, self.bind_name.clone());
        item.local_name = WasmSymbol::from(self.name.to_string());
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
