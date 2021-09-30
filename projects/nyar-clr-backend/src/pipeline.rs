//! `CLR` 高层编译管线。
//!
//! 这里收口从前端声明信息、`LIR` lowering 到 `MSIL` 重写、sidecar 输出与
//! `PE/COFF` 产物落地的完整流程，避免上层驱动理解 `CLR` 内部细节。

use std::{collections::BTreeMap, fs, path::Path};

use miette::{IntoDiagnostic, Result};
use nyar::{
    backends::{clr::ClrImageKind, CompilationOptions, TargetCodeGenBackend},
    packaging::ArtifactSet,
    TargetLoweringLane,
};
use valkyrie_compiler::{hir::HirModule, lir::LirModule};
use valkyrie_parser::{DeclarationStatement, FunctionDeclaration, ValkyrieRoot};

use crate::{
    build_clr_method_signature,
    msil::{MsilInstruction, MsilInstructionOperand, MsilMethodRef, MsilMethodSignature, MsilModule, MsilOpcode, MsilTextWriter},
    resolve_clr_import_ref, write_dotnet_runtime_config, ClrBinaryBackend, ClrBinaryBackendInput, ClrLirLoweringLane,
};

/// `CLR` 高层编译请求。
#[derive(Debug)]
pub struct ClrCompileRequest<'a> {
    /// 前端语法根。
    pub parser_root: &'a ValkyrieRoot,
    /// 前端 `HIR`。
    pub hir_module: &'a HirModule,
    /// 已选择 lane 的 `LIR`。
    pub lir_module: LirModule,
    /// 输出目录。
    pub output_dir: &'a Path,
    /// 逻辑产物名。
    pub artifact_name: &'a str,
    /// 是否输出 `MSIL` sidecar。
    pub emit_msil: bool,
    /// 是否生成 runtime config。
    pub generate_runtime_config: bool,
    /// 通用编译选项。
    pub options: &'a CompilationOptions,
}

