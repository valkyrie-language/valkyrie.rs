use nyar::{abstractions::CanonicalTarget, PublishFormat, RunnerFamily, RunnerSelector, TargetHostKind};

#[test]
fn derives_profile_for_clr() {
    let profile = CanonicalTarget::clr().to_profile(None);
    assert_eq!(profile.host_kind, TargetHostKind::DotNet);
    assert_eq!(profile.runner_family(), RunnerFamily::Clr);
    assert_eq!(profile.artifact_policy.default_publish_format, PublishFormat::Directory);
}

#[test]
fn parses_runner_selector_from_family_and_canonical_target() {
    assert_eq!("clr".parse::<RunnerSelector>().unwrap(), RunnerSelector::Family(RunnerFamily::Clr));
    assert_eq!("wasm32-unknown-browser-wasm".parse::<RunnerSelector>().unwrap(), RunnerSelector::Canonical(CanonicalTarget::wasm()));
}

#[test]
fn checks_publish_format_support() {
    let profile = CanonicalTarget::wasm().to_profile(None);
    assert!(profile.supports_publish_format(PublishFormat::WebApp));
    assert!(!profile.supports_publish_format(PublishFormat::Jar));
}
