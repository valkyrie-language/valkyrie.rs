use super::*;

impl ValkyrieFFI {
    pub(crate) fn export_functions(&self, ty: &Function, namespace: &str, file: &mut File) -> std::io::Result<()> {
        let name = ty.name.to_case(Case::Snake);
        writeln!(file, "ffi(\"{}\", \"{}\")", namespace, ty.name)?;
        writeln!(file, "micro {}(", name)?;
        writeln!(file, "")
    }
}
