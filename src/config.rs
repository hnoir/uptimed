use serde::{Deserialize, Serialize};
use std::fs::{metadata, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use crate::serde_duration;

// Represents a custom HTTP header to be sent on each request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalRequestHeader {
    pub name: String,
    pub value: String,
}

// A struct to hold the configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub targets_path: PathBuf, // Path to the file containing target URLs.

    #[serde(with = "serde_duration")]
    pub request_interval: Duration, // How much time between requests?
    #[serde(with = "serde_duration")]
    pub scan_interval: Duration, // How much time between one complete scan and the next one?

    pub custom_headers: Vec<AdditionalRequestHeader>, // List of custom HTTP headers and values to use in every reqiest.
}

impl Configuration {
    pub fn is_valid(&self) -> Result<(), String> {
        let path = self.targets_path.as_path();

        if !path.exists() {
            return Err(format!(
                "The specified file does not exist: {}",
                path.display()
            ));
        }

        if !path.is_file() {
            return Err(format!("Not a file: {}", path.display()));
        }

        let fmeta = match metadata(path) {
            Ok(m) => m,
            Err(e) => {
                return Err(
                    format!("Failed to retrieve metadata of configuration file: {}", e).into(),
                )
            }
        };

        if fmeta.permissions().mode() & 0o444 == 0 {
            return Err(format!("File is not readable: {}", path.display()));
        }

        if self.scan_interval <= self.request_interval {
            return Err("Scan interval must be greater than request interval".to_string());
        }

        Ok(())
    }
}

// Load the configuration into the Configuration memory structure.
pub fn load_configuration(config_path: &str) -> Result<Configuration, Box<dyn std::error::Error>> {
    let mut file = match File::open(config_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Unable to open configuration file {}: {}", config_path, e).into())
        }
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Unable to read configuration file {}: {}", config_path, e).into())
        }
    };

    let config: Configuration = match serde_yaml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!(
                "Unable to deserialize configuration file {}: {}",
                config_path, e
            )
            .into())
        }
    };

    Ok(config)
}
