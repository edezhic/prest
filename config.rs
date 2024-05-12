use crate::*;

pub struct AppConfig {
    pub name: String,
    pub version: semver::Version,
    pub persistent: bool,
    pub domain: String,
}

pub static APP_CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();

pub trait AppConfigAccess {
    fn init(&self, manifest: &'static str) -> &AppConfig;
    fn check(&self) -> &AppConfig;
}

impl AppConfigAccess for std::sync::OnceLock<AppConfig> {
    fn init(&self, manifest: &str) -> &AppConfig {
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


        #[cfg(debug_assertions)]
        let domain = "localhost".to_owned();
        #[cfg(not(debug_assertions))]
        let domain = if let Some(Some(Some(value))) =
            prest_configs.map(|cfgs| cfgs.get("domain").map(|v| v.as_str()))
        {
            value
        } else {
            "localhost"
        }.to_owned();

        self.get_or_init(|| {
            AppConfig { name, version, persistent, domain }
        })
    }

    fn check(&self) -> &AppConfig {
        self.get().expect("config should be initialized first")
    }
}
