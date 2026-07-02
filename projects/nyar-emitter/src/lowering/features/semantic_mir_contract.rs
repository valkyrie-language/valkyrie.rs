//! Semantic MIR boundary contracts shared by all backend lowerers.
//!
//! This module deliberately validates only semantic metadata completeness. It
//! does not choose JVM, WASM, CLR, or WASI representations; those decisions
//! belong to backend-local preparation after this gate succeeds.

use crate::{
    FragmentSubmission,
    executable_provider::{ExecutableFunction, ExecutableInstructionKind, ExecutableOperand},
};
use nyar::QualifiedName;
use nyar_types::{
    Constant, IntrinsicOpcode, NyarType,
    executable::{TextConversionSemantics, TextEncoding, TextProjectionBoundary},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticMirContractError {
    pub code: &'static str,
    pub function: String,
    pub location: String,
    pub detail: String,
}

/// Stable, backend-independent observation format used by the paired
/// Rust/Valkyrie conformance runner.
pub(crate) fn observation(case_id: &str, result: Result<(), &SemanticMirContractError>) -> String {
    match result {
        Ok(()) => format!("{case_id}|accept||"),
        Err(error) => format!("{case_id}|reject|{}|{}", error.code, observation_site(&error.location)),
    }
}

fn observation_site(location: &str) -> &'static str {
    if location == "entry" {
        "entry"
    }
    else if location.contains("instruction") {
        "instruction"
    }
    else if location.contains("block") {
        "block"
    }
    else if location.contains("layout") || location.contains("sum") {
        "module"
    }
    else {
        "function"
    }
}

pub(crate) fn validate_submission(submission: &FragmentSubmission) -> Result<(), SemanticMirContractError> {
    validate_nominal_sums(submission)?;
    validate_aggregate_layouts(submission)?;

    let Some(executable) = &submission.executable
    else {
        return Ok(());
    };

    for operation in executable.operations() {
        let Some(view) = executable.get_function(&operation)
        else {
            return Err(SemanticMirContractError {
                code: "SMIR003",
                function: operation.to_string(),
                location: "operation".to_string(),
                detail: "executable operation has no function payload".to_string(),
            });
        };
        validate_static_call_resolution(submission, &view.function)?;
        validate_function(&view.function)?;
        validate_aggregate_field_contracts(submission, &view.function)?;
    }
    Ok(())
}

fn validate_aggregate_layouts(submission: &FragmentSubmission) -> Result<(), SemanticMirContractError> {
    for layout in &submission.aggregate_layouts.layouts {
        if layout.name.is_empty() || layout.align == 0 || layout.size == 0 {
            return Err(SemanticMirContractError {
                code: "SMIR010",
                function: layout.name.clone(),
                location: "aggregate layout".to_string(),
                detail: "aggregate layout requires name, non-zero size, and non-zero alignment".to_string(),
            });
        }
        if layout.fields.iter().any(|field| field.name.is_empty())
            || layout.fields.iter().enumerate().any(|(index, field)| layout.fields[..index].iter().any(|prior| prior.name == field.name))
        {
            return Err(SemanticMirContractError {
                code: "SMIR010",
                function: layout.name.clone(),
                location: "aggregate layout".to_string(),
                detail: "aggregate layout fields require unique non-empty names".to_string(),
            });
        }
    }
    Ok(())
}

