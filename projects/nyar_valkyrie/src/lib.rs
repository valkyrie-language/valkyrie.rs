use clap::Parser;
use nyar_runtime::NyarVM;
use std::ffi::OsString;

mod variant;

#[derive(Parser)]
#[command(version, about, long_about = include_str!("readme.md"))]
pub struct Application {
    /// Doc comment
    #[arg(long, short = 'C')]
    directory: Option<OsString>,
}

impl Application {
    pub async fn run(&self) {
        let p1 = NyarVM::load_wast("test").await;
    }
}
