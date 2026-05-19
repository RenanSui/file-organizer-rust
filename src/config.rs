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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directories {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
    pub downloaded_images: String,
    pub downloaded_audios: String,
    pub downloaded_videos: String,
    pub downloaded_documents: String,
    pub downloaded_applications: String,
    pub downloaded_archives: String,
}

impl Directories {
    fn new() -> Self {
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("Downloads"));

        Self {
            source_dir: download_dir.clone(),
            destination_dir: download_dir,
            downloaded_images: String::from("Downloaded Images"),
            downloaded_audios: String::from("Downloaded Audios"),
            downloaded_videos: String::from("Downloaded Videos"),
            downloaded_documents: String::from("Downloaded Documents"),
            downloaded_applications: String::from("Downloaded Applications"),
            downloaded_archives: String::from("Downloaded Archives"),
        }
    }
}

impl Default for Directories {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Extensions {
    pub image: Vec<String>,
    pub audio: Vec<String>,
    pub video: Vec<String>,
    pub document: Vec<String>,
    pub application: Vec<String>,
    pub archive: Vec<String>,
}

impl Extensions {
    fn new() -> Self {
        Self {
            image: vec![
                String::from("png"),
                String::from("jpg"),
                String::from("jpeg"),
                String::from("gif"),
                String::from("jfif"),
                String::from("webp"),
            ],
            audio: vec![String::from("mp3")],
            video: vec![
                String::from("mp4"),
                String::from("mkv"),
                String::from("mov"),
            ],
            document: vec![
                String::from("pdf"),
                String::from("txt"),
                String::from("json"),
                String::from("ini"),
            ],
            application: vec![String::from("exe"), String::from("msi")],
            archive: vec![
                String::from("zip"),
                String::from("rar"),
                String::from("7z"),
                String::from("iso"),
            ],
        }
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub directories: Directories,
    pub extensions: Extensions,
    pub extension_to_directory: HashMap<String, String>,
}

impl Config {
    fn new() -> Self {
        let mut extension_to_directory: HashMap<String, String> = HashMap::new();
        let directories = Directories::new();
        let extensions = Extensions::new();

        for ext in &extensions.image {
            extension_to_directory.insert(ext.clone(), directories.downloaded_images.clone());
        }
        for ext in &extensions.audio {
            extension_to_directory.insert(ext.clone(), directories.downloaded_audios.clone());
        }
        for ext in &extensions.video {
            extension_to_directory.insert(ext.clone(), directories.downloaded_videos.clone());
        }
        for ext in &extensions.document {
            extension_to_directory.insert(ext.clone(), directories.downloaded_documents.clone());
        }
        for ext in &extensions.application {
            extension_to_directory.insert(ext.clone(), directories.downloaded_applications.clone());
        }
        for ext in &extensions.archive {
            extension_to_directory.insert(ext.clone(), directories.downloaded_archives.clone());
        }

        Self {
            directories,
            extensions,
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
