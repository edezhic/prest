use crate::*;

/// Starting point for prest apps that performs basic setup
#[macro_export]
macro_rules! init {
    ($(tables $($table:ident),+)?) => {
        {
            let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
            #[cfg(not(target_arch = "wasm32"))] {
                let _ = prest::dotenv();
                let _ = *prest::SYSTEM_INFO;
            }
            
            let config = APP_CONFIG.init(manifest, env!("CARGO_MANIFEST_DIR"));
            
            #[cfg(not(target_arch = "wasm32"))] {
                prest::init_tracing_subscriber();
                prest::DB.init();
                $(
                    $( $table::prepare_table(); )+
                )?    
                let _ = *prest::RT;
                #[cfg(unix)] {
                    let _ = *prest::SHUTDOWN;
                    prest::RT.spawn(listen_shutdown());
                }
            }

            prest::info!("Initialized {} v{}", config.name, config.version);
        }
    };
}

/// Holds basic information about the app
pub struct AppConfig {
    pub name: String,
    pub version: semver::Version,
    pub persistent: bool,
    pub domain: Option<String>,
    pub manifest_dir: String,
}

/// Holds initialized [`AppConfig`]
pub static APP_CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();

/// Interface for the [`APP_CONFIG`]
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

        #[cfg(host)]
        {
            let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
            let path = project_dirs.data_dir().to_path_buf();
            std::fs::create_dir_all(path).unwrap();
        }

        self.get_or_init(|| AppConfig {
            name,
            version,
            persistent,
            domain,
            manifest_dir: manifest_dir.to_owned(),
        })
    }

    fn check(&self) -> &AppConfig {
        self.get()
            .expect("config should be initialized first. Did you forget to run init! macro?")
    }
}
