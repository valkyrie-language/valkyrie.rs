from pathlib import Path
import re

ROOT = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm")

# host_imports
(ROOT / "host_imports.rs").write_text(
    r'''//! Collect declared host imports / string-output edges for wasm shells.
use crate::FragmentSubmission;
use nyar::{ExternalCallArgument, ExternalCallEdge, QualifiedName};
use std_data::binary::wasm::{WasmOpcode, encode_i32_const};

use super::super::interop::{wasi_host_import_target, wasm_host_import_target};

pub(super) fn wasm_impl_return_offset_body(text_offset: u32) -> Vec<u8> {
    let mut body = vec![0];
    encode_i32_const(i32::try_from(text_offset).unwrap(), &mut body);
    WasmOpcode::End.encode(&mut body);
    body
}

pub(super) fn collect_string_output_for_wasm_host(submission: &FragmentSubmission) -> Vec<u8> {
    collect_string_output(submission, |link| wasm_host_import_target(link).is_some())
}

fn collect_string_output(submission: &FragmentSubmission, matches_host_import: impl Fn(&nyar::ExternalImportLink) -> bool) -> Vec<u8> {
    let mut output = Vec::new();
    for operation in submission.entry_operation.iter().chain(submission.exported_operations.iter()) {
        for edge in outgoing_external_call_edges(operation, &submission.external_call_edges) {
            let Some(link) = submission.external_import_links.get(&edge.callee_symbol)
            else {
                continue;
            };
            if !matches_host_import(link) {
                continue;
            }
            let Some(value) = edge.arguments.iter().find_map(string_literal_argument)
            else {
                continue;
            };
            output.extend_from_slice(value.as_bytes());
        }
        if !output.is_empty() {
            break;
        }
    }
    output
}

pub(super) fn first_wasm_host_import_target(submission: &FragmentSubmission) -> Option<(String, String)> {
    first_host_link_for_output(submission, wasm_host_import_target).and_then(|link| {
        let target = wasm_host_import_target(link)?;
        Some((target.module.to_string(), target.field.to_string()))
    })
}

pub(super) fn declared_wasi_host_import_targets(submission: &FragmentSubmission) -> Vec<(String, String)> {
    let mut imports = Vec::new();
    for link in submission.external_import_links.values() {
        let Some(target) = wasi_host_import_target(link)
        else {
            continue;
        };
        let import_target = (target.module.to_string(), target.function.to_string());
        if !imports.contains(&import_target) {
            imports.push(import_target);
        }
    }
    imports
}

fn first_host_link_for_output<'a, T>(
    submission: &'a FragmentSubmission,
    matches_host_import: impl Fn(&'a nyar::ExternalImportLink) -> Option<T>,
) -> Option<&'a nyar::ExternalImportLink> {
    for operation in submission.entry_operation.iter().chain(submission.exported_operations.iter()) {
        for edge in outgoing_external_call_edges(operation, &submission.external_call_edges) {
            let Some(link) = submission.external_import_links.get(&edge.callee_symbol)
            else {
                continue;
            };
            if matches_host_import(link).is_some() {
                return Some(link);
            }
        }
    }
    submission.external_import_links.values().find(|link| matches_host_import(link).is_some())
}

fn string_literal_argument(argument: &ExternalCallArgument) -> Option<String> {
    match argument {
        ExternalCallArgument::StringLiteral(value) => Some(value.clone()),
    }
}

fn outgoing_external_call_edges<'a>(operation: &QualifiedName, edges: &'a [ExternalCallEdge]) -> Vec<&'a ExternalCallEdge> {
    edges.iter().filter(|edge| &edge.caller == operation).collect()
}
''',
    encoding="utf-8",
)

