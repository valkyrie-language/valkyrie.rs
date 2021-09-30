use crate::state::ServerState;
use core::range::Range;
use oak_lsp::types::*;
use serde::{Deserialize, Serialize};

/// 格式化选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattingOptions {
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub trim_trailing_whitespace: Option<bool>,
    pub insert_final_newline: Option<bool>,
    pub trim_final_newlines: Option<bool>,
}

/// 格式化处理器
pub struct FormattingHandler;

impl FormattingHandler {
    pub async fn handle(state: &ServerState, uri: &str, options: FormattingOptions) -> Vec<TextEdit> {
        let doc = match state.get_document(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let mut formatted = doc.text.clone();

        if options.trim_trailing_whitespace.unwrap_or(false) {
            let tree_ends_with_newline = formatted.ends_with('\n');
            formatted = formatted.lines().map(|line| line.trim_end()).collect::<Vec<_>>().join("\n");
            if tree_ends_with_newline && !formatted.ends_with('\n') {
                formatted.push('\n');
            }
        }

        if options.insert_final_newline.unwrap_or(false) {
            if !formatted.ends_with('\n') {
                formatted.push('\n');
            }
        }
        else if options.trim_final_newlines.unwrap_or(false) {
            while formatted.ends_with("\n\n") {
                formatted.pop();
            }
        }

        if formatted == doc.text {
            return vec![];
        }

        vec![TextEdit { range: Range { start: 0, end: doc.text.len() }, new_text: formatted }]
    }
}
