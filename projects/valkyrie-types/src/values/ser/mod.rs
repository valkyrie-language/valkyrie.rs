use crate::ValkyrieValue;
use serde::{Serialize, Serializer};

impl Serialize for ValkyrieValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Nothing => {
                todo!()
            }
            Self::Null => serializer.serialize_none(),
            Self::Unit => serializer.serialize_unit(),
            Self::Boolean(v) => serializer.serialize_bool(*v),
            Self::Number(v) => v.serialize(serializer),
            Self::Unicode(v) => serializer.serialize_char(*v),
            Self::UTF8String(v) => serializer.serialize_str(&v.get()),
            Self::Bytes(_) => {
                todo!()
            }
            Self::Class(_) => {
                todo!()
            }
            Self::Variant(_) => {
                todo!()
            }
            Self::Image(_) => {
                todo!()
            }
            Self::List(v) => v.serialize(serializer),
            Self::Dict(v) => serializer.collect_map(v.raw.iter()),
            Self::Html(_) => {
                todo!()
            }
        }
    }
}
