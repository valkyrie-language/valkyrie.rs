//! dex/035 写出器：将 JVM `.class` 编入可解析 `classes.dex`。

use std::collections::BTreeMap;

use miette::{Result, miette};

use crate::binary::class::JvmClassFile;

const ENDIAN_TAG: u32 = 0x1234_5678;
const HEADER_SIZE: u32 = 0x70;
const NO_INDEX: u32 = 0xFFFF_FFFF;

/// DEX 镜像构建器（真实 dex/035 结构 + 可选尾段）。
#[derive(Debug, Default)]
pub struct DexImageBuilder {
    classes: Vec<(String, Vec<u8>)>,
    tail: Vec<u8>,
}

impl DexImageBuilder {
    /// 新建。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加 JVM class。
    pub fn add_class(&mut self, internal_name: &str, class_bytes: &[u8]) -> &mut Self {
        self.classes.push((internal_name.to_string(), class_bytes.to_vec()));
        self
    }

    /// 追加尾段（ASGARDNT / ASGARDUI 嵌入）。
    pub fn append_tail(&mut self, data: &[u8]) -> &mut Self {
        self.tail.extend_from_slice(data);
        self
    }

    /// 写出 `classes.dex`。
    pub fn build(self) -> Result<Vec<u8>> {
        let mut writer = DexWriter::new();
        for (name, bytes) in &self.classes {
            let class = JvmClassFile::from_bytes(bytes).map_err(|e| miette!("{e}"))?;
            writer.add_class(&class).map_err(|e| miette!("{e}"))?;
            let _ = name;
        }
        let mut dex = writer.finish()?;
        dex.extend_from_slice(&self.tail);
        Ok(dex)
    }
}

struct DexWriter {
    strings: Vec<String>,
    types: Vec<String>,
    protos: Vec<(String, String)>,
    methods: Vec<MethodRef>,
    classes: Vec<ClassDef>,
    string_index: BTreeMap<String, u32>,
    type_index: BTreeMap<String, u32>,
}

struct MethodRef {
    class_type: String,
    name: String,
    descriptor: String,
    access_flags: u32,
    is_native: bool,
    code: Option<Vec<u8>>,
}

struct ClassDef {
    type_name: String,
    super_name: String,
    access_flags: u32,
    methods: Vec<usize>,
}

