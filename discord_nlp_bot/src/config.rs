use crate::file::read_file_as_string;
use std::{fmt, io};

pub enum Error {
    FileError(io::Error),
    SerdeError(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::FileError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileError(e) => write!(f, "Database error: {}", e),
            Self::SerdeError(e) => write!(f, "Serde error: {}", e),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub discord_token: String,
    pub sql_database_path: String,
}

pub fn read_configuration_from_file(path: &String) -> Result<Configuration, Error> {
    let json_str = read_file_as_string(path)?;
    let configuration: Configuration = serde_json::from_str(&json_str)?;

    Ok(configuration)
}
