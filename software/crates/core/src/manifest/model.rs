//! Manifest schema v1 (Serde + JSON).

use serde::{Deserialize, Serialize};

/// Root document describing a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub executable: String,
    #[serde(default)]
    pub commands: Vec<Command>,
}

/// Operation exposed by a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Fixed argv inserted after the executable and before parameters.
    #[serde(default)]
    pub argv_prefix: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

/// Input of a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// Supported parameter kinds for schema v1 (Phase 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Bool,
    Int,
    Path,
}
