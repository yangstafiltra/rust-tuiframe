use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentDef {
    pub name: String,
    pub category: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
    pub example: String,
    #[serde(default)]
    pub snippet: String,
    #[serde(default)]
    pub reference_apps: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ComponentRegistry {
    categories: Vec<String>,
    components: HashMap<String, Vec<ComponentDef>>,
    by_name: HashMap<String, ComponentDef>,
    #[serde(skip)]
    warnings: Vec<String>,
}

pub(crate) fn toml_files(dir: &Path) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
    walkdir::WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter(|entry| {
            entry.as_ref().is_ok_and(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|e| e == "toml")
            })
        })
}

impl ComponentRegistry {
    pub fn load_from_dir(dir: &Path) -> anyhow::Result<Self> {
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if !dir.exists() {
            anyhow::bail!("Components directory not found: {}", dir.display());
        }

        let mut reg = Self::default();

        for entry in toml_files(&dir) {
            let entry = entry?;
            let path = entry.path();
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    reg.warnings
                        .push(format!("skipping {}: {e}", path.display()));
                    continue;
                }
            };
            let def: ComponentDef = match toml::from_str(&content) {
                Ok(d) => d,
                Err(e) => {
                    reg.warnings
                        .push(format!("skipping {}: {e}", path.display()));
                    continue;
                }
            };
            if reg.by_name.contains_key(&def.name) {
                reg.warnings.push(format!(
                    "duplicate component name '{}' (at {}), keeping first definition",
                    def.name,
                    path.display()
                ));
                continue;
            }
            reg.by_name.insert(def.name.clone(), def.clone());
            reg.components
                .entry(def.category.clone())
                .or_default()
                .push(def);
        }

        let mut cats: Vec<String> = reg.components.keys().cloned().collect();
        cats.sort();
        reg.categories = cats;
        for comps in reg.components.values_mut() {
            comps.sort_by(|a, b| a.name.cmp(&b.name));
        }

        Ok(reg)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    #[must_use]
    pub fn category_count(&self) -> usize {
        self.categories.len()
    }

    #[must_use]
    pub fn component_count(&self) -> usize {
        self.by_name.len()
    }

    #[must_use]
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    #[must_use]
    pub fn get_category(&self, idx: usize) -> Option<&str> {
        self.categories.get(idx).map(String::as_str)
    }

    #[must_use]
    pub fn get_component(&self, name: &str) -> Option<&ComponentDef> {
        self.by_name.get(name)
    }

    #[must_use]
    pub fn contains_name(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    #[must_use]
    pub fn components_for_category(&self, name: &str) -> Option<&[ComponentDef]> {
        self.components.get(name).map(Vec::as_slice)
    }

    #[must_use]
    pub fn get_components_in_category(&self, idx: usize) -> Option<&[ComponentDef]> {
        let cat = self.categories.get(idx)?;
        self.components.get(cat.as_str()).map(Vec::as_slice)
    }

    pub fn iter_categories(&self) -> impl Iterator<Item = (&str, &[ComponentDef])> + use<'_> {
        self.categories.iter().filter_map(|cat| {
            let comps = self.components.get(cat.as_str())?;
            if comps.is_empty() {
                None
            } else {
                Some((cat.as_str(), comps.as_slice()))
            }
        })
    }

    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    #[must_use]
    pub fn validate_dependencies(&self) -> Vec<(String, Vec<&str>)> {
        let mut issues = Vec::new();
        for comp in self.by_name.values() {
            let missing: Vec<&str> = comp
                .dependencies
                .iter()
                .filter(|dep| !self.by_name.contains_key(dep.as_str()))
                .map(String::as_str)
                .collect();
            if !missing.is_empty() {
                issues.push((comp.name.clone(), missing));
            }
        }
        issues
    }

    #[must_use]
    pub fn dependents(&self, name: &str) -> Vec<&ComponentDef> {
        self.by_name
            .values()
            .filter(|c| c.dependencies.iter().any(|d| d == name))
            .collect()
    }

    #[must_use]
    pub fn search(&self, keyword: &str) -> Vec<(&str, &ComponentDef)> {
        let kw = keyword.to_lowercase();
        let mut results: Vec<_> = self
            .by_name
            .iter()
            .filter(|(_, c)| {
                c.name.to_lowercase().contains(&kw)
                    || c.category.to_lowercase().contains(&kw)
                    || c.description.to_lowercase().contains(&kw)
                    || c.dependencies
                        .iter()
                        .any(|d| d.to_lowercase().contains(&kw))
                    || c.features.iter().any(|f| f.to_lowercase().contains(&kw))
            })
            .map(|(_, c)| (c.category.as_str(), c))
            .collect();
        results.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        results
    }

    pub fn load_from_dir_cached(dir: &Path, cache_path: &Path) -> anyhow::Result<Self> {
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if !dir.exists() {
            anyhow::bail!("Components directory not found: {}", dir.display());
        }

        if let Ok(cache_content) = std::fs::read_to_string(cache_path) {
            if let Ok(cache) = serde_json::from_str::<crate::cache::RegistryCache>(&cache_content) {
                if cache.is_fresh(&dir) {
                    if let Ok(reg) = serde_json::from_value::<Self>(cache.registry) {
                        let mut reg = reg;
                        reg.warnings = cache.warnings;
                        return Ok(reg);
                    }
                }
            }
        }

        let reg = Self::load_from_dir(&dir)?;
        let cache = crate::cache::RegistryCache::build(&reg, &dir);
        if let Ok(json) = serde_json::to_string(&cache) {
            let _ = std::fs::write(cache_path, json);
        }

        Ok(reg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    fn write_component(dir: &Path, filename: &str, content: &str) {
        std::fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn load_from_dir_nonexistent() {
        let result = ComponentRegistry::load_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn load_from_empty_dir() {
        let tmp = temp_dir();
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        assert!(reg.is_empty());
        assert!(reg.categories().is_empty());
        assert_eq!(reg.component_count(), 0);
        assert!(reg.warnings().is_empty());
    }

    #[test]
    fn parse_a_component_toml() {
        let content = r#"
name = "test_comp"
category = "test"
description = "A test component"
dependencies = ["foo", "bar"]
features = ["fast", "reliable"]
example = "println!(\"hello\");"
reference_apps = ["app1", "app2"]
"#;
        let def: ComponentDef = toml::from_str(content).unwrap();
        assert_eq!(def.name, "test_comp");
        assert_eq!(def.dependencies, vec!["foo", "bar"]);
        assert_eq!(def.features, vec!["fast", "reliable"]);
        assert_eq!(def.example, "println!(\"hello\");");
        assert_eq!(def.reference_apps, vec!["app1", "app2"]);
    }

    #[test]
    fn parse_component_without_optional_fields() {
        let content = r#"
name = "minimal"
category = "test"
description = "minimal"
dependencies = []
features = []
example = ""
"#;
        let def: ComponentDef = toml::from_str(content).unwrap();
        assert!(def.reference_apps.is_empty());
        assert!(def.snippet.is_empty());
    }

    #[test]
    fn parse_component_rejects_unknown_fields() {
        let content = r#"
name = "bad"
category = "test"
description = "bad"
dependencies = []
features = []
example = ""
typo_field = "should fail"
"#;
        let result: Result<ComponentDef, _> = toml::from_str(content);
        assert!(result.is_err(), "unknown fields should be rejected");
    }

    #[test]
    fn by_name_index() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "alpha.toml",
            "name = \"alpha\"\ncategory = \"test\"\ndescription = \"\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        write_component(
            tmp.path(),
            "beta.toml",
            "name = \"beta\"\ncategory = \"test\"\ndescription = \"\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        assert!(reg.contains_name("alpha"));
        assert!(reg.contains_name("beta"));
        assert_eq!(reg.component_count(), 2);
    }

    #[test]
    fn get_category_out_of_bounds() {
        let reg = ComponentRegistry::default();
        assert!(reg.get_category(0).is_none());
        assert!(reg.get_components_in_category(0).is_none());
    }

    #[test]
    fn get_components_in_empty_category() {
        let reg = ComponentRegistry::default();
        assert!(reg.get_components_in_category(0).is_none());
    }

    #[test]
    fn duplicate_name_is_skipped() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "a.toml",
            "name = \"dup\"\ncategory = \"x\"\ndescription = \"first\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        write_component(
            tmp.path(),
            "b.toml",
            "name = \"dup\"\ncategory = \"y\"\ndescription = \"second\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        assert_eq!(reg.component_count(), 1);
        let comp = reg.get_component("dup").unwrap();
        assert_eq!(comp.category, "x");
        assert_eq!(comp.description, "first");
        assert_eq!(reg.warnings().len(), 1);
    }

    #[test]
    fn iter_categories_skips_empty() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "a.toml",
            "name = \"a\"\ncategory = \"data\"\ndescription = \"\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        let pairs: Vec<_> = reg.iter_categories().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "data");
        assert_eq!(pairs[0].1.len(), 1);
    }

    #[test]
    fn validate_dependencies_ok() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "a.toml",
            "name = \"a\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = [\"b\"]\nfeatures = []\nexample = \"\"\n",
        );
        write_component(
            tmp.path(),
            "b.toml",
            "name = \"b\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        let issues = reg.validate_dependencies();
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_dependencies_missing() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "a.toml",
            "name = \"a\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = [\"b\", \"nonexistent\"]\nfeatures = []\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        let issues = reg.validate_dependencies();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].0, "a");
        assert!(issues[0].1.contains(&"nonexistent"));
    }

    #[test]
    fn search_by_keyword() {
        let tmp = temp_dir();
        write_component(
            tmp.path(),
            "block.toml",
            "name = \"block\"\ncategory = \"core\"\ndescription = \"A bordered container\"\ndependencies = []\nfeatures = [\"borders\"]\nexample = \"\"\n",
        );
        write_component(
            tmp.path(),
            "table.toml",
            "name = \"table\"\ncategory = \"data\"\ndescription = \"A data table\"\ndependencies = [\"block\"]\nfeatures = [\"sort\"]\nexample = \"\"\n",
        );
        let reg = ComponentRegistry::load_from_dir(tmp.path()).unwrap();
        let results = reg.search("borders");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "block");

        let results = reg.search("sort");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "table");

        let results = reg.search("data");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "table");
    }

    #[test]
    fn component_def_partial_eq() {
        let content = "name = \"x\"\ncategory = \"c\"\ndescription = \"d\"\ndependencies = []\nfeatures = []\nexample = \"e\"\n";
        let a: ComponentDef = toml::from_str(content).unwrap();
        let b: ComponentDef = toml::from_str(content).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn cache_roundtrip() {
        let tmp = temp_dir();
        let cache_file = tmp.path().join("cache.json");
        let comp_dir = tmp.path().join("comps");
        std::fs::create_dir_all(&comp_dir).unwrap();
        write_component(
            &comp_dir,
            "test.toml",
            "name = \"test\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = []\nfeatures = []\nexample = \"\"\n",
        );

        let reg = ComponentRegistry::load_from_dir_cached(&comp_dir, &cache_file).unwrap();
        assert!(reg.contains_name("test"));
        assert!(cache_file.exists());

        let reg2 = ComponentRegistry::load_from_dir_cached(&comp_dir, &cache_file).unwrap();
        assert!(reg2.contains_name("test"));
    }
}
