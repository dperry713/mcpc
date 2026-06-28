use serde::{Deserialize, Serialize};

fn default_stage() -> String {
    "development".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteConnection {
    pub name: String,
    pub url: String,
    pub auth_flow: String,
    pub pkce: bool,
    pub audience: String,
    pub scope: String,
    #[serde(default)]
    pub jit_escalation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPSpec {
    pub project: String,
    #[serde(default = "default_stage")]
    pub stage: String,
    pub modules: Vec<Module>,
    #[serde(rename = "_meta", default)]
    pub meta: Option<serde_json::Value>,
    #[serde(default)]
    pub connections: Vec<RemoteConnection>,
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
    #[serde(rename = "_meta", default)]
    pub meta: Option<serde_json::Value>,
}