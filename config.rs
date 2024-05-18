use crate::*;

pub struct AppConfig {
    pub name: String,
    pub version: semver::Version,
    pub persistent: bool,
    pub domain: Option<String>,
    pub manifest_dir: String,
}

pub static APP_CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();

pub trait AppConfigAccess {
    fn init(&self, manifest: &'static str, manifest_dir: &'static str) -> &AppConfig;
    fn check(&self) -> &AppConfig;
}

impl AppConfigAccess for std::sync::OnceLock<AppConfig> {
    fn init(&self, manifest: &str, manifest_dir: &str) -> &AppConfig {
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
        let metadata = parsed.get("package").map(|t| t.get("metadata")).flatten();
        let persistent = if let Some(Some(Some(value))) =
            metadata.map(|cfgs| cfgs.get("persistent").map(|v| v.as_bool()))
        {
            value
        } else {
            true
        };

        let domain = if let Some(Some(Some(value))) =
            metadata.map(|cfgs| cfgs.get("domain").map(|v| v.as_str()))
        {
            Some(value.to_owned())
        } else {
            None
        };

        self.get_or_init(|| AppConfig {
            name,
            version,
            persistent,
            domain,
            manifest_dir: manifest_dir.to_owned(),
        })
    }

    fn check(&self) -> &AppConfig {
        self.get().expect("config should be initialized first")
    }
}
