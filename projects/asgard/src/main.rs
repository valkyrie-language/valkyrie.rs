//! Asgard CLI 入口：`asgard build` / `asgard dev` / `asgard pack` / `asgard plan`。
//!
//! 实现位于 Rust crate `asgard`；用户面对的应用框架名为 **Asgard**。

use std::process::ExitCode;

use clap::Parser;
use miette::Report;
use voa::cli::{AsgardCommands, run};

#[derive(Debug, Parser)]
#[command(name = "asgard", version, about = "Asgard — 跨平台 GUI 应用框架 CLI")]
struct Cli {
    #[command(subcommand)]
    command: AsgardCommands,
}

fn main() -> Result<ExitCode, Report> {
    let cli = Cli::parse();
    run(&cli.command).map_err(Report::from)
}
