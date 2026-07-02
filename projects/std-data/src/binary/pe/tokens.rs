use std::collections::BTreeMap;

use crate::text::msil::{MsilInstructionOperand, MsilMethodRef, MsilMethodSignature, MsilModule, MsilType};

use super::{
    PeWriterError,
    meta_builder::ClrMetadataBuilder,
    metadata::{AssemblyRefRow, MemberRefRow, TypeRefRow, TypeSpecRow, build_method_signature, encode_element_type},
};

/// token 解析结果。
pub(crate) struct TokenResolution {
    /// 替换占位操作数后的模块副本。
    pub module: MsilModule,
    /// `AssemblyRef` 表行。
    pub assembly_ref_rows: Vec<AssemblyRefRow>,
    /// `TypeRef` 表行。
    pub type_ref_rows: Vec<TypeRefRow>,
    /// `MemberRef` 表行。
    pub member_ref_rows: Vec<MemberRefRow>,
    /// `TypeSpec` 表行（`T[]` 等）。
    pub type_spec_rows: Vec<TypeSpecRow>,
    /// `System.Object` 的 `TypeRef` 行号（用于用户类型的 `Extends` 字段）。
    /// 0 表示尚未创建。
    pub system_object_type_ref_row: u16,
    /// `System.ValueType` 的 `TypeRef` 行号（用于结构体的 `Extends` 字段）。
    /// 0 表示尚未创建。
    pub system_value_type_type_ref_row: u16,
}

/// 将字符串/方法/类型/字段操作数解析为真实元数据 token。
pub(crate) fn resolve_module_tokens(
    module: &MsilModule,
    metadata: &mut ClrMetadataBuilder,
    local_method_tokens: &BTreeMap<String, u32>,
) -> Result<TokenResolution, PeWriterError> {
    let mut resolved_module = module.clone();
    let mut assembly_ref_rows = Vec::new();
    let mut type_ref_rows = Vec::new();
    let mut member_ref_rows = Vec::new();
    let mut type_spec_rows = Vec::new();
    let mut assembly_ref_map = BTreeMap::<String, u16>::new();
    let mut type_ref_map = BTreeMap::<(String, String), u16>::new();
    let mut member_ref_map = BTreeMap::<(u16, String, MsilMethodSignature), u16>::new();
    let mut type_spec_map = BTreeMap::<String, u32>::new();

    // 预构建字段 token 映射：(type_name, field_name) -> FieldDef token。
    // Field 表行号按类型定义顺序、字段声明顺序分配（1-indexed）。
    let field_token_map = build_field_token_map(module);
    let local_type_def_tokens = build_local_type_def_token_map(module);
    // 构建本地类型 token 映射，包含值类型信息，用于 MemberRef 签名编码。
    let local_type_token_map = build_local_type_token_map(module);

    // 收集所有用户类型名（简单名和限定名），用于判断方法引用是否指向本地结构体方法。
    // lowering 阶段构造函数的 owner 使用限定名（如 `nyar.CanonicalTarget`），
    // 因此需要同时收集简单名和限定名以匹配不同来源的引用。
    let local_type_names: std::collections::BTreeSet<String> = module
        .types
        .iter()
        .flat_map(|t| {
            let mut names = vec![t.full_name.clone()];
            let qualified = t.qualified_name();
            if qualified != t.full_name {
                names.push(qualified);
            }
            names
        })
        .collect();

    for method in &mut resolved_module.global_methods {
        resolve_method_tokens(
            method,
            metadata,
            local_method_tokens,
            &field_token_map,
            &local_type_def_tokens,
            &local_type_token_map,
            &local_type_names,
            &mut assembly_ref_rows,
            &mut type_ref_rows,
            &mut member_ref_rows,
            &mut type_spec_rows,
            &mut assembly_ref_map,
            &mut type_ref_map,
            &mut member_ref_map,
            &mut type_spec_map,
            &module.assembly.name,
        )?;
    }

    for type_def in &mut resolved_module.types {
        for method in &mut type_def.methods {
            resolve_method_tokens(
                method,
                metadata,
                local_method_tokens,
                &field_token_map,
                &local_type_def_tokens,
                &local_type_token_map,
                &local_type_names,
                &mut assembly_ref_rows,
                &mut type_ref_rows,
                &mut member_ref_rows,
                &mut type_spec_rows,
                &mut assembly_ref_map,
                &mut type_ref_map,
                &mut member_ref_map,
                &mut type_spec_map,
                &module.assembly.name,
            )?;
        }
    }

    // 确保 System.Object 的 TypeRef 存在，用于用户类型的 Extends 字段。
    let system_object_type_ref_row = ensure_type_ref_token(
        "[mscorlib]System.Object",
        metadata,
        &mut assembly_ref_rows,
        &mut type_ref_rows,
        &mut assembly_ref_map,
        &mut type_ref_map,
    )?;
    let system_object_type_ref_row = (system_object_type_ref_row & 0x00FF_FFFF) as u16;

    // 确保 System.ValueType 的 TypeRef 存在，用于结构体的 Extends 字段。
    let system_value_type_type_ref_row = ensure_type_ref_token(
        "[mscorlib]System.ValueType",
        metadata,
        &mut assembly_ref_rows,
        &mut type_ref_rows,
        &mut assembly_ref_map,
        &mut type_ref_map,
    )?;
    let system_value_type_type_ref_row = (system_value_type_type_ref_row & 0x00FF_FFFF) as u16;

    Ok(TokenResolution {
        module: resolved_module,
        assembly_ref_rows,
        type_ref_rows,
        member_ref_rows,
        type_spec_rows,
        system_object_type_ref_row,
        system_value_type_type_ref_row,
    })
}

