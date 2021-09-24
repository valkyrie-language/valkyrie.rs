#[derive(Debug)]
pub enum LinearizeError {
    Circular { node: String },
    NotFound,
}
