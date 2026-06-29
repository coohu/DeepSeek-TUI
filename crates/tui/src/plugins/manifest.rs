use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::mcp::McpServerConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    pub skills: Option<PluginSkills>,
    #[serde(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    pub when: Option<PluginWhen>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginSkills {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginWhen {
    pub os: Option<Vec<String>>,
    pub binaries: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub base_path: PathBuf,
    pub enabled: bool,
}

impl PluginManifest {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read plugin.toml: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("failed to parse plugin.toml: {}", e))
    }

    pub fn check_when(&self) -> bool {
        if let Some(when) = &self.when {
            if let Some(os_list) = &when.os {
                let os = std::env::consts::OS;
                if !os_list.iter().any(|o| o.eq_ignore_ascii_case(os)) {
                    return false;
                }
            }
            if let Some(binaries) = &when.binaries {
                for binary in binaries {
                    if !Self::has_binary(binary) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn has_binary(name: &str) -> bool {
        let paths = std::env::var_os("PATH").unwrap_or_default();
        for path in std::env::split_paths(&paths) {
            let candidate = path.join(name);
            if candidate.exists() {
                return true;
            }
            #[cfg(windows)]
            {
                let candidate_exe = candidate.with_extension("exe");
                if candidate_exe.exists() {
                    return true;
                }
            }
        }
        false
    }
}
