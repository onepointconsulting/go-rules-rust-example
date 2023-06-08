use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct MainConfig {
    pub server_addr: String,
    pub rules_folder: String
}