/// Aggregate field instructions are Semantic MIR, not a request for a backend
/// to discover an owner from a field spelling or physical object carrier.
fn validate_aggregate_field_contracts(submission: &FragmentSubmission, function: &ExecutableFunction) -> Result<(), SemanticMirContractError> {
    for block in &function.blocks {
        for (index, instruction) in block.instructions.iter().enumerate() {
            if let ExecutableInstructionKind::StructNew { type_name, storage, layout_id, fields } = &instruction.kind {
                let location = format!("block {} instruction {index}", block.id.0);
                let Some(layout_id) = layout_id
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "aggregate construction has no layout id".to_string(),
                    });
                };
                let Some(layout) = submission.aggregate_layouts.layouts.iter().find(|layout| layout.id == *layout_id)
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "aggregate construction references an unknown layout".to_string(),
                    });
                };
                let output_type = instruction.output.and_then(|output| function.value_types.get(&output));
                let fields_match = fields.len() == layout.fields.len()
                    && fields.iter().all(|(name, value)| {
                        layout.fields.iter().find(|field| field.name == *name).is_some_and(
                            |field| matches!(value, ExecutableOperand::Value(value) if function.value_types.get(value) == Some(&field.ty)),
                        )
                    });
                if layout.name != *type_name
                    || layout.storage != *storage
                    || output_type != Some(&NyarType::Named(nyar::Identifier::new(type_name)))
                    || !fields_match
                {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "aggregate construction contract disagrees with declared layout metadata".to_string(),
                    });
                }
                continue;
            }
            if let ExecutableInstructionKind::SumNew { sum_type, variant, payload_type, payload } = &instruction.kind {
                let location = format!("block {} instruction {index}", block.id.0);
                let Some(sum) = submission.sum_types.iter().find(|sum| sum.name == *sum_type)
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: "sum construction references an undeclared sum".to_string(),
                    });
                };
                let Some(declared) = sum.variants.iter().find(|candidate| candidate.name == *variant)
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: "sum construction references an undeclared variant".to_string(),
                    });
                };
                let payload_value_type = match payload {
                    Some(ExecutableOperand::Value(value)) => function.value_types.get(value),
                    Some(_) => None,
                    None => None,
                };
                if instruction.output.and_then(|output| function.value_types.get(&output))
                    != Some(&NyarType::Named(nyar::Identifier::new(sum_type)))
                    || payload_type.as_ref() != declared.payload_type.as_ref()
                    || match (&declared.payload_type, payload, payload_value_type) {
                        (None, None, _) => false,
                        (Some(expected), Some(ExecutableOperand::Value(_)), Some(actual)) => actual != expected,
                        _ => true,
                    }
                {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: "sum construction contract disagrees with declared sum metadata".to_string(),
                    });
                }
                continue;
            }
            if let ExecutableInstructionKind::SumPayloadGet { sum_type, variant, payload_type, object } = &instruction.kind {
                let location = format!("block {} instruction {index}", block.id.0);
                let Some(sum) = submission.sum_types.iter().find(|sum| sum.name == *sum_type)
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: format!("sum payload extraction references undeclared sum `{sum_type}`"),
                    });
                };
                let Some(declared) =
                    sum.variants.iter().find(|candidate| candidate.name == *variant).and_then(|candidate| candidate.payload_type.as_ref())
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: format!("sum `{sum_type}` has no payload-bearing variant `{variant}`"),
                    });
                };
                let output_type = instruction.output.and_then(|output| function.value_types.get(&output));
                let receiver_type = match object {
                    ExecutableOperand::Value(value) => function.value_types.get(value),
                    _ => None,
                };
                if declared != payload_type
                    || output_type != Some(payload_type)
                    || receiver_type != Some(&NyarType::Named(nyar::Identifier::new(sum_type)))
                {
                    return Err(SemanticMirContractError {
                        code: "SMIR006",
                        function: function.symbol.clone(),
                        location,
                        detail: "sum payload extraction contract disagrees with declared sum metadata".to_string(),
                    });
                }
                continue;
            }
            let (field, storage, layout_id, value) = match &instruction.kind {
                ExecutableInstructionKind::FieldGet { field, storage, layout_id, .. } => (field, storage, layout_id, None),
                ExecutableInstructionKind::FieldSet { field, storage, layout_id, value, .. } => (field, storage, layout_id, Some(value)),
                _ => continue,
            };
            let location = format!("block {} instruction {index}", block.id.0);
            let Some(layout_id) = layout_id
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: "aggregate field access has no layout id".to_string(),
                });
            };
            let Some(layout) = submission.aggregate_layouts.layouts.iter().find(|layout| layout.id == *layout_id)
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: format!("aggregate field access references unknown layout {layout_id}"),
                });
            };
            if layout.storage != *storage {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: "aggregate field access storage differs from its declared layout".to_string(),
                });
            }
            let Some(declared) = layout.fields.iter().find(|candidate| candidate.name == *field)
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: format!("aggregate layout {} has no field `{field}`", layout.name),
                });
            };
            if let Some(output) = instruction.output {
                if function.value_types.get(&output) != Some(&declared.ty) {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "FieldGet result type differs from declared aggregate field type".to_string(),
                    });
                }
            }
            if let Some(ExecutableOperand::Value(value)) = value {
                if function.value_types.get(value) != Some(&declared.ty) {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "FieldSet value type differs from declared aggregate field type".to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_static_call_resolution(submission: &FragmentSubmission, function: &ExecutableFunction) -> Result<(), SemanticMirContractError> {
    for block in &function.blocks {
        for (index, instruction) in block.instructions.iter().enumerate() {
            let ExecutableInstructionKind::Call { dispatch, callee, intrinsic_opcode, .. } = &instruction.kind
            else {
                continue;
            };
            if !matches!(dispatch, crate::contracts::DispatchKind::Static) || intrinsic_opcode.is_some() {
                continue;
            }
            let location = format!("block {} instruction {index}", block.id.0);
            let ExecutableOperand::Symbol(path) = callee
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR003",
                    function: function.symbol.clone(),
                    location,
                    detail: "static call requires an explicit callee symbol".to_string(),
                });
            };
            let symbol = path.to_string();
            let local = submission.executable.as_ref().is_some_and(|executable| executable.find_by_symbol(&symbol).is_some());
            let external = submission.external_import_links.contains_key(&QualifiedName::new(path.parts().to_vec()));
            if !local && !external {
                return Err(SemanticMirContractError {
                    code: "SMIR003",
                    function: function.symbol.clone(),
                    location,
                    detail: format!("static callee `{symbol}` is absent from the exact function registry and external import registry"),
                });
            }
        }
    }
    Ok(())
}

fn validate_nominal_sums(submission: &FragmentSubmission) -> Result<(), SemanticMirContractError> {
    for sum in &submission.sum_types {
        if sum.name.is_empty() || sum.variants.is_empty() || sum.tag_width == 0 {
            return Err(SemanticMirContractError {
                code: "SMIR006",
                function: sum.name.clone(),
                location: "sum layout".to_string(),
                detail: "nominal sum layout requires a name, tag width, and at least one variant".to_string(),
            });
        }
        for (index, variant) in sum.variants.iter().enumerate() {
            if variant.name.is_empty() {
                return Err(SemanticMirContractError {
                    code: "SMIR006",
                    function: sum.name.clone(),
                    location: "sum variant".to_string(),
                    detail: format!("nominal sum variant {index} has no name"),
                });
            }
            if sum.variants[..index].iter().any(|prior| prior.name == variant.name || prior.tag == variant.tag) {
                return Err(SemanticMirContractError {
                    code: "SMIR006",
                    function: sum.name.clone(),
                    location: "sum variant".to_string(),
                    detail: format!("nominal sum variant {} duplicates a prior name or tag", variant.name),
                });
            }
        }
    }
    Ok(())
}