/// 构建 `(qualified_type_name, field_name) -> FieldDef token` 映射。
///
/// Field 表行号按类型定义顺序、字段声明顺序分配（1-indexed）。
/// 键使用命名空间限定名（如 `nyar.CanonicalTarget`），与 lowering 阶段
/// `FieldGet`/`FieldSet` 生成的字段引用格式一致。
fn build_field_token_map(module: &MsilModule) -> BTreeMap<(String, String), u32> {
    let mut map = BTreeMap::new();
    let mut row = 1u32;
    for type_def in &module.types {
        let qualified = type_def.qualified_name();
        for field in &type_def.fields {
            map.insert((qualified.clone(), field.name.clone()), 0x0400_0000 | row);
            row += 1;
        }
    }
    map
}

fn build_local_type_def_token_map(module: &MsilModule) -> BTreeMap<String, u32> {
    let mut tokens = BTreeMap::new();
    for (index, type_def) in module.types.iter().enumerate() {
        // TypeDef rows: 1=<Module>, 2=_GlobalMethods, 3+=user types.
        let token = 0x0200_0000 | ((index as u32) + 3);
        tokens.insert(type_def.full_name.clone(), token);
        let qualified = type_def.qualified_name();
        tokens.insert(qualified, token);
    }
    tokens
}

/// 构建本地类型 token 映射，包含值类型信息。
///
/// 与 `build_local_type_def_token_map` 相同的 token 分配逻辑，
/// 同时收集值类型名称集合，供 `encode_element_type` 区分
/// `ELEMENT_TYPE_VALUETYPE` 与 `ELEMENT_TYPE_CLASS`。
fn build_local_type_token_map(module: &MsilModule) -> super::metadata::TypeTokenMap {
    let mut tokens = BTreeMap::new();
    let mut value_type_names = std::collections::BTreeSet::new();
    let mut value_type_tokens = std::collections::BTreeSet::new();
    for (index, type_def) in module.types.iter().enumerate() {
        // TypeDef rows: 1=<Module>, 2=_GlobalMethods, 3+=user types.
        let token = 0x0200_0000 | ((index as u32) + 3);
        tokens.insert(type_def.full_name.clone(), token);
        let qualified = type_def.qualified_name();
        tokens.insert(qualified.clone(), token);
        if type_def.is_value_type {
            value_type_names.insert(type_def.full_name.clone());
            value_type_names.insert(qualified);
            value_type_tokens.insert(token);
        }
    }
    super::metadata::TypeTokenMap { tokens, value_type_names, value_type_tokens }
}

