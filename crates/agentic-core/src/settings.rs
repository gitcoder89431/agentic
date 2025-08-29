use crate::models::ModelValidator;
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
    LocalEndpointUnreachable,
    LocalModelNotFound,
    CloudEndpointUnreachable,
    CloudModelNotFound,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
            endpoint: "localhost:11434".to_string(),
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

    pub async fn validate_endpoints(&self) -> Result<(), ValidationError> {
        let validator = ModelValidator::new();

        // First do basic validation
        self.is_valid()?;

        // Then validate actual endpoints and generation capabilities
        validator
            .validate_local_endpoint(&self.endpoint, &self.local_model)
            .await
            .map_err(|_| ValidationError::LocalEndpointUnreachable)?;

        validator
            .test_local_generation(&self.endpoint, &self.local_model)
            .await
            .map_err(|_| ValidationError::LocalEndpointUnreachable)?;

        validator
            .validate_cloud_endpoint(&self.api_key, &self.cloud_model)
            .await
            .map_err(|_| ValidationError::CloudEndpointUnreachable)?;

        validator
            .test_cloud_generation(&self.api_key, &self.cloud_model)
            .await
            .map_err(|_| ValidationError::CloudEndpointUnreachable)?;

        Ok(())
    }

    pub async fn validate_local_only(&self) -> Result<(), ValidationError> {
        if self.local_model == LOCAL_MODEL_PLACEHOLDER {
            return Err(ValidationError::LocalModel);
        }

        let validator = ModelValidator::new();

        // First validate the model exists
        validator
            .validate_local_endpoint(&self.endpoint, &self.local_model)
            .await
            .map_err(|_| ValidationError::LocalEndpointUnreachable)?;

        // Then test actual generation capability
        validator
            .test_local_generation(&self.endpoint, &self.local_model)
            .await
            .map_err(|_| ValidationError::LocalEndpointUnreachable)?;

        Ok(())
    }

    pub async fn validate_cloud_only(&self) -> Result<(), ValidationError> {
        if self.cloud_model == CLOUD_MODEL_PLACEHOLDER {
            return Err(ValidationError::CloudModel);
        }
        if self.api_key == API_KEY_PLACEHOLDER {
            return Err(ValidationError::ApiKey);
        }

        let validator = ModelValidator::new();

        // First validate the model exists
        validator
            .validate_cloud_endpoint(&self.api_key, &self.cloud_model)
            .await
            .map_err(|_| ValidationError::CloudEndpointUnreachable)?;

        // Then test actual generation capability
        validator
            .test_cloud_generation(&self.api_key, &self.cloud_model)
            .await
            .map_err(|_| ValidationError::CloudEndpointUnreachable)?;

        Ok(())
    }
}
