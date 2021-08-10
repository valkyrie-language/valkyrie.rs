use super::*;
use nyar_error::third_party::WalkDir;
use std::fs::FileType;

impl ResolveState {
    pub fn resolve_package<P>(&mut self, directory: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = directory.as_ref();
        for entry in WalkDir::new(path).contents_first(true) {
            match entry {
                Ok(path) if path.file_type().is_file() => {
                    if let Err(e) = self.resolve_file(path.path()) {
                        println!("error: {:?}\n       {}", path, e);
                        self.push_error(e)
                    }
                }
                Ok(_) => {}
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
        root.to_mir(self, &())
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

impl Hir2Mir for ProgramRoot {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        for statement in self.statements {
            statement.to_mir(store, &())?
        }
        Ok(())
    }
}

impl Hir2Mir for StatementKind {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        match self {
            Self::Nothing => {
                todo!()
            }
            Self::Document(_) => {
                todo!()
            }
            Self::Annotation(_) => {
                todo!()
            }
            Self::Namespace(v) => v.to_mir(store, &())?,
            Self::Import(_) => {
                todo!()
            }
            Self::Class(v) => v.to_mir(store, &())?,
            Self::Union(v) => v.to_mir(store, &())?,
            Self::Enumerate(v) => v.to_mir(store, &())?,
            Self::Trait(v) => v.to_mir(store, &())?,
            Self::Extends(_) => {
                todo!()
            }
            Self::Function(v) => v.to_mir(store, &())?,
            Self::Variable(_) => {
                todo!()
            }
            Self::Guard(_) => {
                todo!()
            }
            Self::While(_) => {
                todo!()
            }
            Self::For(_) => {
                todo!()
            }
            Self::Control(_) => {
                todo!()
            }
            Self::Expression(_) => {
                todo!()
            }
        }
        Ok(())
    }
}

impl Hir2Mir for NamespaceDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        store.namespace.clear();
        match self.path.path.as_slice() {
            // clear current namespace
            [head] if head.name.as_ref().eq("_") => {}
            [head, rest @ ..] => {
                match head.name.as_ref().eq("package") {
                    true => store.namespace.push(store.package.clone()),
                    false => store.namespace.push(head.name.clone()),
                }
                for x in rest {
                    store.namespace.push(x.name.clone())
                }
            }
            _ => {}
        }
        Ok(())
    }
}
