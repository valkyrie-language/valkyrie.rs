use super::*;

impl ValkyrieFFI {
    pub(crate) fn export_functions(&self, ty: &Function, file: &mut File) -> std::io::Result<()> {
        let name = ty.name.to_case(Case::Snake);
        writeln!(file, "micro {}(", name)?;
        writeln!(file, "")
    }
}
