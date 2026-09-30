//! Profile: reusable single-execution configuration.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ids::slug_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub tool_id: String,
    pub command_id: String,
    pub parameters: HashMap<String, String>,
    /// When `None`, inherit the active workspace global workdir.
    pub workdir: Option<PathBuf>,
}

impl Profile {
    pub fn new_id(name: &str) -> String {
        slug_id("profile", name)
    }
}