impl DexWriter {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            types: Vec::new(),
            protos: Vec::new(),
            methods: Vec::new(),
            classes: Vec::new(),
            string_index: BTreeMap::new(),
            type_index: BTreeMap::new(),
        }
    }

    fn add_class(&mut self, class: &JvmClassFile) -> Result<(), String> {
        let type_name = format!("L{};", class.internal_name);
        let super_name = if class.super_name.is_empty() { "Ljava/lang/Object;".to_string() } else { format!("L{};", class.super_name) };
        self.intern_type(&type_name);
        self.intern_type(&super_name);

        let class_idx = self.classes.len();
        let mut method_indices = Vec::new();
        for method in &class.methods {
            let is_native = method.access_flags & 0x0100 != 0;
            let mref = MethodRef {
                class_type: type_name.clone(),
                name: method.name.clone(),
                descriptor: method.descriptor.to_string(),
                access_flags: u32::from(method.access_flags),
                is_native,
                code: if is_native { None } else { method.code.as_ref().map(|c| encode_code_item(c)) },
            };
            method_indices.push(self.methods.len());
            self.intern_string(&mref.name);
            self.intern_proto(&mref.descriptor);
            self.methods.push(mref);
        }
        self.classes.push(ClassDef { type_name, super_name, access_flags: u32::from(class.access_flags), methods: method_indices });
        let _ = class_idx;
        Ok(())
    }

    fn intern_string(&mut self, s: &str) -> u32 {
        if let Some(&idx) = self.string_index.get(s) {
            return idx;
        }
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.string_index.insert(s.to_string(), idx);
        idx
    }

    fn intern_type(&mut self, descriptor: &str) -> u32 {
        if let Some(&idx) = self.type_index.get(descriptor) {
            return idx;
        }
        let idx = self.types.len() as u32;
        self.types.push(descriptor.to_string());
        self.type_index.insert(descriptor.to_string(), idx);
        self.intern_string(descriptor);
        idx
    }

    fn intern_proto(&mut self, descriptor: &str) -> u32 {
        let shorty = descriptor_to_shorty(descriptor);
        if let Some((idx, _)) = self.protos.iter().enumerate().find(|(_, (d, _))| d == descriptor) {
            return idx as u32;
        }
        for ty in types_in_method_descriptor(descriptor) {
            self.intern_type(&ty);
        }
        self.intern_string(&shorty);
        let idx = self.protos.len() as u32;
        self.protos.push((descriptor.to_string(), shorty));
        idx
    }

    fn finish(self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut string_data = Vec::new();
        let mut string_ids = Vec::new();
        for s in &self.strings {
            string_ids.push(string_data.len() as u32);
            write_uleb128(&mut string_data, s.len() as u32);
            string_data.extend_from_slice(s.as_bytes());
        }

        let mut type_ids = Vec::new();
        for t in &self.types {
            type_ids.push(self.string_index[t]);
        }

        let mut proto_ids = Vec::new();
        for (desc, shorty) in &self.protos {
            let return_type = desc.rsplit(')').next().unwrap_or("V");
            proto_ids.push((self.string_index[shorty], self.type_index[return_type], 0u32));
        }

        let mut method_ids = Vec::new();
        for m in &self.methods {
            let proto_idx = self.protos.iter().position(|(d, _)| d == &m.descriptor).unwrap_or(0) as u32;
            method_ids.push((self.type_index[&m.class_type], proto_idx, self.string_index[&m.name]));
        }

        let mut class_data_items = Vec::new();
        let mut code_items = Vec::new();
        let mut class_defs = Vec::new();

        for class in &self.classes {
            let mut cd = Vec::new();
            write_uleb128(&mut cd, 0);
            write_uleb128(&mut cd, 0);
            write_uleb128(&mut cd, class.methods.len() as u32);
            write_uleb128(&mut cd, 0);
            for &mid in &class.methods {
                let m = &self.methods[mid];
                write_uleb128(&mut cd, method_ids[mid].2);
                write_uleb128(&mut cd, m.access_flags);
                if m.is_native {
                    write_uleb128(&mut cd, 0);
                }
                else if let Some(code) = &m.code {
                    let off = code_items.len() as u32;
                    code_items.extend_from_slice(code);
                    write_uleb128(&mut cd, off);
                }
                else {
                    write_uleb128(&mut cd, 0);
                }
            }
            let cd_off = class_data_items.len() as u32;
            class_data_items.extend_from_slice(&cd);
            class_defs.push((
                self.type_index[&class.type_name],
                class.access_flags,
                self.type_index[&class.super_name],
                0u32,
                NO_INDEX,
                0u32,
                cd_off,
                0u32,
            ));
        }

        let map_off = 0u32;
        let file_size = HEADER_SIZE
            + string_ids.len() as u32 * 4
            + type_ids.len() as u32 * 4
            + proto_ids.len() as u32 * 12
            + method_ids.len() as u32 * 8
            + class_defs.len() as u32 * 32
            + string_data.len() as u32
            + class_data_items.len() as u32
            + code_items.len() as u32;

        out.extend_from_slice(b"dex\n035\0");
        out.extend_from_slice(&[0u8; 24]); // checksum(4) + signature(20)
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&HEADER_SIZE.to_le_bytes());
        out.extend_from_slice(&ENDIAN_TAG.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // link_size
        out.extend_from_slice(&0u32.to_le_bytes()); // link_off
        out.extend_from_slice(&map_off.to_le_bytes());
        out.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());
        out.extend_from_slice(&HEADER_SIZE.to_le_bytes());
        out.extend_from_slice(&(self.types.len() as u32).to_le_bytes());
        out.extend_from_slice(&(HEADER_SIZE + self.strings.len() as u32 * 4).to_le_bytes());
        out.extend_from_slice(&(self.protos.len() as u32).to_le_bytes());
        out.extend_from_slice(&(HEADER_SIZE + self.strings.len() as u32 * 4 + self.types.len() as u32 * 4).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(self.methods.len() as u32).to_le_bytes());
        let method_ids_off = HEADER_SIZE + self.strings.len() as u32 * 4 + self.types.len() as u32 * 4 + self.protos.len() as u32 * 12;
        out.extend_from_slice(&method_ids_off.to_le_bytes());
        out.extend_from_slice(&(self.classes.len() as u32).to_le_bytes());
        let class_defs_off = method_ids_off + self.methods.len() as u32 * 8;
        out.extend_from_slice(&class_defs_off.to_le_bytes());
        let data_off = class_defs_off + self.classes.len() as u32 * 32;
        out.extend_from_slice(&(string_data.len() as u32 + class_data_items.len() as u32 + code_items.len() as u32).to_le_bytes());
        out.extend_from_slice(&data_off.to_le_bytes());

        for id in &string_ids {
            out.extend_from_slice(&(data_off + id).to_le_bytes());
        }
        for id in &type_ids {
            out.extend_from_slice(&id.to_le_bytes());
        }
        for (shorty_idx, return_idx, params_off) in &proto_ids {
            out.extend_from_slice(&shorty_idx.to_le_bytes());
            out.extend_from_slice(&return_idx.to_le_bytes());
            out.extend_from_slice(&params_off.to_le_bytes());
        }
        for (class_idx, proto_idx, name_idx) in &method_ids {
            out.extend_from_slice(&class_idx.to_le_bytes());
            out.extend_from_slice(&proto_idx.to_le_bytes());
            out.extend_from_slice(&name_idx.to_le_bytes());
        }
        for (class_idx, access, super_idx, interfaces, source, annotations, class_data_off, static_values) in &class_defs {
            out.extend_from_slice(&class_idx.to_le_bytes());
            out.extend_from_slice(&access.to_le_bytes());
            out.extend_from_slice(&super_idx.to_le_bytes());
            out.extend_from_slice(&interfaces.to_le_bytes());
            out.extend_from_slice(&source.to_le_bytes());
            out.extend_from_slice(&annotations.to_le_bytes());
            out.extend_from_slice(&(data_off + string_data.len() as u32 + class_data_off).to_le_bytes());
            out.extend_from_slice(&static_values.to_le_bytes());
        }

        out.extend_from_slice(&string_data);
        out.extend_from_slice(&class_data_items);
        out.extend_from_slice(&code_items);

        Ok(out)
    }
}