/// `CLR` 高层编译结果。
#[derive(Debug, Default)]
pub struct ClrCompileReport {
    /// 产物集合。
    pub artifacts: ArtifactSet,
    /// 入口符号。
    pub entry_symbol: Option<String>,
    /// 主要物理入口文件。
    pub artifact_file_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionSpec {
    symbol: String,
    signature: MsilMethodSignature,
    is_entry_point: bool,
    external_ref: Option<MsilMethodRef>,
}

/// 使用 `CLR` bundled backend 完成完整编译流程。
pub fn compile_clr_bundle(request: ClrCompileRequest<'_>) -> Result<ClrCompileReport> {
    let function_specs = collect_function_specs(request.parser_root, request.hir_module)?;
    let lane = ClrLirLoweringLane::new();
    let lowered = lane.lower_partition(request.lir_module)?;
    let mut msil_module = lowered.input;
    msil_module.assembly.name = request.artifact_name.to_string();
    rewrite_msil_module(&mut msil_module, &function_specs);

    let image_kind = infer_image_kind(&msil_module);
    let artifact_file_name = format!("{}.{}", request.artifact_name, image_kind.file_extension());
    let entry_symbol = function_specs.values().find(|spec| spec.is_entry_point).map(|spec| spec.symbol.clone());

    if request.emit_msil {
        write_msil_sidecar(request.output_dir, request.artifact_name, &artifact_file_name, &msil_module)?;
    }

    let backend = ClrBinaryBackend::new();
    let input = ClrBinaryBackendInput { module: msil_module, output_dir: request.output_dir.to_path_buf(), image_kind: Some(image_kind) };
    backend.validate(&input)?;
    let artifacts = backend.compile(input, request.options)?;

    if request.generate_runtime_config {
        write_dotnet_runtime_config(request.output_dir, request.artifact_name)?;
    }

    Ok(ClrCompileReport { artifacts, entry_symbol, artifact_file_name })
}

fn collect_function_specs(root: &ValkyrieRoot, hir_module: &HirModule) -> Result<BTreeMap<String, FunctionSpec>> {
    let mut hir_by_name = BTreeMap::new();
    for function in &hir_module.functions {
        hir_by_name.insert(function.name.as_str().to_string(), function);
    }

    let mut declared_functions = Vec::new();
    for statement in &root.statements {
        let DeclarationStatement::Function(function) = statement
        else {
            continue;
        };

        let Some(hir_function) = hir_by_name.get(function.name.as_str())
        else {
            continue;
        };
        declared_functions.push((function, *hir_function));
    }

    let entry_symbol = select_entry_symbol(&declared_functions);
    let mut specs = BTreeMap::new();
    for (function, hir_function) in declared_functions {
        let is_entry_point = entry_symbol.as_deref() == Some(function.name.as_str());
        let spec = FunctionSpec {
            symbol: function.name.clone(),
            signature: build_clr_method_signature(hir_function, is_entry_point)?,
            is_entry_point,
            external_ref: resolve_clr_import_ref(hir_function)?,
        };
        specs.insert(spec.symbol.clone(), spec);
    }
    Ok(specs)
}

fn select_entry_symbol<T>(declared_functions: &[(&FunctionDeclaration, T)]) -> Option<String> {
    declared_functions
        .iter()
        .find(|(function, _)| has_main_attribute(function) && function.name == "main")
        .map(|(function, _)| function.name.clone())
        .or_else(|| declared_functions.iter().find(|(function, _)| has_main_attribute(function)).map(|(function, _)| function.name.clone()))
        .or_else(|| declared_functions.iter().find(|(function, _)| function.name == "main").map(|(function, _)| function.name.clone()))
}

fn has_main_attribute(function: &FunctionDeclaration) -> bool {
    function.annotations.attributes().any(|attribute| attribute.name.parts.last().is_some_and(|name| name == "main"))
}

fn rewrite_msil_module(module: &mut MsilModule, function_specs: &BTreeMap<String, FunctionSpec>) {
    module.global_methods.retain(|method| function_specs.get(&method.method.name).is_none_or(|spec| spec.external_ref.is_none()));

    for method in &mut module.global_methods {
        if let Some(spec) = function_specs.get(&method.method.name) {
            method.method.signature = spec.signature.clone();
            method.is_entry_point = spec.is_entry_point;
        }
        rewrite_call_instructions(&mut method.instructions, function_specs, &module.assembly.name);
    }
}

fn rewrite_call_instructions(instructions: &mut Vec<MsilInstruction>, function_specs: &BTreeMap<String, FunctionSpec>, module_name: &str) {
    let mut rewritten = Vec::with_capacity(instructions.len());
    let mut index = 0;

    while index < instructions.len() {
        let current = &instructions[index];
        let next = instructions.get(index + 1);
        if let (
            MsilOpcode::Ldsfld,
            Some(MsilInstructionOperand::Symbol(symbol)),
            Some(MsilInstruction { opcode: MsilOpcode::Call, operand: None, .. }),
        ) = (current.opcode, current.operand.as_ref(), next)
        {
            if let Some(method_ref) = method_ref_for_symbol(symbol, function_specs, module_name) {
                rewritten.push(MsilInstruction {
                    label: current.label.clone(),
                    opcode: MsilOpcode::Call,
                    operand: Some(MsilInstructionOperand::Method(method_ref)),
                });
                index += 2;
                continue;
            }
        }

        if let (MsilOpcode::Call, Some(MsilInstructionOperand::Symbol(symbol))) = (current.opcode, current.operand.as_ref()) {
            if let Some(method_ref) = method_ref_for_symbol(symbol, function_specs, module_name) {
                rewritten.push(MsilInstruction {
                    label: current.label.clone(),
                    opcode: MsilOpcode::Call,
                    operand: Some(MsilInstructionOperand::Method(method_ref)),
                });
                index += 1;
                continue;
            }
        }

        rewritten.push(current.clone());
        index += 1;
    }

    *instructions = rewritten;
}

fn method_ref_for_symbol(symbol: &str, function_specs: &BTreeMap<String, FunctionSpec>, module_name: &str) -> Option<MsilMethodRef> {
    let short_name = symbol.rsplit("::").next().unwrap_or(symbol);
    let spec = function_specs.get(short_name)?;
    if let Some(external_ref) = &spec.external_ref {
        return Some(external_ref.clone());
    }

    Some(MsilMethodRef { owner: Some(module_name.to_string()), name: spec.symbol.clone(), signature: spec.signature.clone() })
}

fn infer_image_kind(module: &MsilModule) -> ClrImageKind {
    let has_entry_point = module.global_methods.iter().any(|method| method.is_entry_point)
        || module.types.iter().flat_map(|type_def| type_def.methods.iter()).any(|method| method.is_entry_point);
    ClrImageKind::infer(has_entry_point)
}

fn write_msil_sidecar(output_dir: &Path, artifact_name: &str, artifact_file_name: &str, module: &MsilModule) -> Result<()> {
    fs::create_dir_all(output_dir).into_diagnostic().map_err(|error| error.wrap_err(format!("创建输出目录失败 {}", output_dir.display())))?;
    let msil_path = output_dir.join(format!("{}.msil", artifact_name));
    let source = MsilTextWriter::new().write_module_with_file_name(module, artifact_file_name);
    fs::write(&msil_path, source).into_diagnostic().map_err(|error| error.wrap_err(format!("写入 MSIL 文件失败 {}", msil_path.display())))?;
    Ok(())
}
