use super::*;
use wit_parser::Function;

impl ValkyrieFFI {
    pub(crate) fn export_functions(&self, ty: &Function, file: &mut File) -> std::io::Result<()> {
        file.write(b"function\n\n")?;
        Ok(())
    }
}