fn validate_function(function: &ExecutableFunction) -> Result<(), SemanticMirContractError> {
    if !function.blocks.iter().any(|block| block.id == function.entry) {
        return Err(SemanticMirContractError {
            code: "SMIR009",
            function: function.symbol.clone(),
            location: "entry".to_string(),
            detail: format!("entry block {:?} is not present in the function", function.entry),
        });
    }
    let missing = |value, location: String| SemanticMirContractError {
        code: "SMIR001",
        function: function.symbol.clone(),
        location: location.clone(),
        detail: format!("semantic type missing for value {value:?} at {location}"),
    };

    for block in &function.blocks {
        for (index, parameter) in block.parameters.iter().enumerate() {
            if !function.value_types.contains_key(parameter) {
                return Err(missing(parameter, format!("block {} parameter {index}", block.id.0)));
            }
        }
        for (index, instruction) in block.instructions.iter().enumerate() {
            // A residual high-level pattern is the primary contract failure.
            // Report SMIR008 before secondary metadata checks so malformed
            // residual MIR cannot be misreported as a missing value type.
            if let ExecutableInstructionKind::PatternMatch { .. } = instruction.kind {
                return Err(SemanticMirContractError {
                    code: "SMIR008",
                    function: function.symbol.clone(),
                    location: format!("block {} instruction {index}", block.id.0),
                    detail: format!("residual PatternMatch instruction at block {} instruction {index}", block.id.0),
                });
            }
            if let Some(output) = instruction.output {
                if !function.value_types.contains_key(&output) {
                    return Err(SemanticMirContractError {
                        code: "SMIR001",
                        function: function.symbol.clone(),
                        location: format!("block {} instruction {index}", block.id.0),
                        detail: format!(
                            "semantic type missing for value {output:?} at block {} instruction {index}; kind={:?}",
                            block.id.0, instruction.kind
                        ),
                    });
                }
            }
            if let ExecutableInstructionKind::LoadConstant { constant, ty } = &instruction.kind {
                if let Some(literal_type) = text_constant_type(constant) {
                    let location = format!("block {} instruction {index}", block.id.0);
                    if ty.as_ref() != Some(&literal_type) {
                        return Err(SemanticMirContractError {
                            code: "SMIR007",
                            function: function.symbol.clone(),
                            location,
                            detail: "text constant encoding/type contract is absent or disagrees with the literal".to_string(),
                        });
                    }
                    if let Some(output) = instruction.output {
                        if function.value_types.get(&output) != Some(&literal_type) {
                            return Err(SemanticMirContractError {
                                code: "SMIR007",
                                function: function.symbol.clone(),
                                location: format!("block {} instruction {index}", block.id.0),
                                detail: "text constant result SSA type disagrees with the literal encoding".to_string(),
                            });
                        }
                    }
                }
            }
            if let ExecutableInstructionKind::TextConvert { source_encoding, target_encoding, semantics, boundary, value } = &instruction.kind {
                validate_text_convert_contract(
                    function,
                    instruction.output,
                    *source_encoding,
                    *target_encoding,
                    *semantics,
                    *boundary,
                    value,
                    block.id.0,
                    index,
                )?;
            }
            for (operand_index, operand) in instruction_operands(&instruction.kind).into_iter().enumerate() {
                if let ExecutableOperand::Value(value) = operand {
                    if !function.value_types.contains_key(value) {
                        return Err(missing(value, format!("block {} instruction {index} operand {operand_index}", block.id.0)));
                    }
                }
            }
            if let ExecutableInstructionKind::Call { arguments, parameter_types, .. } = &instruction.kind {
                let location = format!("block {} instruction {index}", block.id.0);
                let Some(parameter_types) = parameter_types
                else {
                    return Err(SemanticMirContractError {
                        code: "SMIR002",
                        function: function.symbol.clone(),
                        location,
                        detail: "call has no explicit formal parameter signature".to_string(),
                    });
                };
                if arguments.len() != parameter_types.len() {
                    return Err(SemanticMirContractError {
                        code: "SMIR002",
                        function: function.symbol.clone(),
                        location,
                        detail: format!(
                            "call argument metadata arity mismatch at block {} instruction {index}: {} operands, {} parameter types",
                            block.id.0,
                            arguments.len(),
                            parameter_types.len()
                        ),
                    });
                }
            }
            if let ExecutableInstructionKind::Call { arguments, intrinsic_opcode: Some(opcode), .. } = &instruction.kind {
                validate_intrinsic_call(function, instruction.output, arguments, *opcode, block.id.0, index)?;
            }
        }
        validate_terminator(function, block)?;
    }
    Ok(())
}

fn validate_text_convert_contract(
    function: &ExecutableFunction,
    output: Option<crate::contracts::ValueRef>,
    source_encoding: Option<TextEncoding>,
    target_encoding: Option<TextEncoding>,
    semantics: Option<TextConversionSemantics>,
    boundary: Option<TextProjectionBoundary>,
    value: &ExecutableOperand,
    block: u32,
    instruction: usize,
) -> Result<(), SemanticMirContractError> {
    let location = format!("block {block} instruction {instruction}");
    let (Some(source_encoding), Some(target_encoding), Some(semantics), Some(boundary)) =
        (source_encoding, target_encoding, semantics, boundary)
    else {
        return Err(SemanticMirContractError {
            code: "SMIR004",
            function: function.symbol.clone(),
            location,
            detail: "text conversion requires explicit source encoding, target encoding, semantics, and projection boundary".to_string(),
        });
    };
    let expected = |encoding| match encoding {
        TextEncoding::Utf8 => NyarType::Utf8,
        TextEncoding::Utf16 => NyarType::Utf16,
    };
    let source = match value {
        ExecutableOperand::Value(value) => function.value_types.get(value),
        _ => None,
    };
    if source != Some(&expected(source_encoding)) {
        return Err(SemanticMirContractError {
            code: "SMIR007",
            function: function.symbol.clone(),
            location,
            detail: "text conversion source SSA type disagrees with explicit source encoding".to_string(),
        });
    }
    if output.and_then(|value| function.value_types.get(&value)) != Some(&expected(target_encoding)) {
        return Err(SemanticMirContractError {
            code: "SMIR007",
            function: function.symbol.clone(),
            location,
            detail: "text conversion result SSA type disagrees with explicit target encoding".to_string(),
        });
    }
    match (semantics, boundary) {
        (
            TextConversionSemantics::UnicodeScalarPreserving,
            TextProjectionBoundary::Language
            | TextProjectionBoundary::Clr
            | TextProjectionBoundary::Jvm
            | TextProjectionBoundary::WasmJsGlue
            | TextProjectionBoundary::WasiComponent,
        ) => Ok(()),
    }
}

