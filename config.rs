use std::{ops::Deref, sync::OnceLock};

use crate::*;

/// Holds initialized [`AppConfig`], requires arguments from the init macro to initialize so cant be Lazy
pub static APP_CONFIG: AppConfig = AppConfig::new();

/// Holds basic information about the app
pub struct AppConfig {
    info: OnceLock<AppConfigInfo>,
}

#[derive(Debug)]
pub struct AppConfigInfo {
    pub name: &'static str,
    pub version: semver::Version,
    pub persistent: bool,
    pub domain: Option<&'static str>,
    pub manifest_dir: &'static str,
    #[cfg(host)]
    pub data_dir: std::path::PathBuf,
}

impl AppConfig {
    const fn new() -> Self {
        Self {
            info: OnceLock::new(),
        }
    }

    pub fn _init(
        &self,
        manifest_dir: &'static str,
        name: &'static str,
        version: &str,
        persistent: bool,
        domain: Option<&'static str>,
    ) {
        let version = version.parse::<semver::Version>().unwrap();

        #[cfg(host)]
        let data_dir = {
            let project_dirs = prest::ProjectDirs::from("", "", name).unwrap();
            let path = project_dirs.data_dir().to_path_buf();
            std::fs::create_dir_all(&path).unwrap();
            path
        };

        self.info
            .set(AppConfigInfo {
                name,
                version,
                persistent,
                domain,
                manifest_dir,
                #[cfg(host)]
                data_dir,
            })
            .expect("App config should initialize");
    }
}

impl Deref for AppConfig {
    type Target = AppConfigInfo;

    fn deref(&self) -> &Self::Target {
        self.info.get().expect("App config should be initialized. Did you forget to add `#[init]` macro to the main function?")
    }
}
