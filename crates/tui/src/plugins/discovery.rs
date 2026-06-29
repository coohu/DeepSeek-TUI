use std::path::{Path, PathBuf};

use super::manifest::{LoadedPlugin, PluginManifest};
use super::registry::PluginRegistry;

const PLUGIN_MANIFEST: &str = "plugin.toml";

pub fn default_user_plugins_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".deepseek").join("plugins"))
        .unwrap_or_else(|| PathBuf::from("/tmp/deepseek/plugins"))
}

pub fn discover_all(builtin_dirs: &[&str]) -> PluginRegistry {
    let mut registry = PluginRegistry::new();

    for dir in builtin_dirs {
        let path = PathBuf::from(dir);
        if path.exists() {
            discover_from_dir(&path, &mut registry, true);
        }
    }

    let user_dir = default_user_plugins_dir();
    if user_dir.exists() {
        discover_from_dir(&user_dir, &mut registry, false);
    }

    registry
}

fn discover_from_dir(dir: &Path, registry: &mut PluginRegistry, builtin: bool) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join(PLUGIN_MANIFEST);
        if !manifest_path.exists() {
            continue;
        }

        match PluginManifest::from_path(&manifest_path) {
            Ok(manifest) => {
                if !manifest.check_when() {
                    continue;
                }

                let name = manifest.plugin.name.clone();
                let plugin = LoadedPlugin {
                    manifest,
                    base_path: path,
                    enabled: !builtin,
                };

                registry.register(name, plugin);
            }
            Err(e) => {
                tracing::warn!("Failed to load plugin from {}: {}", path.display(), e);
            }
        }
    }
}
