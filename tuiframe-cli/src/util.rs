use std::path::{Path, PathBuf};
use tuiframe_core::ComponentRegistry;

pub fn project_root() -> PathBuf {
    if let Ok(dir) = std::env::var("TUIFRAME_DIR") {
        return PathBuf::from(dir);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tuiframe-cli should be inside a workspace")
        .to_path_buf()
}

pub fn load_registry() -> anyhow::Result<ComponentRegistry> {
    let root = project_root();
    let comp_dir = root.join("components");
    let cache_path = root.join(".tuiframe_cache.json");
    let reg = ComponentRegistry::load_from_dir_cached(&comp_dir, &cache_path).map_err(|e| {
        anyhow::anyhow!(
            "{e}\n[help]  Set TUIFRAME_DIR to the project root, or run from the workspace."
        )
    })?;
    for warn in reg.warnings() {
        eprintln!("  [warn] {warn}");
    }
    Ok(reg)
}

pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !name.contains("..")
        && !name.contains('\0')
        && name != "."
        && name != ".."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_root_exists() {
        let root = project_root();
        assert!(root.exists());
        assert!(root.join("Cargo.toml").exists());
    }

    #[test]
    fn test_load_registry_ok() {
        let reg = load_registry();
        assert!(reg.is_ok());
    }

    #[test]
    fn test_is_safe_name_valid() {
        assert!(is_safe_name("my-project"));
        assert!(is_safe_name("hello_world"));
        assert!(is_safe_name("a"));
        assert!(is_safe_name("abc123"));
    }

    #[test]
    fn test_is_safe_name_invalid() {
        assert!(!is_safe_name(""));
        assert!(!is_safe_name("../foo"));
        assert!(!is_safe_name("foo/bar"));
        assert!(!is_safe_name("foo\\bar"));
        assert!(!is_safe_name("."));
        assert!(!is_safe_name(".."));
        assert!(!is_safe_name("foo\0bar"));
        assert!(!is_safe_name("foo bar"));
    }
}
