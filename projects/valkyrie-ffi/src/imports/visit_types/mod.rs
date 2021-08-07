use super::*;

mod visit_functions;

impl ValkyrieFFI {
    pub(crate) fn export_type(&self, ty: &TypeDef, file: &mut File) -> std::io::Result<()> {
        match ty.kind {
            TypeDefKind::Record(_) => {
                file.write(b"Record\n\n")?;
            }
            TypeDefKind::Resource => self.export_resource(ty, file)?,
            TypeDefKind::Handle(_) => {
                file.write(b"Handle\n\n")?;
            }
            TypeDefKind::Flags(_) => {
                file.write(b"Flags\n\n")?;
            }
            TypeDefKind::Tuple(_) => {
                file.write(b"Tuple\n\n")?;
            }
            TypeDefKind::Variant(_) => {
                file.write(b"Variant\n\n")?;
            }
            TypeDefKind::Enum(_) => {
                file.write(b"Enum\n\n")?;
            }
            TypeDefKind::Option(_) => {
                file.write(b"Option\n\n")?;
            }
            TypeDefKind::Result(_) => {
                file.write(b"Result\n\n")?;
            }
            TypeDefKind::List(_) => {
                file.write(b"List\n\n")?;
            }
            TypeDefKind::Future(_) => {
                file.write(b"Future\n\n")?;
            }
            TypeDefKind::Stream(_) => {
                file.write(b"Stream\n\n")?;
            }
            TypeDefKind::Type(v) => {
                writeln!(file, "Type: {:?} {:?}", ty.name, v)?;
            }
            TypeDefKind::Unknown => {
                file.write(b"Unknown\n\n")?;
            }
        }
        Ok(())
    }
    fn export_resource(&self, ty: &TypeDef, file: &mut File) -> std::io::Result<()> {
        let name = match &ty.name {
            Some(s) => s.to_case(Case::UpperCamel),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "missing name"))?,
        };
        writeln!(file, "#resource")?;
        writeln!(file, "#ffi(\"{}\")", name)?;
        writeln!(file, "class {} {{}}", name)?;
        writeln!(file, "")
    }
}
