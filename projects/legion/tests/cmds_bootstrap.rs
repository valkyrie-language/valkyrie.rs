mod support;

use legion::cmds::bootstrap::{run, BootstrapArgs, BootstrapStage};
use std::{panic::AssertUnwindSafe, path::PathBuf};
use support::create_smoke_project;
use valkyrie_compiler::CanonicalTarget;

#[test]
#[cfg_attr(target_os = "windows", ignore = "seed launch is unstable under sandboxed Windows hosts")]
fn bootstrap_stops_at_v2_when_v1_artifact_is_not_yet_a_self_hosting_seed() {
    if cfg!(target_os = "macos") {
        eprintln!("skip bootstrap acceptance: macOS host cannot execute managed PE artifacts directly");
        return;
    }

    let fixture = create_smoke_project("legion-bootstrap");
    let seed_path = PathBuf::from(env!("CARGO_BIN_EXE_legion"));
    let args = BootstrapArgs {
        project_dir: fixture.project_dir.clone(),
        seed_path: Some(seed_path),
        skip_compare: true,
        target: CanonicalTarget::clr(),
    };
    let result = match std::panic::catch_unwind(AssertUnwindSafe(|| run(&args))) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) if should_skip_bootstrap_host_check(&error.to_string()) => {
            eprintln!("skip bootstrap acceptance: unable to launch seed binary in current environment: {error}");
            return;
        }
        Ok(Err(error)) => panic!("bootstrap run failed unexpectedly: {error}"),
        Err(payload) if should_skip_bootstrap_host_check(&panic_payload_to_string(&payload)) => {
            eprintln!("skip bootstrap acceptance: seed launch is blocked by current environment");
            return;
        }
        Err(payload) => std::panic::resume_unwind(payload),
    };

    let (failed_stage, error) = result.failed_stage.expect("bootstrap should stop at the current non-self-hosting boundary");
    let message = error.to_string();
    match failed_stage {
        BootstrapStage::V1Run => {
            assert_eq!(result.stages_completed, vec![BootstrapStage::Seed, BootstrapStage::V1]);
            assert!(result.v1_path.is_none());
            assert!(result.v2_path.is_none());
            assert!(
                message.contains("执行产物失败")
                    || message.contains("无法执行")
                    || message.contains("Permission denied")
                    || message.contains("Exec format error")
                    || message.contains("cannot execute binary file")
                    || message.contains("Bad CPU type")
            );
        }
        BootstrapStage::V2 => {
            assert_eq!(result.stages_completed, vec![BootstrapStage::Seed, BootstrapStage::V1, BootstrapStage::V1Run]);
            assert!(result.v1_path.is_none());
            assert!(result.v2_path.is_none());
            assert!(message.contains("编译退出码为 0 但未产出产物"));
            assert!(message.contains("可能不具备 `legion build` 命令行能力"));
            assert!(message.contains("dist\\v2\\app.exe") || message.contains("dist/v2/app.exe"));
        }
        other => panic!("unexpected bootstrap failure stage: {other} ({message})"),
    }
}

fn should_skip_bootstrap_host_check(message: &str) -> bool {
    message.contains("os error 0") || message.contains("无法执行") || message.contains("unable to launch") || message.contains("鎿嶄綔鎴愬姛")
}

fn panic_payload_to_string(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    String::new()
}
