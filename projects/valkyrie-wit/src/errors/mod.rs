use crate::bindings::ToolsError;

impl From<anyhow::Error> for ToolsError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}


impl From<wat::Error> for ToolsError {
    fn from(error: wat::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}
