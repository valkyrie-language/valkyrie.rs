use std::collections::BTreeMap;

use crate::{
    metadata::ClrMetadataBuilder,
    msil::{MsilInstructionOperand, MsilMethodRef, MsilMethodSignature, MsilModule},
};

use super::{
    metadata::{build_method_signature, AssemblyRefRow, MemberRefRow, TypeRefRow},
    PeWriterError,
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
    let mut assembly_ref_map = BTreeMap::<String, u16>::new();
    let mut type_ref_map = BTreeMap::<(String, String), u16>::new();
    let mut member_ref_map = BTreeMap::<(u16, String, MsilMethodSignature), u16>::new();

    // 预构建字段 token 映射：(type_name, field_name) -> FieldDef token。
    // Field 表行号按类型定义顺序、字段声明顺序分配（1-indexed）。
    let field_token_map = build_field_token_map(module);

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
            &local_type_names,
            &mut assembly_ref_rows,
            &mut type_ref_rows,
            &mut member_ref_rows,
            &mut assembly_ref_map,
            &mut type_ref_map,
            &mut member_ref_map,
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
                &local_type_names,
                &mut assembly_ref_rows,
                &mut type_ref_rows,
                &mut member_ref_rows,
                &mut assembly_ref_map,
                &mut type_ref_map,
                &mut member_ref_map,
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

#[allow(clippy::too_many_arguments)]
fn resolve_method_tokens(
    method: &mut crate::msil::MsilMethodBody,
    metadata: &mut ClrMetadataBuilder,
    local_method_tokens: &BTreeMap<String, u32>,
    field_token_map: &BTreeMap<(String, String), u32>,
    local_type_names: &std::collections::BTreeSet<String>,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    member_ref_rows: &mut Vec<MemberRefRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
    member_ref_map: &mut BTreeMap<(u16, String, MsilMethodSignature), u16>,
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
                    local_type_names,
                    assembly_ref_rows,
                    type_ref_rows,
                    member_ref_rows,
                    assembly_ref_map,
                    type_ref_map,
                    member_ref_map,
                    module_name,
                )?;
                MsilInstructionOperand::Token(token)
            }
            MsilInstructionOperand::Type(type_name) => {
                let type_token = ensure_type_ref_token(&type_name, metadata, assembly_ref_rows, type_ref_rows, assembly_ref_map, type_ref_map)?;
                MsilInstructionOperand::Token(type_token)
            }
            MsilInstructionOperand::Field(type_name, field_name) => {
                // 解析字段引用为 FieldDef token。
                let key = (type_name.clone(), field_name.clone());
                let token =
                    *field_token_map.get(&key).ok_or_else(|| PeWriterError::MissingLocalMethodToken(format!("{type_name}.{field_name}")))?;
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
    local_type_names: &std::collections::BTreeSet<String>,
    assembly_ref_rows: &mut Vec<AssemblyRefRow>,
    type_ref_rows: &mut Vec<TypeRefRow>,
    member_ref_rows: &mut Vec<MemberRefRow>,
    assembly_ref_map: &mut BTreeMap<String, u16>,
    type_ref_map: &mut BTreeMap<(String, String), u16>,
    member_ref_map: &mut BTreeMap<(u16, String, MsilMethodSignature), u16>,
    module_name: &str,
) -> Result<u32, PeWriterError> {
    // 判断是否为本地方法引用：owner 为 None（全局方法）、等于模块名、或匹配用户类型名。
    let is_local = match method_ref.owner.as_deref() {
        None => true,
        Some(owner) => owner == module_name || local_type_names.contains(owner),
    };
    if is_local {
        // 本地方法：先尝试限定名 `TypeName.method_name`，再尝试裸 `method_name`。
        let normalized_name = method_ref.name.trim_start_matches('.');
        let mut lookup_keys = Vec::new();
        if let Some(owner) = method_ref.owner.as_deref() {
            if local_type_names.contains(owner) {
                lookup_keys.push(format!("{}.{}", owner, method_ref.name));
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

    let name_offset = to_u16(metadata.strings.add(&method_ref.name), "MemberRef 名称")?;
    // MemberRef 签名中的 `Named` 类型引用外部类型，不使用本地 TypeDef token 映射。
    let empty_type_token_map = BTreeMap::new();
    let signature_offset = to_u16(
        metadata.blob.add(&build_method_signature(&method_ref.signature, method_ref.name == ".ctor", &empty_type_token_map)),
        "MemberRef 签名",
    )?;
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
        let name_offset = to_u16(metadata.strings.add(&assembly_name), "AssemblyRef 名称")?;
        assembly_ref_rows.push(AssemblyRefRow { name_offset });
        assembly_ref_map.insert(assembly_name.clone(), row);
        row
    };

    let (type_namespace, type_name) = split_namespace_and_name(&full_type_name);
    let resolution_scope = (assembly_row << 2) | 0x0002;
    let row = to_u16(type_ref_rows.len() as u32 + 1, "TypeRef 行号")?;
    type_ref_rows.push(TypeRefRow {
        resolution_scope,
        type_name_offset: to_u16(metadata.strings.add(&type_name), "TypeRef 名称")?,
        type_namespace_offset: to_u16(metadata.strings.add(&type_namespace), "TypeRef 命名空间")?,
    });
    type_ref_map.insert(key, row);
    Ok(0x0100_0000 | u32::from(row))
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
        "object" => "[mscorlib]System.Object",
        _ => owner,
    }
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
