use super::*;
use nyar_error::third_party::WalkDir;

impl ResolveState {
    pub fn resolve_package<P>(&mut self, directory: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = directory.as_ref();
        for entry in WalkDir::new(path).contents_first(true) {
            match entry {
                Ok(path) => {
                    if !path.file_type().is_file() {
                        continue;
                    }
                    if !(path.file_name().to_string_lossy().ends_with("vk")
                        || path.file_name().to_string_lossy().ends_with("valkyrie"))
                    {
                        continue;
                    }

                    if let Err(e) = self.resolve_file(path.path()) {
                        println!("error: {:?}\n       {}", path, e);
                        self.push_error(e)
                    }
                }

                Err(e) => self.push_error(e),
            }
        }
        Ok(())
    }
    pub fn resolve_file<P>(&mut self, file: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let source = self.sources.load_local(file)?;
        let root = ProgramContext { file: source }.parse(&mut self.sources);
        match root {
            Success { value, diagnostics } => {
                self.errors.extend(diagnostics);
                self.resolve_ast(value)
            }
            Failure { fatal, diagnostics } => {
                self.errors.extend(diagnostics);
                Err(fatal)
            }
        }
    }
    /// Parse a fetch text from the source cache
    pub fn resolve_ast(&mut self, root: ProgramRoot) -> Result<()> {
        root.to_mir(self, ())
    }
    pub fn push_error<E: Into<NyarError>>(&mut self, e: E) {
        self.errors.push(e.into())
    }
    pub fn show_errors(&mut self) {
        let errors = take(&mut self.errors);
        for error in errors {
            match error.as_report().print(&self.sources) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
