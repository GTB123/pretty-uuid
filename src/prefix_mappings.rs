use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct PrefixMappings {
   pub mappings: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read the YAML file")]
    ReadError(#[from] std::io::Error),
    #[error("failed to parse the YAML file")]
    ParseError(#[from] serde_yaml::Error),
}

impl PrefixMappings {
    pub fn new() -> Result<Self, Error> {
        let yaml_content = fs::read_to_string("src/prefix_mappings.yaml")?;
        let mappings: HashMap<String, String> = serde_yaml::from_str(&yaml_content)?;
        Ok(PrefixMappings { mappings })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.mappings.get(key)
    }
}