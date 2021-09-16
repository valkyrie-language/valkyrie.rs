use super::*;

impl Debug for WasmIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identifier").field("path", &self.namespace.join("∷")).field("name", &self.name).finish()
    }
}

impl Display for WasmIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for path in &self.namespace {
            f.write_str(path)?;
            if f.alternate() { f.write_str("::")? } else { f.write_str("∷")? }
        }
        f.write_str(&self.name)
    }
}
