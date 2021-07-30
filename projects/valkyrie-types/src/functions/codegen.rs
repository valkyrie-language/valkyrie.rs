use super::*;

impl Hir2Mir for FunctionDeclaration {
    fn to_mir(self, ctx: &mut ModuleResolver) -> Result<Self::Output> {
        let symbol = self.name.as_namespace_symbol(&ctx.namespace);
        for attr in self.annotations.attributes.terms {
            let name = attr.path.to_string();
            match name.as_str() {
                "ffi" => {
                    let (module, name) = attr.as_ffi()?;
                    let mut external = ValkyrieExternalFunction::new(symbol.clone());
                    external.set_path(module, name);
                    for i in self.parameters.terms {
                        external += i
                    }
                    ctx.items.insert(symbol.to_string(), ModuleItem::External(external));
                    return Ok(());
                }
                _ => {
                    println!("Unhanded: {name}")
                }
            }
        }

        ctx.items.insert(symbol.to_string(), ModuleItem::Function(ValkyrieFunction::new(symbol)));
        Ok(())
    }
}

impl ConvertTo<FunctionType> for ValkyrieFunction {
    fn convert(&self) -> FunctionType {
        let mut item = FunctionType::new(self.name.to_string());

        item
    }
}
