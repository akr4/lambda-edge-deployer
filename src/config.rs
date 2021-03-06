use crate::models::*;
use serde_derive::Deserialize;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub functions: Vec<Function>,
}

type Result<T> = std::result::Result<T, failure::Error>;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut file = std::fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    toml::from_str(&content).map_err(|e| failure::err_msg(e.to_string()))
}
