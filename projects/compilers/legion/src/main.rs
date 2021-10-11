//! Legion 可执行入口（供 `cargo test` 与本地调试；npm 宿主仍经 `vcc-napi` 调用 `run_from_env`）。

fn main() {
    std::process::exit(legion::cli::run_from_env());
}
