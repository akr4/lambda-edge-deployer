use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Function {
    pub name: String,
    pub bundle: PathBuf,
}