# js_glue
js = (ROOT / "host" / "js_glue.rs").read_text(encoding="utf-8")
if "std_data::binary::wasm" not in js:
    js = js.replace(
        "use crate::{FragmentSubmission, nyar_backend_wasi::WasmBinaryModule};",
        """use crate::{FragmentSubmission, nyar_backend_wasi::WasmBinaryModule};
use std_data::binary::wasm::{VALTYPE_I32, WasmExternalKind, WasmOpcode, encode_i32_const, encode_call};""",
    )
    js = js.replace("&[0x7F]", "&[VALTYPE_I32]")
    js = js.replace("&[], &[0x7F]", "&[], &[VALTYPE_I32]")
    js = js.replace('("main", 0x00,', '("main", WasmExternalKind::Func.as_u8(),')
    js = js.replace(
        """fn js_glue_main_body(output: &[u8]) -> Vec<u8> {
    let mut body = vec![0x00];
    for byte in output {
        body.push(0x41);
        encode_sleb128_i32(i32::from(*byte), &mut body);
        body.push(0x10);
        encode_uleb128(0, &mut body);
    }
    body.push(0x41);
    encode_sleb128_i32(0, &mut body);
    body.push(0x0B);
    body
}""",
        """fn js_glue_main_body(output: &[u8]) -> Vec<u8> {
    let mut body = vec![0];
    for byte in output {
        encode_i32_const(i32::from(*byte), &mut body);
        encode_call(0, &mut body);
    }
    encode_i32_const(0, &mut body);
    WasmOpcode::End.encode(&mut body);
    body
}""",
    )
    # also fix wasm_function_type(&[0x7F], &[])
    js = re.sub(r"wasm_function_type\(\&\[0x7F\]", "wasm_function_type(&[VALTYPE_I32]", js)
    js = re.sub(r"wasm_function_type\(\&\[\], \&\[0x7F\]\)", "wasm_function_type(&[], &[VALTYPE_I32])", js)
    (ROOT / "host" / "js_glue.rs").write_text(js, encoding="utf-8")
print("js_glue", "std_data" in (ROOT / "host" / "js_glue.rs").read_text(encoding="utf-8"))

# wasi_cm
wasi = (ROOT / "host" / "wasi_cm.rs").read_text(encoding="utf-8")
if "std_data::binary::wasm" not in wasi:
    wasi = wasi.replace(
        "use nyar::WitnessSubmission;",
        """use nyar::WitnessSubmission;
use std_data::binary::wasm::{VALTYPE_I32, WasmExternalKind, WasmOpcode};""",
    )
    wasi = wasi.replace("0x7F", "VALTYPE_I32")
    wasi = wasi.replace("vec![0x00, 0x0B]", "vec![0, WasmOpcode::End.as_u8()]")
    wasi = wasi.replace(", 0x00,", ", WasmExternalKind::Func.as_u8(),")
    wasi = wasi.replace(", 0x02,", ", WasmExternalKind::Memory.as_u8(),")
    (ROOT / "host" / "wasi_cm.rs").write_text(wasi, encoding="utf-8")
print("wasi_cm", "std_data" in (ROOT / "host" / "wasi_cm.rs").read_text(encoding="utf-8"))

# suspend.rs - valtypes + export kinds + common noop bodies
sus = (ROOT / "suspend.rs").read_text(encoding="utf-8")
if "std_data::binary::wasm" not in sus:
    sus = sus.replace(
        "use nyar::{SuspendFunctionArtifact, SuspendStateArtifact, WitnessSubmission};",
        """use nyar::{SuspendFunctionArtifact, SuspendStateArtifact, WitnessSubmission};
use std_data::binary::wasm::{
    BLOCKTYPE_EMPTY, VALTYPE_I32, WasmExternalKind, WasmOpcode, encode_br_if, encode_i32_const, encode_i32_eqz, encode_i32_load,
    encode_i32_store, encode_local_get, encode_unreachable,
};""",
    )
    sus = re.sub(r"\b0x7F\b", "VALTYPE_I32", sus)
    sus = re.sub(r',\s*0x00,', ", WasmExternalKind::Func.as_u8(),", sus)
    sus = re.sub(r',\s*0x02,', ", WasmExternalKind::Memory.as_u8(),", sus)
    sus = sus.replace("vec![0x00, 0x0B]", "vec![0, WasmOpcode::End.as_u8()]")
    sus = sus.replace("vec![0x00, 0x00, 0x0B]", "vec![0, WasmOpcode::Unreachable.as_u8(), WasmOpcode::End.as_u8()]")
    (ROOT / "suspend.rs").write_text(sus, encoding="utf-8")
print("suspend remaining hex", len(re.findall(r"0x[0-9A-Fa-f]+", (ROOT / "suspend.rs").read_text(encoding="utf-8"))))
print("done")
