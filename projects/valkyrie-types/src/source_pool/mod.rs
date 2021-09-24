use crate::bindings::exports::valkyrie::valkyrie_legacy::source_pool::{Guest, GuestSourceId, GuestSourcePool};

pub struct SourceContext {}

impl Guest for SourceContext {
    type SourcePool = SourcePool;
    type SourceId = SourceId;
}

pub struct SourcePool {}

impl GuestSourcePool for SourcePool {}

pub struct SourceId {}
impl GuestSourceId for SourceId {}
