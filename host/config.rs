use crate::*;

pub struct CrateConfig {
    pub name: String,
    pub version: semver::Version,
    pub persistent: bool,
    pub project_dirs: ProjectDirs,
}

pub static CRATE_CONFIG: std::sync::OnceLock<CrateConfig> = std::sync::OnceLock::new();

pub trait CrateConfigAccess {
    fn init(&self, manifest: &'static str) -> &CrateConfig;
    fn check(&self) -> &CrateConfig;
}

impl CrateConfigAccess for std::sync::OnceLock<CrateConfig> {
    fn init(&self, manifest: &str) -> &CrateConfig {
        let parsed = manifest.parse::<prest::TomlTable>().unwrap();
        let name = parsed["package"]["name"]
            .as_str()
            .unwrap()
            .replace("-", "_");
        let version = parsed["package"]
            .get("version")
            .map(|v| v.as_str().unwrap())
            .unwrap_or("0.0.0")
            .parse::<semver::Version>()
            .unwrap();
        let prest_configs = parsed.get("prest");
        let persistent = if let Some(Some(Some(value))) =
            prest_configs.map(|cfgs| cfgs.get("persistent").map(|v| v.as_bool()))
        {
            value
        } else {
            true
        };

        let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
        
        self.get_or_init(|| {
            CrateConfig { name, version, persistent, project_dirs }
        })
    }

    fn check(&self) -> &CrateConfig {
        self.get().expect("config should be initialized first")
    }
}
