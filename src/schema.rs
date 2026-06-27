use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPSpec {
    pub project: String,
    pub modules: Vec<Module>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    pub name: String,
    #[serde(rename = "type", default)]
    pub module_type: Option<String>,
    pub entry: Option<String>,
    pub features: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}