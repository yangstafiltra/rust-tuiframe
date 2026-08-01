use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::registry::{self, ComponentRegistry};

#[derive(Serialize, Deserialize)]
pub struct RegistryCache {
    pub mtimes: HashMap<String, u64>,
    pub registry: serde_json::Value,
    pub warnings: Vec<String>,
}

impl RegistryCache {
    pub fn is_fresh(&self, dir: &Path) -> bool {
        for entry in registry::toml_files(dir) {
            let Ok(entry) = entry else {
                return false;
            };
            let path = entry.path();
            let Ok(meta) = path.metadata() else {
                return false;
            };
            let Ok(mtime) = meta.modified() else {
                return false;
            };
            let mtime_ms = mtime
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            if self.mtimes.get(&path.to_string_lossy().to_string()) != Some(&mtime_ms) {
                return false;
            }
        }
        true
    }

    pub fn build(reg: &ComponentRegistry, dir: &Path) -> Self {
        let mut mtimes = HashMap::new();
        for entry in registry::toml_files(dir) {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if let Ok(meta) = path.metadata() {
                if let Ok(mtime) = meta.modified() {
                    let mtime_ms = mtime
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    mtimes.insert(path.to_string_lossy().to_string(), mtime_ms);
                }
            }
        }
        let registry = serde_json::to_value(reg).unwrap_or_default();
        Self {
            mtimes,
            registry,
            warnings: reg.warnings().to_vec(),
        }
    }
}
