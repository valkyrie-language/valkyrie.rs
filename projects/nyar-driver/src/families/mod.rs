use miette::{miette, Result};
use nyar::TargetBackendFamily;

use crate::{DriverCompileReport, DriverCompileRequest};

mod clr;
mod jvm;
mod native;
mod wasm;

trait BundledFamilyCompiler: Sync {
    fn family(&self) -> TargetBackendFamily;

    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport>;
}

static CLR_COMPILER: clr::ClrFamilyCompiler = clr::ClrFamilyCompiler;
static JVM_COMPILER: jvm::JvmFamilyCompiler = jvm::JvmFamilyCompiler;
static WASM_COMPILER: wasm::WasmFamilyCompiler = wasm::WasmFamilyCompiler;
static NATIVE_COMPILER: native::NativeFamilyCompiler = native::NativeFamilyCompiler;

static FAMILY_COMPILERS: [&dyn BundledFamilyCompiler; 4] = [&CLR_COMPILER, &JVM_COMPILER, &WASM_COMPILER, &NATIVE_COMPILER];

pub(crate) fn compile(request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
    let Some(compiler) = FAMILY_COMPILERS.iter().copied().find(|compiler| compiler.family() == request.backend_family)
    else {
        return Err(miette!("当前 `nyar-driver` 尚未接入 {:?} 后端家族", request.backend_family));
    };
    compiler.compile(request)
}

#[cfg(test)]
mod tests {
    use nyar::TargetBackendFamily;

    use super::FAMILY_COMPILERS;

    #[test]
    fn registers_supported_bundled_families_once() {
        let families = FAMILY_COMPILERS.iter().map(|compiler| compiler.family()).collect::<Vec<_>>();
        let mut unique = Vec::new();
        for family in &families {
            if !unique.contains(family) {
                unique.push(*family);
            }
        }

        assert_eq!(families.len(), unique.len());
        assert_eq!(unique.len(), 4);
        assert!(unique.contains(&TargetBackendFamily::Clr));
        assert!(unique.contains(&TargetBackendFamily::Jvm));
        assert!(unique.contains(&TargetBackendFamily::Wasm));
        assert!(unique.contains(&TargetBackendFamily::Native));
    }
}
