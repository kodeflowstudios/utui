use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type", default)]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<ParamValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub package: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

impl Command {
    pub fn from_json(entry: &Value) -> Option<Vec<Self>> {
        let commands = entry.get("commands")?;
        serde_json::from_value(commands.clone()).ok()
    }
}