fn text_constant_type(constant: &Constant) -> Option<NyarType> {
    match constant {
        Constant::Utf8(_) => Some(NyarType::Utf8),
        Constant::Utf16(_) => Some(NyarType::Utf16),
        _ => None,
    }
}

fn validate_intrinsic_call(
    function: &ExecutableFunction,
    output: Option<crate::contracts::ValueRef>,
    arguments: &[ExecutableOperand],
    opcode: IntrinsicOpcode,
    block: u32,
    instruction: usize,
) -> Result<(), SemanticMirContractError> {
    let location = format!("block {block} instruction {instruction}");
    let reject = |code, detail| SemanticMirContractError { code, function: function.symbol.clone(), location: location.clone(), detail };
    let value_type = |operand: &ExecutableOperand| match operand {
        ExecutableOperand::Value(value) => function.value_types.get(value),
        _ => None,
    };
    let is_array = |ty: &NyarType| matches!(ty, NyarType::Array(_) | NyarType::FixedArray { .. });
    let is_i32 = |ty: &NyarType| matches!(ty, NyarType::Integer32 { .. });

    match opcode {
        IntrinsicOpcode::ArrayLen => {
            if arguments.len() != 1 {
                return Err(reject("SMIR004", format!("ArrayLen requires one array receiver, got {} operands", arguments.len())));
            }
            let Some(receiver_type) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR005", "ArrayLen receiver has no SSA semantic type".to_string()));
            };
            if !is_array(receiver_type) {
                return Err(reject("SMIR005", format!("ArrayLen receiver must be Array or FixedArray, got {receiver_type}")));
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "ArrayLen result has no SSA semantic type".to_string()));
            };
            if !is_i32(result) {
                return Err(reject("SMIR005", format!("ArrayLen result must be i32, got {result}")));
            }
        }
        IntrinsicOpcode::ArrayGet => {
            if arguments.len() != 2 {
                return Err(reject("SMIR004", format!("ArrayGet requires 2 operands, got {}", arguments.len())));
            }
            let Some(receiver_type) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR005", "ArrayGet receiver has no SSA semantic type".to_string()));
            };
            let element_type = match receiver_type {
                NyarType::Array(element) | NyarType::FixedArray { element, .. } => element.as_ref(),
                _ => return Err(reject("SMIR005", format!("ArrayGet receiver must be Array or FixedArray, got {receiver_type}"))),
            };
            if !value_type(&arguments[1]).is_some_and(is_i32) {
                return Err(reject("SMIR005", "ArrayGet index must have an explicit i32 type".to_string()));
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "ArrayGet result has no SSA semantic type".to_string()));
            };
            if result != element_type {
                return Err(reject("SMIR005", format!("ArrayGet result must be {element_type}, got {result}")));
            }
        }
        IntrinsicOpcode::ArraySet | IntrinsicOpcode::ArrayPush => {
            let expected_arity = match opcode {
                IntrinsicOpcode::ArraySet => 3,
                IntrinsicOpcode::ArrayPush => 2,
                _ => unreachable!(),
            };
            if arguments.len() != expected_arity {
                return Err(reject("SMIR004", format!("{opcode:?} requires {expected_arity} operands, got {}", arguments.len())));
            }
            let Some(receiver_type) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR005", format!("{opcode:?} receiver has no SSA semantic type")));
            };
            let element_type = match receiver_type {
                NyarType::Array(element) | NyarType::FixedArray { element, .. } => element.as_ref(),
                _ => return Err(reject("SMIR005", format!("{opcode:?} receiver must be Array or FixedArray, got {receiver_type}"))),
            };
            let value_index = if matches!(opcode, IntrinsicOpcode::ArraySet) {
                if !value_type(&arguments[1]).is_some_and(is_i32) {
                    return Err(reject("SMIR005", "ArraySet index must have an explicit i32 type".to_string()));
                }
                2
            }
            else {
                1
            };
            if value_type(&arguments[value_index]) != Some(element_type) {
                return Err(reject("SMIR005", format!("{opcode:?} value must be the declared array element type {element_type}")));
            }
            if matches!(opcode, IntrinsicOpcode::ArrayPush) {
                let Some(result) = output.and_then(|value| function.value_types.get(&value))
                else {
                    return Err(reject("SMIR001", "ArrayPush result has no SSA semantic type".to_string()));
                };
                if result != receiver_type {
                    return Err(reject("SMIR005", "ArrayPush result must preserve the declared array type".to_string()));
                }
            }
        }
        IntrinsicOpcode::Utf8ScalarLength => {
            if arguments.len() != 1 {
                return Err(reject("SMIR004", format!("Utf8ScalarLength requires one utf8 receiver, got {} operands", arguments.len())));
            }
            let Some(receiver_type) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR005", "Utf8ScalarLength receiver has no SSA semantic type".to_string()));
            };
            if !matches!(receiver_type, NyarType::Utf8) {
                return Err(reject("SMIR005", format!("Utf8ScalarLength receiver must be utf8, got {receiver_type}")));
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "Utf8ScalarLength result has no SSA semantic type".to_string()));
            };
            if !is_i32(result) {
                return Err(reject("SMIR005", format!("Utf8ScalarLength result must be i32, got {result}")));
            }
        }
        IntrinsicOpcode::Utf8ContentEqual | IntrinsicOpcode::Utf8ContentNotEqual => {
            if arguments.len() != 2 {
                return Err(reject("SMIR004", format!("Utf8 content comparison requires two operands, got {}", arguments.len())));
            }
            for argument in arguments {
                let Some(argument_type) = value_type(argument)
                else {
                    return Err(reject("SMIR001", "Utf8 content comparison operand has no SSA semantic type".to_string()));
                };
                if !matches!(argument_type, NyarType::Utf8) {
                    return Err(reject("SMIR007", format!("Utf8 content comparison requires utf8 operands, got {argument_type}")));
                }
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "Utf8 content comparison result has no SSA semantic type".to_string()));
            };
            if !matches!(result, NyarType::Boolean) {
                return Err(reject("SMIR005", format!("Utf8 content comparison result must be bool, got {result}")));
            }
        }
        IntrinsicOpcode::Utf8ScalarSlice => {
            if arguments.len() != 3 {
                return Err(reject(
                    "SMIR004",
                    format!("Utf8ScalarSlice requires utf8, scalar_start, scalar_count; got {} operands", arguments.len()),
                ));
            }
            let Some(receiver_type) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR005", "Utf8ScalarSlice receiver has no SSA semantic type".to_string()));
            };
            if !matches!(receiver_type, NyarType::Utf8) {
                return Err(reject("SMIR005", format!("Utf8ScalarSlice receiver must be utf8, got {receiver_type}")));
            }
            for (position, argument) in arguments[1..].iter().enumerate() {
                let Some(argument_type) = value_type(argument)
                else {
                    return Err(reject("SMIR005", format!("Utf8ScalarSlice scalar operand {} has no SSA semantic type", position + 1)));
                };
                if !is_i32(argument_type) {
                    return Err(reject("SMIR005", format!("Utf8ScalarSlice scalar operand {} must be i32, got {argument_type}", position + 1)));
                }
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "Utf8ScalarSlice result has no SSA semantic type".to_string()));
            };
            if !matches!(result, NyarType::Utf8) {
                return Err(reject("SMIR005", format!("Utf8ScalarSlice result must be utf8, got {result}")));
            }
        }
        IntrinsicOpcode::Utf8Trim => {
            if arguments.len() != 1 {
                return Err(reject("SMIR004", "Utf8Trim requires one utf8 operand".to_string()));
            }
            let Some(input) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR001", "Utf8Trim operand has no SSA semantic type".to_string()));
            };
            if !matches!(input, NyarType::Utf8) {
                return Err(reject("SMIR005", format!("Utf8Trim requires utf8, got {input}")));
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "Utf8Trim result has no SSA semantic type".to_string()));
            };
            if !matches!(result, NyarType::Utf8) {
                return Err(reject("SMIR005", format!("Utf8Trim result must be utf8, got {result}")));
            }
        }
        IntrinsicOpcode::Utf8IndexOf | IntrinsicOpcode::Utf8Contains | IntrinsicOpcode::Utf8StartsWith | IntrinsicOpcode::Utf8EndsWith => {
            if arguments.len() != 2 {
                return Err(reject("SMIR004", format!("{opcode:?} requires two utf8 operands")));
            }
            for argument in arguments {
                let Some(ty) = value_type(argument)
                else {
                    return Err(reject("SMIR001", format!("{opcode:?} operand has no SSA semantic type")));
                };
                if !matches!(ty, NyarType::Utf8) {
                    return Err(reject("SMIR005", format!("{opcode:?} requires utf8 operands, got {ty}")));
                }
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", format!("{opcode:?} result has no SSA semantic type")));
            };
            let valid_result = matches!(opcode, IntrinsicOpcode::Utf8IndexOf) && is_i32(result)
                || !matches!(opcode, IntrinsicOpcode::Utf8IndexOf) && matches!(result, NyarType::Boolean);
            if !valid_result {
                return Err(reject("SMIR005", format!("{opcode:?} result type is {result}")));
            }
        }
        IntrinsicOpcode::SumStructuralEqual => {
            if arguments.len() != 2 {
                return Err(reject("SMIR004", "SumStructuralEqual requires two sum operands".to_string()));
            }
            let Some(left) = value_type(&arguments[0])
            else {
                return Err(reject("SMIR001", "SumStructuralEqual left operand has no SSA semantic type".to_string()));
            };
            let Some(right) = value_type(&arguments[1])
            else {
                return Err(reject("SMIR001", "SumStructuralEqual right operand has no SSA semantic type".to_string()));
            };
            let nominal_name = |ty: &NyarType| match ty {
                NyarType::Named(name) => Some(name.to_string()),
                NyarType::Apply(base, _) => match base.as_ref() {
                    NyarType::Named(name) => Some(name.to_string()),
                    _ => None,
                },
                _ => None,
            };
            let (Some(left_name), Some(right_name)) = (nominal_name(left), nominal_name(right))
            else {
                return Err(reject("SMIR005", "SumStructuralEqual operands must be nominal sum types".to_string()));
            };
            if left_name != right_name {
                return Err(reject("SMIR005", format!("SumStructuralEqual nominal types differ: {left_name} vs {right_name}")));
            }
            let Some(result) = output.and_then(|value| function.value_types.get(&value))
            else {
                return Err(reject("SMIR001", "SumStructuralEqual result has no SSA semantic type".to_string()));
            };
            if !matches!(result, NyarType::Boolean) {
                return Err(reject("SMIR005", format!("SumStructuralEqual result must be bool, got {result}")));
            }
        }
        IntrinsicOpcode::SumVariantIs => {
            return Err(reject("SMIR004", "SumVariantIs requires an explicit variant descriptor contract".to_string()));
        }
        _ => {}
    }
    Ok(())
}

