use crate::theme::ThemeVariant;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::fs;

const LOCAL_MODEL_PLACEHOLDER: &str = "[SELECT]";
const CLOUD_MODEL_PLACEHOLDER: &str = "[SELECT]";
const API_KEY_PLACEHOLDER: &str = "sk-or-v1-982...b52";

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    LocalModel,
    CloudModel,
    ApiKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub theme: ThemeVariant,
    pub endpoint: String,
    pub local_model: String,
    pub api_key: String,
    pub cloud_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::default(),
            endpoint: "LOCALHOST:2032".to_string(),
            local_model: LOCAL_MODEL_PLACEHOLDER.to_string(),
            api_key: API_KEY_PLACEHOLDER.to_string(),
            cloud_model: CLOUD_MODEL_PLACEHOLDER.to_string(),
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // This will create a default config if it doesn't exist
        let config_path = "config.toml";
        let figment = Figment::new().merge(Toml::file(config_path));

        match figment.extract() {
            Ok(settings) => Ok(settings),
            Err(_) => {
                let default_settings = Settings::default();
                default_settings.save().unwrap_or_default();
                Ok(default_settings)
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let toml_string =
            toml::to_string_pretty(self).expect("Failed to serialize settings to TOML");
        fs::write("config.toml", toml_string)
    }

    pub fn is_valid(&self) -> Result<(), ValidationError> {
        if self.local_model == LOCAL_MODEL_PLACEHOLDER {
            return Err(ValidationError::LocalModel);
        }
        if self.cloud_model == CLOUD_MODEL_PLACEHOLDER {
            return Err(ValidationError::CloudModel);
        }
        if self.api_key == API_KEY_PLACEHOLDER {
            return Err(ValidationError::ApiKey);
        }
        Ok(())
    }
}
