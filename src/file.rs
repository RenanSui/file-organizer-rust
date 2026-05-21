use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::config::Config;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Path is not a file.")]
    PathIsNotFile,

    #[error("Unsupported file extension.")]
    UnsupportedExtension,

    #[error("Empty file.")]
    EmptyFile,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    destination: PathBuf,
    file_name: String,
    file_extension: String,
}

impl File {
    fn new() -> Self {
        File {
            path: PathBuf::from(""),
            destination: PathBuf::from(""),
            file_name: String::from(""),
            file_extension: String::from(""),
        }
    }

    pub fn with_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.path = path.as_ref().to_owned();
        self
    }

    pub fn extract_metadata(&mut self) -> Result<&mut Self, FileError> {
        // get file name or return a FileError
        let file_name = self
            .path
            .file_name()
            .ok_or(FileError::PathIsNotFile)?
            .to_string_lossy()
            .to_string();

        // get file extension or return a FileError
        let file_extension = self
            .path
            .extension()
            .ok_or(FileError::PathIsNotFile)?
            .to_string_lossy()
            .to_string()
            .to_lowercase();

        // Check if file is empty
        if let Ok(metadata) = self.path.metadata() {
            if metadata.len() == 0 {
                println!("Empty file: {:#?}", self.path);
                return Err(FileError::EmptyFile);
            }
        }

        self.file_name = file_name;
        self.file_extension = file_extension;

        println!("extract_metadata: {self:#?}");

        Ok(self)
    }

    pub fn ensure_destination(&mut self, config: &Config) -> Result<&mut Self, FileError> {
        // get extension sub directory
        let extension_subdir = config
            .clone()
            .get_extension_dir(self.file_extension.clone())
            .ok_or(FileError::UnsupportedExtension)?;

        // merges destination directory with extension directory
        self.destination = config.directories.destination_dir.join(extension_subdir);

        // recursively creates all directories for the extension directory
        fs::create_dir_all(&self.destination).ok();

        println!("ensure_destination: {self:#?}");

        // Ok(self)
        Ok(self)
    }

    // check if file is a duplicate and returns a new file name without conflicting
    pub fn ensure_unique_filename(&mut self) -> &mut Self {
        let base_name = Path::new(&self.file_name)
            .file_prefix()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.file_name);

        let mut counter = 1;

        loop {
            let new_file_name = if counter == 1 {
                format!("{}.{}", base_name, self.file_extension)
            } else {
                format!("{} ({}).{}", base_name, counter, self.file_extension)
            };

            if !self.destination.join(&new_file_name).exists() {
                self.file_name = new_file_name;
                println!("ensure_unique_filename: {self:#?}");
                return self;
            }

            counter += 1
        }
    }

    // Move file to a new destination, in case of error tries a fallback
    pub fn move_file(&mut self) -> Result<(), FileError> {
        // merges the new file name with the directory
        self.destination = self.destination.join(&self.file_name);
        println!("move_file: {self:#?}");

        // Move file
        fs::rename(&self.path, &self.destination).or_else(|_| {
            // Fallback copy + delete original
            fs::copy(&self.path, &self.destination)?;
            fs::remove_file(&self.path)?;
            Ok(())
        })
    }
}

impl Default for File {
    fn default() -> Self {
        Self::new()
    }
}