fn instruction_operands(kind: &ExecutableInstructionKind) -> Vec<&ExecutableOperand> {
    use crate::contracts::InstructionKind::*;
    match kind {
        LoadConstant { .. } | LoadSymbol { .. } => Vec::new(),
        Copy { source } => vec![source],
        StoreVar { value, .. } => vec![value],
        Call { callee, arguments, witness, effect, .. } => {
            let mut values = vec![callee];
            values.extend(arguments);
            if let Some(witness) = witness {
                values.push(witness);
            }
            if let Some(effect) = effect {
                values.push(effect);
            }
            values
        }
        StructNew { fields, .. } => fields.iter().map(|(_, value)| value).collect(),
        TupleNew { fields, .. } => fields.iter().collect(),
        FixedArrayNew { items, .. } => items.iter().collect(),
        AggregateCopy { source, dest, .. } => vec![source, dest],
        FieldGet { object, .. } => vec![object],
        FieldSet { object, value, .. } => vec![object, value],
        SumNew { payload, .. } => payload.iter().collect(),
        SumPayloadGet { object, .. } => vec![object],
        TextConvert { value, .. } => vec![value],
        PatternMatch { value, .. } => vec![value],
        ArrayNew { length, .. } => vec![length],
        ArrayLiteral { items, .. } => items.iter().collect(),
    }
}

