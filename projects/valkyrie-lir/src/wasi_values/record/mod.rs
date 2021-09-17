use super::*;
use crate::WasiRecordType;
use indexmap::IndexMap;
use std::sync::Arc;
use valkyrie_types::Identifier;

/// A [record]() value in WASI.
pub struct RecordValue {
    /// The type info of the record
    pub r#type: WasiRecordType,
    /// The override values of the record
    pub values: IndexMap<Identifier, WasiValue>,
}

impl EmitConstant for RecordValue {
    fn emit_constant<W: Write>(&self, w: &mut WastEncoder<W>) -> std::fmt::Result {
        write!(w, "struct.new_default {}", self.r#type.symbol)?;
        // local.set ...
        w.stack.push(WasiType::Record(self.r#type.clone()));

        // struct.set ...

        Ok(())
    }
}
