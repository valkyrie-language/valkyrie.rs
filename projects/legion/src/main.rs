use std::{ffi::OsString, process::ExitCode};

use clap::{Parser, Subcommand};
use legion::{
    SpyOptions,
    cmds::{
        audit::{AuditArgs, run as run_audit},
        bench::{BenchArgs, run as run_bench},
        bootstrap::{BootstrapArgs, run as run_bootstrap},
        build::{BuildArgs, run as run_build},
        check::{CheckArgs, run as run_check},
        clean::{CleanArgs, run as run_clean},
        cov::{CovArgs, run as run_cov},
        doc::{DocArgs, run as run_doc},
        fmt::{FmtArgs, run as run_fmt},
        install::{AddArgs, InstallArgs, RemoveArgs, UpdateArgs, run as run_install, run_add, run_remove, run_update},
        lint::{LintArgs, run as run_lint},
        login::{LoginArgs, LogoutArgs, WhoamiArgs, run_login, run_logout, run_whoami},
        publish::{PublishArgs, run as run_publish},
        registry::{RegistryArgs, run as run_registry},
        run::{RunArgs, run as run_run},
        search::{InfoArgs, SearchArgs, run as run_search, run_info},
        test::{TestArgs, run as run_test},
        vendor::{VendorArgs, run as run_vendor},
    },
    forward::try_forward_companion,
    run_spy,
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
    /// 检查项目可编译性。
    Check(CheckArgs),
    /// 清理构建产物目录。
    Clean(CleanArgs),
    /// 运行项目产物。
    Run(RunArgs),
    /// 诊断目标产物。
    Spy(SpyOptions),
    /// 自举编译链：seed -> v1 -> v2。
    Bootstrap(BootstrapArgs),
    /// 生成项目文档站点（用户文档 + hub）。
    Doc(DocArgs),
    /// 运行测试并生成 HTML 报告。
    Test(TestArgs),
    /// 收集语法覆盖率并生成 HTML 报告。
    #[command(visible_alias = "coverage")]
    Cov(CovArgs),
    /// 运行基准测试并生成 HTML 报告。
    #[command(visible_alias = "benchmark")]
    Bench(BenchArgs),
    /// 格式化代码。
    #[command(visible_alias = "format")]
    Fmt(FmtArgs),
    /// 运行前端语义 lint（不生成后端产物）。
    Lint(LintArgs),
    /// 发布包到注册表。
    Publish(PublishArgs),
    /// 登录注册表（与 npm / deno 等官方 CLI 凭据互通）。
    Login(LoginArgs),
    /// 退出注册表登录。
    Logout(LogoutArgs),
    /// 查看当前登录用户。
    Whoami(WhoamiArgs),
    /// 安装依赖。
    Install(InstallArgs),
    /// 添加依赖。
    Add(AddArgs),
    /// 移除依赖。
    Remove(RemoveArgs),
    /// 更新依赖。
    Update(UpdateArgs),
    /// 搜索注册表中的包。
    Search(SearchArgs),
    /// 查看包信息。
    Info(InfoArgs),
    /// 注册表认证管理。
    Vendor(VendorArgs),
    /// 审计依赖漏洞与许可证。
    Audit(AuditArgs),
    /// 管理注册表源 endpoint。
    Registry(RegistryArgs),
}

fn main() {
    // Debug builds use a comparatively small process stack on Windows. The
    // self-hosting compiler walks deeply nested source/AST structures, so run
    // the CLI on an explicitly sized stack to keep debug and release behavior
    // aligned.
    let result = std::thread::Builder::new()
        .name("legion-main".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(run_main)
        .expect("failed to spawn legion main thread")
        .join()
        .expect("legion main thread panicked");
    let exit_code = match result {
        Ok(code) => {
            if code == ExitCode::SUCCESS {
                0
            }
            else {
                1
            }
        }
        Err(report) => {
            eprintln!("{report:?}");
            1
        }
    };
    std::process::exit(exit_code);
}

fn run_main() -> Result<ExitCode, Report> {
    let argv: Vec<OsString> = std::env::args_os().collect();
    if let Some(code) = try_forward_companion(&argv) {
        return Ok(code);
    }

    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<ExitCode, Report> {
    match cli.command {
        Commands::Build(args) => run_build(&args),
        Commands::Check(args) => run_check(&args),
        Commands::Clean(args) => run_clean(&args),
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
        Commands::Doc(args) => run_doc(&args),
        Commands::Test(args) => run_test(&args),
        Commands::Cov(args) => run_cov(&args),
        Commands::Bench(args) => run_bench(&args),
        Commands::Fmt(args) => run_fmt(&args),
        Commands::Lint(args) => run_lint(&args),
        Commands::Publish(args) => run_publish(&args),
        Commands::Login(args) => run_login(&args),
        Commands::Logout(args) => run_logout(&args),
        Commands::Whoami(args) => run_whoami(&args),
        Commands::Install(args) => run_install(&args),
        Commands::Add(args) => run_add(&args),
        Commands::Remove(args) => run_remove(&args),
        Commands::Update(args) => run_update(&args),
        Commands::Search(args) => run_search(&args),
        Commands::Info(args) => run_info(&args),
        Commands::Vendor(args) => run_vendor(&args),
        Commands::Audit(args) => run_audit(&args),
        Commands::Registry(args) => run_registry(&args),
    }
}