fn validate_terminator(function: &ExecutableFunction, block: &crate::contracts::Block) -> Result<(), SemanticMirContractError> {
    let location = format!("block {} terminator", block.id.0);
    let value_type = |operand: &ExecutableOperand| match operand {
        ExecutableOperand::Value(value) => function.value_types.get(value),
        ExecutableOperand::Constant(Constant::Bool(_)) => Some(&NyarType::Boolean),
        _ => None,
    };
    let operands: Vec<&ExecutableOperand> = match &block.terminator {
        crate::contracts::Terminator::Return { value: Some(value) } => vec![value],
        crate::contracts::Terminator::Jump { arguments, .. } => arguments.iter().collect(),
        crate::contracts::Terminator::Branch { condition, .. } => vec![condition],
        crate::contracts::Terminator::PerformEffect { payload: Some(payload), .. } => vec![payload],
        crate::contracts::Terminator::YieldToRuntime { payload: Some(payload), .. } => vec![payload],
        crate::contracts::Terminator::StateDispatch { .. } => Vec::new(),
        _ => Vec::new(),
    };
    for operand in operands {
        if let ExecutableOperand::Value(value) = operand {
            if !function.value_types.contains_key(value) {
                return Err(SemanticMirContractError {
                    code: "SMIR001",
                    function: function.symbol.clone(),
                    location: location.clone(),
                    detail: format!("semantic type missing for terminator value {value:?} in block {}", block.id.0),
                });
            }
        }
    }
    if let crate::contracts::Terminator::StateDispatch { state, .. } = &block.terminator {
        if !function.value_types.contains_key(state) {
            return Err(SemanticMirContractError {
                code: "SMIR001",
                function: function.symbol.clone(),
                location: format!("block {} terminator state", block.id.0),
                detail: format!("semantic type missing for terminator state {state:?} in block {}", block.id.0),
            });
        }
    }
    match &block.terminator {
        crate::contracts::Terminator::Return { value: Some(value) } if value_type(value) != Some(&function.return_type) => {
            return Err(SemanticMirContractError {
                code: "SMIR007",
                function: function.symbol.clone(),
                location,
                detail: "return operand type differs from function return type".to_string(),
            });
        }
        crate::contracts::Terminator::Jump { target, arguments } => {
            let Some(destination) = function.blocks.iter().find(|candidate| candidate.id == *target)
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR007",
                    function: function.symbol.clone(),
                    location,
                    detail: "jump target is absent".to_string(),
                });
            };
            if destination.parameters.len() != arguments.len() {
                return Err(SemanticMirContractError {
                    code: "SMIR007",
                    function: function.symbol.clone(),
                    location,
                    detail: "jump arity differs from target block parameters".to_string(),
                });
            }
            for (argument, parameter) in arguments.iter().zip(&destination.parameters) {
                if value_type(argument) != function.value_types.get(parameter) {
                    return Err(SemanticMirContractError {
                        code: "SMIR007",
                        function: function.symbol.clone(),
                        location,
                        detail: "jump argument type differs from target block parameter".to_string(),
                    });
                }
            }
        }
        crate::contracts::Terminator::Branch { condition, .. } if value_type(condition) != Some(&NyarType::Boolean) => {
            return Err(SemanticMirContractError {
                code: "SMIR007",
                function: function.symbol.clone(),
                location,
                detail: "branch condition must be bool".to_string(),
            });
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{Block, BlockRef, DispatchKind, ExecutableFunction, Instruction, InstructionKind, Operand, Terminator, ValueRef};
    use nyar::{Identifier, NamePath};
    use nyar_types::{
        NyarType,
        layout::{SumTypeLayout, SumVariantLayout},
    };
    use std::collections::BTreeMap;

    fn function(instructions: Vec<Instruction>, terminator: Terminator, value_types: BTreeMap<ValueRef, NyarType>) -> ExecutableFunction {
        ExecutableFunction {
            symbol: "contract_fixture".to_string(),
            return_type: NyarType::Unit,
            param_types: Vec::new(),
            value_types,
            entry: BlockRef(0),
            values: Vec::new(),
            intrinsic: None,
            suspend_points: Vec::new(),
            frame_layouts: Vec::new(),
            continuations: Vec::new(),
            case_chains: Vec::new(),
            #[allow(deprecated)]
            state_machine: None,
            suspend_plan: None,
            state_machine_lowered: true,
            blocks: vec![Block { id: BlockRef(0), label: "entry".to_string(), parameters: Vec::new(), instructions, terminator }],
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn rejects_instruction_output_without_semantic_type() {
        let result = validate_function(&function(
            vec![Instruction {
                output: Some(ValueRef(7)),
                kind: InstructionKind::Copy { source: Operand::Constant(crate::contracts::Constant::Unit) },
            }],
            Terminator::Return { value: None },
            BTreeMap::new(),
        ));
        assert_eq!(result.unwrap_err().code, "SMIR001");
    }

    #[test]
    fn rejects_residual_pattern_match() {
        let result = validate_function(&function(
            vec![Instruction {
                output: None,
                kind: InstructionKind::PatternMatch {
                    value: Operand::Constant(crate::contracts::Constant::Unit),
                    pattern_debug: "fixture".to_string(),
                },
            }],
            Terminator::Return { value: None },
            BTreeMap::new(),
        ));
        assert_eq!(result.unwrap_err().code, "SMIR008");
    }

    #[test]
    fn accepts_minimal_semantic_function() {
        let result = validate_function(&function(Vec::new(), Terminator::Return { value: None }, BTreeMap::new()));
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_nominal_sum_without_layout_metadata() {
        let mut submission = FragmentSubmission::default();
        submission.sum_types.push(SumTypeLayout {
            name: "Option".to_string(),
            is_unite: true,
            tag_width: 0,
            variants: vec![SumVariantLayout { name: "Some".to_string(), tag: 0, payload_type: Some(NyarType::Integer32 { signed: true }) }],
        });
        let result = validate_submission(&submission);
        assert_eq!(result.unwrap_err().code, "SMIR006");
    }

    #[test]
    fn rejects_nominal_sum_with_duplicate_variant_tag() {
        let mut submission = FragmentSubmission::default();
        submission.sum_types.push(SumTypeLayout {
            name: "Choice".to_string(),
            is_unite: true,
            tag_width: 32,
            variants: vec![
                SumVariantLayout { name: "Left".to_string(), tag: 0, payload_type: None },
                SumVariantLayout { name: "Right".to_string(), tag: 0, payload_type: None },
            ],
        });
        let result = validate_submission(&submission);
        assert_eq!(result.unwrap_err().code, "SMIR006");
    }

    #[test]
    fn observation_is_stable_and_does_not_include_backend_names() {
        let result = validate_function(&function(Vec::new(), Terminator::Return { value: None }, BTreeMap::new()));
        assert_eq!(observation("valid_minimal", result.as_ref().map(|_| ()).map_err(|error| error)), "valid_minimal|accept||");

        let result = validate_function(&function(
            vec![Instruction {
                output: Some(ValueRef(7)),
                kind: InstructionKind::Copy { source: Operand::Constant(crate::contracts::Constant::Unit) },
            }],
            Terminator::Return { value: None },
            BTreeMap::new(),
        ));
        assert_eq!(
            observation("missing_value_type", result.as_ref().map(|_| ()).map_err(|error| error)),
            "missing_value_type|reject|SMIR001|block 0 instruction 0"
        );
    }

    fn sum_equal_fixture(left_ty: NyarType, right_ty: NyarType, include_result_type: bool) -> Result<(), SemanticMirContractError> {
        let left = ValueRef(20);
        let right = ValueRef(21);
        let result = ValueRef(22);
        let mut value_types = BTreeMap::new();
        value_types.insert(left, left_ty.clone());
        value_types.insert(right, right_ty.clone());
        if include_result_type {
            value_types.insert(result, NyarType::Boolean);
        }
        validate_function(&function(
            vec![Instruction {
                output: Some(result),
                kind: InstructionKind::Call {
                    dispatch: DispatchKind::Static,
                    callee: Operand::Symbol(NamePath::new(vec![Identifier::new("primitive"), Identifier::new("sum_equal")])),
                    arguments: vec![Operand::Value(left), Operand::Value(right)],
                    witness: None,
                    effect: None,
                    receiver_kind: None,
                    parameter_types: Some(vec![left_ty, right_ty]),
                    intrinsic_opcode: Some(IntrinsicOpcode::SumStructuralEqual),
                },
            }],
            Terminator::Return { value: Some(Operand::Value(result)) },
            value_types,
        ))
    }

    #[test]
    fn accepts_structural_sum_equality_for_same_nominal_type() {
        let ty = NyarType::Named(Identifier::new("TokenKind"));
        assert!(sum_equal_fixture(ty.clone(), ty, true).is_ok());
    }

    #[test]
    fn rejects_structural_sum_equality_for_different_nominal_types() {
        let result = sum_equal_fixture(NyarType::Named(Identifier::new("Left")), NyarType::Named(Identifier::new("Right")), true);
        assert_eq!(result.unwrap_err().code, "SMIR005");
    }

    #[test]
    fn rejects_structural_sum_equality_without_result_type() {
        let ty = NyarType::Named(Identifier::new("TokenKind"));
        assert_eq!(sum_equal_fixture(ty.clone(), ty, false).unwrap_err().code, "SMIR001");
    }

    #[test]
    fn aggregate_array_sum_contract_uses_structured_array_len() {
        let receiver = ValueRef(10);
        let output = ValueRef(11);
        let mut value_types = BTreeMap::new();
        value_types.insert(receiver, NyarType::Array(Box::new(NyarType::Integer32 { signed: true })));
        value_types.insert(output, NyarType::Integer32 { signed: true });
        let result = validate_function(&function(
            vec![Instruction {
                output: Some(output),
                kind: InstructionKind::Call {
                    dispatch: DispatchKind::Static,
                    callee: Operand::Symbol(NamePath::new(vec![Identifier::new("unrelated"), Identifier::new("operation")])),
                    arguments: vec![Operand::Value(receiver)],
                    witness: None,
                    effect: None,
                    receiver_kind: None,
                    parameter_types: Some(vec![NyarType::Array(Box::new(NyarType::Integer32 { signed: true }))]),
                    intrinsic_opcode: Some(IntrinsicOpcode::ArrayLen),
                },
            }],
            Terminator::Return { value: Some(Operand::Value(output)) },
            value_types,
        ));
        assert_eq!(
            observation("aggregate_array_sum.valid_array_len", result.as_ref().map(|_| ()).map_err(|error| error)),
            "aggregate_array_sum.valid_array_len|accept||"
        );
    }

    #[test]
    fn aggregate_array_sum_rejects_array_len_arity_without_symbol_inference() {
        let receiver = ValueRef(10);
        let output = ValueRef(11);
        let mut value_types = BTreeMap::new();
        value_types.insert(receiver, NyarType::Array(Box::new(NyarType::Integer32 { signed: true })));
        value_types.insert(output, NyarType::Integer32 { signed: true });
        let result = validate_function(&function(
            vec![Instruction {
                output: Some(output),
                kind: InstructionKind::Call {
                    dispatch: DispatchKind::Static,
                    callee: Operand::Symbol(NamePath::new(vec![
                        Identifier::new("array"),
                        Identifier::new("length"),
                        Identifier::new("without"),
                        Identifier::new("contract"),
                    ])),
                    arguments: vec![Operand::Value(receiver), Operand::Constant(crate::contracts::Constant::Unit)],
                    witness: None,
                    effect: None,
                    receiver_kind: None,
                    parameter_types: Some(vec![NyarType::Array(Box::new(NyarType::Integer32 { signed: true })), NyarType::Unit]),
                    intrinsic_opcode: Some(IntrinsicOpcode::ArrayLen),
                },
            }],
            Terminator::Return { value: Some(Operand::Value(output)) },
            value_types,
        ));
        assert_eq!(
            observation("aggregate_array_sum.invalid_array_len_arity", result.as_ref().map(|_| ()).map_err(|error| error)),
            "aggregate_array_sum.invalid_array_len_arity|reject|SMIR004|block 0 instruction 0"
        );
    }

    #[test]
    fn text_contract_uses_structured_unicode_scalar_length() {
        let receiver = ValueRef(20);
        let output = ValueRef(21);
        let mut value_types = BTreeMap::new();
        value_types.insert(receiver, NyarType::Utf8);
        value_types.insert(output, NyarType::Integer32 { signed: true });
        let result = validate_function(&function(
            vec![Instruction {
                output: Some(output),
                kind: InstructionKind::Call {
                    dispatch: DispatchKind::Static,
                    callee: Operand::Symbol(NamePath::new(vec![Identifier::new("unrelated"), Identifier::new("operation")])),
                    arguments: vec![Operand::Value(receiver)],
                    witness: None,
                    effect: None,
                    receiver_kind: None,
                    parameter_types: Some(vec![NyarType::Utf8]),
                    intrinsic_opcode: Some(IntrinsicOpcode::Utf8ScalarLength),
                },
            }],
            Terminator::Return { value: Some(Operand::Value(output)) },
            value_types,
        ));
        assert_eq!(
            observation("text.valid_scalar_length", result.as_ref().map(|_| ()).map_err(|error| error)),
            "text.valid_scalar_length|accept||"
        );
    }

    #[test]
    fn text_contract_rejects_utf16_for_utf8_scalar_length() {
        let receiver = ValueRef(20);
        let output = ValueRef(21);
        let mut value_types = BTreeMap::new();
        value_types.insert(receiver, NyarType::Utf16);
        value_types.insert(output, NyarType::Integer32 { signed: true });
        let result = validate_function(&function(
            vec![Instruction {
                output: Some(output),
                kind: InstructionKind::Call {
                    dispatch: DispatchKind::Static,
                    callee: Operand::Symbol(NamePath::new(vec![Identifier::new("unrelated"), Identifier::new("operation")])),
                    arguments: vec![Operand::Value(receiver)],
                    witness: None,
                    effect: None,
                    receiver_kind: None,
                    parameter_types: Some(vec![NyarType::Utf16]),
                    intrinsic_opcode: Some(IntrinsicOpcode::Utf8ScalarLength),
                },
            }],
            Terminator::Return { value: Some(Operand::Value(output)) },
            value_types,
        ));
        assert_eq!(
            observation("text.invalid_scalar_length_receiver", result.as_ref().map(|_| ()).map_err(|error| error)),
            "text.invalid_scalar_length_receiver|reject|SMIR005|block 0 instruction 0"
        );
    }
}