fn resolve_local_type_def_token_by_namespace(type_name: &str, tokens: &BTreeMap<String, u32>) -> Option<u32> {
    let prefix = format!("{type_name}.");
    let (first_token,) = {
        let mut matches = tokens.iter().filter(|(name, _)| name.starts_with(&prefix) || **name == type_name);
        let (_, first_token) = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        (first_token,)
    };
    Some(*first_token)
}

fn is_local_type_owner(owner: &str, local_type_names: &std::collections::BTreeSet<String>) -> bool {
    if local_type_names.contains(owner) {
        return true;
    }
    let prefix = format!("{owner}.");
    local_type_names.iter().any(|name| name.starts_with(&prefix))
}

#[allow(clippy::too_many_arguments)]
fn resolve_method_tokens(
    method: &mut crate::text::msil::MsilMethodBody,
    metadata: &mut ClrMetadataBuilder,
    local_method_tokens: &BTreeMap<String, u32>,
    field_token_map: &BTreeMap<(String, String), u32>,
    local_type_def_tokens: &BTreeMap<String, u32>,
    local_type_token_map: &super::metadata::TypeTokenMap,
    local_type_names: &std::collections::BTreeSet<String>,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    member_ref_rows: &mut Vec<MemberRefRow>,
    type_spec_rows: &mut Vec<TypeSpecRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
    member_ref_map: &mut BTreeMap<(u16, String, MsilMethodSignature), u16>,
    type_spec_map: &mut BTreeMap<String, u32>,
    module_name: &str,
) -> Result<(), PeWriterError> {
    for instruction in &mut method.instructions {
        let Some(operand) = instruction.operand.take()
        else {
            continue;
        };

        let resolved = match operand {
            MsilInstructionOperand::StringLiteral(value) => {
                let offset = metadata.user_strings.add(&value);
                MsilInstructionOperand::Token(0x7000_0000 | offset)
            }
            MsilInstructionOperand::Method(method_ref) => {
                let token = resolve_method_ref_token(
                    &method_ref,
                    metadata,
                    local_method_tokens,
                    local_type_token_map,
                    local_type_names,
                    assembly_ref_rows,
                    type_ref_rows,
                    member_ref_rows,
                    assembly_ref_map,
                    type_ref_map,
                    member_ref_map,
                    module_name,
                )
                .map_err(|err| annotate_pe_owner_error(err, &method.method.name, method_ref.owner.as_deref(), &method_ref.name))?;
                MsilInstructionOperand::Token(token)
            }
            MsilInstructionOperand::Type(type_name) => {
                // `T[]` / nested arrays → TypeSpec (ECMA-335 II.22.39), never bare TypeRef.
                if type_name.ends_with("[]") {
                    let type_token = ensure_type_spec_token(
                        &type_name,
                        metadata,
                        local_type_def_tokens,
                        local_type_token_map,
                        assembly_ref_rows,
                        type_ref_rows,
                        type_spec_rows,
                        assembly_ref_map,
                        type_ref_map,
                        type_spec_map,
                    )
                    .map_err(|err| annotate_pe_owner_error(err, &method.method.name, Some(type_name.as_str()), "type-operand"))?;
                    MsilInstructionOperand::Token(type_token)
                }
                else if let Some(token) = local_type_def_tokens.get(&type_name).copied() {
                    MsilInstructionOperand::Token(token)
                }
                else if let Some(token) = resolve_local_type_def_token_by_namespace(&type_name, local_type_def_tokens) {
                    MsilInstructionOperand::Token(token)
                }
                else {
                    let normalized = normalize_external_type_owner_alias(&type_name);
                    let lookup = if normalized != type_name { normalized.to_string() } else { type_name.clone() };
                    let type_token = ensure_type_ref_token(&lookup, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)
                        .map_err(|err| annotate_pe_owner_error(err, &method.method.name, Some(type_name.as_str()), "type-operand"))?;
                    MsilInstructionOperand::Token(type_token)
                }
            }
            MsilInstructionOperand::Field(type_name, field_name) => {
                // 解析字段引用为 FieldDef token。
                let key = (type_name.clone(), field_name.clone());
                let token = field_token_map
                    .get(&key)
                    .copied()
                    .or_else(|| {
                        // 回退：当 owner 推断失败（如回退为 "Object"）时，
                        // 按字段名在所有本地类型中搜索唯一匹配项。
                        let mut matches = field_token_map.iter().filter(|((_, candidate), _)| candidate.as_str() == field_name.as_str());
                        let (_, first_token) = matches.next()?;
                        if matches.next().is_some() {
                            return None;
                        }
                        Some(*first_token)
                    })
                    .ok_or_else(|| PeWriterError::MissingLocalMethodToken(format!("{type_name}.{field_name}")))?;
                MsilInstructionOperand::Token(token)
            }
            other => other,
        };

        instruction.operand = Some(resolved);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn resolve_method_ref_token(
    method_ref: &MsilMethodRef,
    metadata: &mut ClrMetadataBuilder,
    local_method_tokens: &BTreeMap<String, u32>,
    local_type_token_map: &super::metadata::TypeTokenMap,
    local_type_names: &std::collections::BTreeSet<String>,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    member_ref_rows: &mut Vec<MemberRefRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
    member_ref_map: &mut BTreeMap<(u16, String, MsilMethodSignature), u16>,
    module_name: &str,
) -> Result<u32, PeWriterError> {
    let normalized_name = method_ref.name.trim_start_matches('.');
    let mut lookup_keys = Vec::new();
    if let Some(owner) = method_ref.owner.as_deref() {
        lookup_keys.push(format!("{}.{}", owner, method_ref.name));
        if normalized_name != method_ref.name {
            lookup_keys.push(format!("{}.{}", owner, normalized_name));
        }
    }
    lookup_keys.push(method_ref.name.clone());
    if normalized_name != method_ref.name {
        lookup_keys.push(normalized_name.to_string());
    }
    for key in lookup_keys {
        if let Some(token) = local_method_tokens.get(&key) {
            return Ok(*token);
        }
    }
    let suffix = format!("__{}", normalized_name);
    let mut suffix_matches = local_method_tokens.iter().filter(|(key, _)| key.ends_with(&suffix) || key.as_str() == method_ref.name);
    if let Some((_, token)) = suffix_matches.next() {
        if suffix_matches.next().is_none() {
            return Ok(*token);
        }
    }

    // 判断是否为本地方法引用：owner 为 None（全局方法）、等于模块名、或匹配用户类型名。
    let is_local = match method_ref.owner.as_deref() {
        None => true,
        Some(owner) => owner == module_name || is_local_type_owner(owner, local_type_names),
    };
    if is_local {
        return Err(PeWriterError::MissingLocalMethodToken(method_ref.name.clone()));
    }

    let owner = method_ref.owner.as_deref().ok_or_else(|| PeWriterError::MissingExternalMethodOwner(method_ref.name.clone()))?;
    let type_token = ensure_type_ref_token(owner, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)?;
    let type_row = (type_token & 0x00FF_FFFF) as u16;
    let class_coded_index = (type_row << 3) | 0x0001;
    let signature_key = method_ref.signature.clone();
    let member_key = (class_coded_index, method_ref.name.clone(), signature_key.clone());
    if let Some(row) = member_ref_map.get(&member_key) {
        return Ok(0x0A00_0000 | u32::from(*row));
    }

    let name_offset = metadata.strings.add(&method_ref.name);
    // 构建 MemberRef 签名用的类型 token 映射，包含本地 TypeDef 和外部 TypeRef。
    // 这样签名中的 `Named` 类型能被正确编码为 `ELEMENT_TYPE_CLASS` + TypeDefOrRef，
    // 而不是统一回退为 `ELEMENT_TYPE_OBJECT`，避免 CLR 方法绑定失败。
    let mut sig_type_token_map = super::metadata::TypeTokenMap {
        tokens: local_type_token_map.tokens.clone(),
        value_type_names: local_type_token_map.value_type_names.clone(),
        value_type_tokens: local_type_token_map.value_type_tokens.clone(),
    };
    let owner_normalized = normalize_external_type_owner_alias(owner);
    if let Ok((_assembly, full_type_name)) = parse_external_owner(owner_normalized) {
        sig_type_token_map.tokens.insert(full_type_name, type_token);
    }
    // MemberRef blobs must encode BCL class returns/params (e.g. DirectoryInfo),
    // not collapse unknown Named → OBJECT — that yields MissingMethodException.
    ensure_named_type_tokens_for_encode(
        &method_ref.signature.return_type,
        &mut sig_type_token_map,
        metadata,
        assembly_ref_rows,
        type_ref_rows,
        assembly_ref_map,
        type_ref_map,
    )?;
    for param in &method_ref.signature.parameter_types {
        ensure_named_type_tokens_for_encode(
            param,
            &mut sig_type_token_map,
            metadata,
            assembly_ref_rows,
            type_ref_rows,
            assembly_ref_map,
            type_ref_map,
        )?;
    }
    let signature_offset = metadata.blob.add(&build_method_signature(
        &method_ref.signature,
        method_ref.name == ".ctor" || method_ref.signature.has_this,
        &sig_type_token_map,
    ));
    let row = to_u16(member_ref_rows.len() as u32 + 1, "MemberRef 行号")?;
    member_ref_rows.push(MemberRefRow { class: class_coded_index, name_offset, signature_offset });
    member_ref_map.insert(member_key, row);
    Ok(0x0A00_0000 | u32::from(row))
}

fn ensure_type_ref_token(
    owner: &str,
    metadata: &mut ClrMetadataBuilder,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
) -> Result<u32, PeWriterError> {
    let owner = normalize_external_type_owner_alias(owner);
    let (assembly_name, full_type_name) = parse_external_owner(owner)?;
    let key = (assembly_name.clone(), full_type_name.clone());
    if let Some(row) = type_ref_map.get(&key) {
        return Ok(0x0100_0000 | u32::from(*row));
    }

    let assembly_row = if let Some(row) = assembly_ref_map.get(&assembly_name) {
        *row
    }
    else {
        let row = to_u16(assembly_ref_rows.len() as u32 + 1, "AssemblyRef 行号")?;
        let name_offset = metadata.strings.add(&assembly_name);
        assembly_ref_rows.push(AssemblyRefRow { name_offset });
        assembly_ref_map.insert(assembly_name.clone(), row);
        row
    };

    let (type_namespace, type_name) = split_namespace_and_name(&full_type_name);
    let resolution_scope = (assembly_row << 2) | 0x0002;
    let row = to_u16(type_ref_rows.len() as u32 + 1, "TypeRef 行号")?;
    type_ref_rows.push(TypeRefRow {
        resolution_scope,
        type_name_offset: metadata.strings.add(&type_name),
        type_namespace_offset: metadata.strings.add(&type_namespace),
    });
    type_ref_map.insert(key, row);
    Ok(0x0100_0000 | u32::from(row))
}

/// Resolve `T[]` / nested array type operands to a `TypeSpec` token (`0x1Bxxxxxx`).
///
/// TypeSpecBlob is a bare `Type` (ECMA-335 II.23.2.14); for single-dimensional arrays
/// that is `ELEMENT_TYPE_SZARRAY` + element encoding via [`encode_element_type`].
#[allow(clippy::too_many_arguments)]
fn ensure_type_spec_token(
    type_name: &str,
    metadata: &mut ClrMetadataBuilder,
    local_type_def_tokens: &BTreeMap<String, u32>,
    local_type_token_map: &super::metadata::TypeTokenMap,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    type_spec_rows: &mut Vec<TypeSpecRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
    type_spec_map: &mut BTreeMap<String, u32>,
) -> Result<u32, PeWriterError> {
    if let Some(&token) = type_spec_map.get(type_name) {
        return Ok(token);
    }

    let msil_ty = parse_type_operand_name(type_name);
    let mut encode_map = super::metadata::TypeTokenMap {
        tokens: local_type_def_tokens.clone(),
        value_type_names: local_type_token_map.value_type_names.clone(),
        value_type_tokens: local_type_token_map.value_type_tokens.clone(),
    };
    ensure_named_type_tokens_for_encode(&msil_ty, &mut encode_map, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)?;

    let blob = encode_element_type(&msil_ty, &encode_map);
    let signature_offset = metadata.blob.add(&blob);
    let row = to_u16(type_spec_rows.len() as u32 + 1, "TypeSpec 行号")?;
    type_spec_rows.push(TypeSpecRow { signature_offset });
    let token = 0x1B00_0000 | u32::from(row);
    type_spec_map.insert(type_name.to_string(), token);
    Ok(token)
}

fn parse_type_operand_name(name: &str) -> MsilType {
    if let Some(inner) = name.strip_suffix("[]") {
        return MsilType::SzArray(Box::new(parse_type_operand_name(inner)));
    }
    match name {
        "bool" => MsilType::Bool,
        "char" => MsilType::Char,
        "int8" => MsilType::Int8 { signed: true },
        "uint8" => MsilType::Int8 { signed: false },
        "int16" => MsilType::Int16 { signed: true },
        "uint16" => MsilType::Int16 { signed: false },
        "int32" => MsilType::Int32 { signed: true },
        "uint32" => MsilType::Int32 { signed: false },
        "int64" => MsilType::Int64 { signed: true },
        "uint64" => MsilType::Int64 { signed: false },
        "float32" => MsilType::Float32,
        "float64" => MsilType::Float64,
        "string" => MsilType::String,
        "object" => MsilType::Object,
        "native int" => MsilType::IntPtr { signed: true },
        "native unsigned int" => MsilType::IntPtr { signed: false },
        other => MsilType::Named(other.to_string()),
    }
}

fn ensure_named_type_tokens_for_encode(
    ty: &MsilType,
    encode_map: &mut super::metadata::TypeTokenMap,
    metadata: &mut ClrMetadataBuilder,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
) -> Result<(), PeWriterError> {
    match ty {
        MsilType::SzArray(inner) => {
            ensure_named_type_tokens_for_encode(inner, encode_map, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)
        }
        MsilType::Named(name) => {
            let normalized = normalize_external_type_owner_alias(name);
            let lookup = if normalized != name.as_str() { normalized.to_string() } else { name.clone() };
            if let Some(&token) = encode_map.tokens.get(name).or_else(|| encode_map.tokens.get(&lookup)) {
                // Token may have been inserted earlier without valuetype annotation (e.g. shared
                // TypeRef map). Re-apply so SearchOption encodes as ELEMENT_TYPE_VALUETYPE.
                if lookup.starts_with('[') && is_external_valuetype_type_ref(&lookup) {
                    encode_map.value_type_names.insert(name.clone());
                    encode_map.value_type_names.insert(lookup);
                    encode_map.value_type_tokens.insert(token);
                }
                return Ok(());
            }
            if lookup.starts_with('[') {
                let token = ensure_type_ref_token(&lookup, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)?;
                encode_map.tokens.insert(name.clone(), token);
                encode_map.tokens.insert(lookup.clone(), token);
                // BCL enums / structs referenced only via MemberRef must encode as
                // ELEMENT_TYPE_VALUETYPE (0x11). Default CLASS (0x12) yields MissingMethodException
                // for e.g. Directory.GetFiles(..., SearchOption).
                if is_external_valuetype_type_ref(&lookup) {
                    encode_map.value_type_names.insert(name.clone());
                    encode_map.value_type_names.insert(lookup);
                    encode_map.value_type_tokens.insert(token);
                }
                Ok(())
            }
            else {
                // Unresolved bare Named inside a TypeSpec is not inventable.
                Err(PeWriterError::UnsupportedExternalTypeOwner(name.clone()))
            }
        }
        _ => Ok(()),
    }
}

fn annotate_pe_owner_error(err: PeWriterError, enclosing_method: &str, owner: Option<&str>, callee: &str) -> PeWriterError {
    match err {
        PeWriterError::UnsupportedExternalTypeOwner(bare) => PeWriterError::UnsupportedExternalTypeOwner(format!(
            "{bare} (enclosing=`{enclosing_method}`, owner=`{}`, callee=`{callee}`)",
            owner.unwrap_or("<none>")
        )),
        other => other,
    }
}

fn parse_external_owner(owner: &str) -> Result<(String, String), PeWriterError> {
    if let Some(rest) = owner.strip_prefix('[') {
        let Some(close) = rest.find(']')
        else {
            return Err(PeWriterError::ExternalTypeOwnerMissingBracket(owner.to_string()));
        };
        let assembly_name = rest[..close].trim().to_string();
        let full_type_name = rest[close + 1..].trim().to_string();
        if assembly_name.is_empty() || full_type_name.is_empty() {
            return Err(PeWriterError::InvalidExternalTypeOwner(owner.to_string()));
        }
        return Ok((assembly_name, full_type_name));
    }

    Err(PeWriterError::UnsupportedExternalTypeOwner(owner.to_string()))
}

fn normalize_external_type_owner_alias(owner: &str) -> &str {
    match owner {
        "bool" => "[mscorlib]System.Boolean",
        "char" => "[mscorlib]System.Char",
        "int8" => "[mscorlib]System.SByte",
        "uint8" => "[mscorlib]System.Byte",
        "int16" => "[mscorlib]System.Int16",
        "uint16" => "[mscorlib]System.UInt16",
        "int32" => "[mscorlib]System.Int32",
        "uint32" => "[mscorlib]System.UInt32",
        "int64" => "[mscorlib]System.Int64",
        "uint64" => "[mscorlib]System.UInt64",
        "float32" => "[mscorlib]System.Single",
        "float64" => "[mscorlib]System.Double",
        "string" => "[mscorlib]System.String",
        "object" | "Object" => "[mscorlib]System.Object",
        // compile-time attribute namespace：无 CLR 类型定义，降级为 System.Object 占位。
        "marker" => "[mscorlib]System.Object",
        _ => owner,
    }
}

/// External TypeRefs that must use ELEMENT_TYPE_VALUETYPE in method signatures.
fn is_external_valuetype_type_ref(owner: &str) -> bool {
    let type_name = owner.rsplit_once(']').map(|(_, rest)| rest.trim()).unwrap_or(owner);
    matches!(
        type_name,
        "System.IO.SearchOption"
            | "System.Boolean"
            | "System.Char"
            | "System.SByte"
            | "System.Byte"
            | "System.Int16"
            | "System.UInt16"
            | "System.Int32"
            | "System.UInt32"
            | "System.Int64"
            | "System.UInt64"
            | "System.Single"
            | "System.Double"
            | "System.IntPtr"
            | "System.UIntPtr"
            | "System.Decimal"
            | "System.DateTime"
            | "System.Guid"
    )
}

fn split_namespace_and_name(full_type_name: &str) -> (String, String) {
    if let Some((namespace_name, type_name)) = full_type_name.rsplit_once('.') {
        (namespace_name.to_string(), type_name.to_string())
    }
    else {
        (String::new(), full_type_name.to_string())
    }
}

fn to_u16(value: u32, what: &str) -> Result<u16, PeWriterError> {
    u16::try_from(value).map_err(|_| PeWriterError::MetadataIndexOverflow { what: what.to_string(), value })
}
