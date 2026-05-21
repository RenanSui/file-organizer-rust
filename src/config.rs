use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Configuration not found")]
    ConfigNotFound,

    #[error("Source is not a directory")]
    SourceIsNotDir,

    #[error("Destination is not a directory")]
    DestinationIsNotDir,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("toml parse error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

trait HashMapExt {
    fn insert_many<S>(&mut self, keys: &[S], value: impl Into<String>)
    where
        S: AsRef<str>;
}

impl HashMapExt for HashMap<String, String> {
    fn insert_many<S>(&mut self, keys: &[S], value: impl Into<String>)
    where
        S: AsRef<str>,
    {
        let value = value.into();
        for key in keys {
            self.insert(key.as_ref().to_owned(), value.clone());
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directories {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
}

impl Directories {
    fn new() -> Self {
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("Downloads"));

        Self {
            source_dir: download_dir.clone(),
            destination_dir: download_dir,
        }
    }
}

impl Default for Directories {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub directories: Directories,
    pub extension_to_directory: HashMap<String, String>,
}

impl Config {
    fn new() -> Self {
        let directories = Directories::new();
        let mut extension_to_directory: HashMap<String, String> = HashMap::new();

        extension_to_directory
            .insert_many(&["png", "jpg", "jpeg", "gif", "webp"], "Downloaded Images");
        extension_to_directory.insert_many(&["mp3"], "Downloaded Audios");
        extension_to_directory.insert_many(&["mp4", "mkv", "mov"], "Downloaded Videos");
        extension_to_directory.insert_many(&["pdf", "txt", "json", "ini"], "Downloaded Documents");
        extension_to_directory.insert_many(&["exe", "msi"], "Downloaded Applications");
        extension_to_directory.insert_many(&["zip", "rar", "7z", "iso"], "Downloaded Archives");

        Self {
            directories,
            extension_to_directory,
        }
    }

    fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::HomeDirNotFound)?
            .join(".config")
            .join("file organizer rust");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn create_default_config() -> Result<(), ConfigError> {
        let config_path = self::Config::get_config_path()?;

        if config_path.exists() {
            return Ok(());
        }

        let config = self::Config::new().to_toml()?;
        fs::write(config_path, config)?;
        Ok(())
    }

    pub fn load_config() -> Result<Self, ConfigError> {
        let config_path = self::Config::get_config_path()?;

        if !config_path.exists() {
            return Err(ConfigError::ConfigNotFound);
        }

        let string_config = fs::read_to_string(&config_path)?;
        let config = toml::from_str::<Config>(&string_config)?;

        config.validate()?;

        Ok(config)
    }

    pub fn get_extension_dir<S: AsRef<str>>(self, file_extension: S) -> Option<String> {
        self.extension_to_directory
            .get(file_extension.as_ref())
            .cloned()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.directories.source_dir.is_dir() {
            return Err(ConfigError::SourceIsNotDir);
        }

        if !self.directories.destination_dir.is_dir() {
            return Err(ConfigError::DestinationIsNotDir);
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
