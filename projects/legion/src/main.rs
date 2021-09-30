use std::process::ExitCode;

use clap::{Parser, Subcommand};
use legion::{
    cmds::{
        bootstrap::{run as run_bootstrap, BootstrapArgs},
        build::{run as run_build, BuildArgs},
        run::{run as run_run, RunArgs},
    },
    run_spy, SpyOptions,
};
use miette::Report;

#[derive(Debug, Parser)]
#[command(name = "legion", version, about = "Valkyrie 工作区命令行入口")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 构建项目。
    Build(BuildArgs),
    /// 运行项目产物。
    Run(RunArgs),
    /// 诊断目标产物。
    Spy(SpyOptions),
    /// 自举编译链：seed -> v1 -> v2。
    Bootstrap(BootstrapArgs),
}

fn main() -> Result<ExitCode, Report> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<ExitCode, Report> {
    match cli.command {
        Commands::Build(args) => run_build(&args),
        Commands::Run(args) => run_run(&args),
        Commands::Spy(options) => run_spy(&options),
        Commands::Bootstrap(args) => {
            let result = run_bootstrap(&args)?;
            if result.is_success() {
                println!("自举完成: 成功 {} 个阶段", result.stages_completed.len());
                return Ok(ExitCode::SUCCESS);
            }
            else {
                if let Some((stage, report)) = result.failed_stage {
                    return Err(report.wrap_err(format!("自举失败于阶段 [{}]", stage)));
                }
                Ok(ExitCode::FAILURE)
            }
        }
    }
}
