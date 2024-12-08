use crate::*;

/// Starting point for prest apps that performs basic setup
#[macro_export]
macro_rules! init {
    ($(tables $($table:ident),+)?) => {
        let __config = APP_CONFIG.init(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")), env!("CARGO_MANIFEST_DIR"));
        #[cfg(not(target_arch = "wasm32"))]
        let __________ = prest::init_tracing_subscriber();    
        #[cfg(not(target_arch = "wasm32"))] {
            prest::Lazy::force(&RT);
            let _ = prest::dotenv();
            prest::Lazy::force(&DB);
            // initializing here because it starts with the runtime but requires the DB
            prest::ScheduledJobRecord::migrate();
            $(
                $( $table::prepare_table(); )+
            )?
            prest::Lazy::force(&SYSTEM_INFO);
        }
        prest::info!(target: "prest", "Initialized {} v{}", __config.name, __config.version);
    };
}

/// Holds basic information about the app
pub struct AppConfig {
    pub name: String,
    pub version: semver::Version,
    pub persistent: bool,
    pub domain: Option<String>,
    pub manifest_dir: String,
    #[cfg(host)]
    pub data_dir: std::path::PathBuf,
}

/// Holds initialized [`AppConfig`], requires arguments from the init macro to initialize so cant be Lazy
pub static APP_CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();

/// Interface for the [`APP_CONFIG`]
pub trait AppConfigAccess {
    /// Runs inside of the [`init!`] macro to read options from app's manifest
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
        let data_dir = {
            let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
            let path = project_dirs.data_dir().to_path_buf();
            std::fs::create_dir_all(&path).unwrap();
            path
        };

        self.get_or_init(|| AppConfig {
            name,
            version,
            persistent,
            domain,
            manifest_dir: manifest_dir.to_owned(),
            #[cfg(host)]
            data_dir,
        })
    }

    fn check(&self) -> &AppConfig {
        self.get()
            .expect("App config should be initialized first. Did you forget to add `init!();` macro in the beginning of the main function?")
    }
}
