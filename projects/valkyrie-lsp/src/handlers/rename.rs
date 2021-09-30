use crate::{state::ServerState, types::Position};
use oak_lsp::types::*;
use std::collections::HashMap;

/// 重命名处理器
pub struct RenameHandler;

impl RenameHandler {
    pub async fn handle(state: &ServerState, uri: &str, position: Position, new_name: String) -> Option<WorkspaceEdit> {
        // 1. 获取所有需要重命名的位置
        let locations = super::ReferencesHandler::handle(state, uri, position).await;

        if locations.is_empty() {
            return None;
        }

        // 2. 按 URI 分组
        let mut changes = HashMap::new();
        for loc in locations {
            let edits = changes.entry(loc.uri.to_string()).or_insert_with(Vec::new);
            edits.push(TextEdit { range: loc.range, new_text: new_name.clone() });
        }

        Some(WorkspaceEdit { changes })
    }
}