fn types_in_method_descriptor(descriptor: &str) -> Vec<String> {
    let mut types = Vec::new();
    let Some((params, ret)) = descriptor.split_once(')')
    else {
        return types;
    };
    let mut chars = params.trim_start_matches('(').chars().peekable();
    while chars.peek().is_some() {
        match chars.next().unwrap() {
            'L' => {
                let mut name = String::from("L");
                while let Some(ch) = chars.next() {
                    name.push(ch);
                    if ch == ';' {
                        break;
                    }
                }
                types.push(name);
            }
            '[' => {
                let mut array = String::from("[");
                if chars.peek() == Some(&'L') {
                    chars.next();
                    array.push('L');
                    while let Some(ch) = chars.next() {
                        array.push(ch);
                        if ch == ';' {
                            break;
                        }
                    }
                }
                else if let Some(base) = chars.next() {
                    array.push(base);
                }
                types.push(array);
            }
            other => types.push(other.to_string()),
        }
    }
    if !ret.is_empty() {
        types.push(ret.to_string());
    }
    types
}

fn descriptor_to_shorty(descriptor: &str) -> String {
    let mut shorty = String::new();
    let params = descriptor.split(')').next().unwrap_or("").trim_start_matches('(');
    let mut chars = params.chars().peekable();
    while chars.peek().is_some() {
        match chars.next().unwrap() {
            'L' => {
                shorty.push('L');
                while chars.next().is_some() && chars.peek() != Some(&';') {}
            }
            '[' => shorty.push('L'),
            'Z' => shorty.push('Z'),
            'B' => shorty.push('B'),
            'C' => shorty.push('C'),
            'S' => shorty.push('S'),
            'I' => shorty.push('I'),
            'J' => shorty.push('J'),
            'F' => shorty.push('F'),
            'D' => shorty.push('D'),
            _ => shorty.push('L'),
        }
    }
    let ret = descriptor.rsplit(')').next().unwrap_or("V");
    shorty.push(match ret.chars().next().unwrap_or('V') {
        'V' => 'V',
        'Z' => 'Z',
        'I' => 'I',
        'J' => 'J',
        'F' => 'F',
        'D' => 'D',
        _ => 'L',
    });
    shorty
}

fn encode_code_item(code: &crate::binary::class::JvmCodeBody) -> Vec<u8> {
    let insns = code.instructions.iter().flat_map(|_| [0u8, 0u8]).collect::<Vec<_>>();
    let mut out = Vec::new();
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(insns.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&insns);
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn write_uleb128(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::class::{JvmClassFile, JvmMethodDescriptor, JvmTypeDescriptor};

    #[test]
    fn dex_has_class_defs_and_magic() {
        let mut bridge = JvmClassFile::new("com/asgard/runtime/AsgardHostBridge");
        bridge.access_flags = 0x0031;
        bridge.push_method(
            "invokeExport",
            JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".into())], JvmTypeDescriptor::Void),
        );
        bridge.methods[0].access_flags = 0x0109;
        let bytes = bridge.to_bytes().expect("class");
        let mut builder = DexImageBuilder::new();
        builder.add_class("com/asgard/runtime/AsgardHostBridge", &bytes);
        let dex = builder.build().expect("dex");
        assert!(dex.starts_with(b"dex\n035"));
        assert!(dex.windows(b"AsgardHostBridge".len()).any(|w| w == b"AsgardHostBridge"));
    }
}
