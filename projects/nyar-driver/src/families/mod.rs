use miette::{miette, Result};
use nyar::{BackendCandidate, BackendSelector, PartitionBackendRequirement};

use crate::{DriverCompileReport, DriverCompileRequest};

mod clr;
mod jvm;
mod native;
mod wasm;

trait BundledFamilyCompiler: Sync {
    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport>;
}

struct DriverCompilerRegistration {
    name: &'static str,
    priority: u16,
    supports: fn(&PartitionBackendRequirement) -> bool,
    compiler: &'static dyn BundledFamilyCompiler,
}

impl DriverCompilerRegistration {
    fn candidate(&self, requirement: &PartitionBackendRequirement) -> Option<BackendCandidate> {
        (self.supports)(requirement).then(|| BackendCandidate {
            name: self.name.to_string(),
            requirement: requirement.clone(),
            priority: self.priority,
        })
    }
}

static CLR_COMPILER: clr::ClrFamilyCompiler = clr::ClrFamilyCompiler;
static JVM_COMPILER: jvm::JvmFamilyCompiler = jvm::JvmFamilyCompiler;
static WASM_COMPILER: wasm::WasmFamilyCompiler = wasm::WasmFamilyCompiler;
static NATIVE_COMPILER: native::NativeFamilyCompiler = native::NativeFamilyCompiler;

static DRIVER_COMPILERS: [DriverCompilerRegistration; 4] = [
    DriverCompilerRegistration { name: "clr-binary", priority: 100, supports: clr::supports_requirement, compiler: &CLR_COMPILER },
    DriverCompilerRegistration { name: "jvm-binary", priority: 100, supports: jvm::supports_requirement, compiler: &JVM_COMPILER },
    DriverCompilerRegistration { name: "wasm-binary", priority: 100, supports: wasm::supports_requirement, compiler: &WASM_COMPILER },
    DriverCompilerRegistration { name: "native-binary", priority: 100, supports: native::supports_requirement, compiler: &NATIVE_COMPILER },
];

pub(crate) fn compile(request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
    let mut selector = BackendSelector::default();
    let mut matched_compilers = Vec::new();
    for registration in DRIVER_COMPILERS.iter() {
        if let Some(candidate) = registration.candidate(&request.requirement) {
            selector.register(candidate);
            matched_compilers.push((registration.name, registration.compiler));
        }
    }

    let Some(selected) = selector.select(&request.requirement)
    else {
        return Err(miette!("当前 `nyar-driver` 尚未接入该后端需求：{:?}", request.requirement));
    };
    let Some((_, compiler)) = matched_compilers.into_iter().find(|(name, _)| *name == selected.name)
    else {
        return Err(miette!("后端候选 `{}` 已注册，但找不到对应的 driver compiler", selected.name));
    };
    compiler.compile(request)
}
