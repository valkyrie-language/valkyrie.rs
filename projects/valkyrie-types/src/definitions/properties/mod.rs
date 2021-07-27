use super::*;

/// A property is a field that can be accessed by a getter and setter
#[derive(Clone, Debug)]
pub struct ValkyrieProperty {
    /// The field name of this property
    name: String,
    /// The type of the property
    initial: InitializeType,
    /// The type of the property
    typing: ValkyrieMetaType,
    /// Can be a progress or a value
    default: Option<ExpressionNode>,
}
