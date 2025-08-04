use crate::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub exclude: Vec<String>,
}

pub fn load_config(root_path: &Path) -> Result<Config> {
    let config_path = root_path.join(".devcatrc");
    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
