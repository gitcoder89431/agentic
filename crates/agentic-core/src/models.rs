use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

// Global provider cache to ensure consistency across different ModelValidator instances
static PROVIDER_CACHE: OnceLock<Arc<Mutex<HashMap<String, LocalProvider>>>> = OnceLock::new();

fn get_global_provider_cache() -> &'static Arc<Mutex<HashMap<String, LocalProvider>>> {
    PROVIDER_CACHE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicNote {
    pub header_tags: Vec<String>,
    pub body_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: String,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub name: String,
    pub id: String,
    pub provider: LocalProvider,
    pub size: String,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocalProvider {
    Ollama,
    LMStudio,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pricing: ModelPricing,
    pub context_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaListResponse {
    models: Vec<OllamaModelRaw>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaModelRaw {
    name: String,
    size: i64,
    modified_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterListResponse {
    data: Vec<OpenRouterModelRaw>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterModelRaw {
    id: String,
    name: String,
    description: Option<String>,
    pricing: ModelPricingRaw,
    context_length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelPricingRaw {
    prompt: String,
    completion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIListResponse {
    data: Vec<OpenAIModelRaw>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIModelRaw {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    created: Option<u64>,
}

pub struct ModelValidator {
    client: Client,
}

impl ModelValidator {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub async fn detect_provider_type(&self, endpoint: &str) -> LocalProvider {
        // Check global cache first
        let cache = get_global_provider_cache();
        if let Ok(cache_lock) = cache.lock() {
            if let Some(cached_provider) = cache_lock.get(endpoint) {
                return cached_provider.clone();
            }
        }

        // Perform actual detection
        let detected_provider = self.perform_provider_detection(endpoint).await;

        // Cache the result globally
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(endpoint.to_string(), detected_provider.clone());
        }

        detected_provider
    }

    async fn perform_provider_detection(&self, endpoint: &str) -> LocalProvider {
        // Try OpenAI/LM Studio API first for port 1234
        if endpoint.to_lowercase().contains("1234")
            && self.test_openai_endpoint(endpoint).await.is_ok()
        {
            return LocalProvider::LMStudio;
        }

        // Try Ollama API
        if self.test_ollama_endpoint(endpoint).await.is_ok() {
            return LocalProvider::Ollama;
        }

        // Try generic OpenAI API
        if self.test_openai_endpoint(endpoint).await.is_ok() {
            return LocalProvider::OpenAI;
        }

        // Default to Ollama if all detection fails
        LocalProvider::Ollama
    }

    async fn test_ollama_endpoint(&self, endpoint: &str) -> Result<()> {
        let url = if endpoint.starts_with("http") {
            format!("{}/api/tags", endpoint)
        } else {
            format!("http://{}/api/tags", endpoint)
        };

        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Ollama endpoint not accessible"))
        }
    }

    async fn test_openai_endpoint(&self, endpoint: &str) -> Result<()> {
        let normalized_endpoint = endpoint.to_lowercase();
        let url = if normalized_endpoint.starts_with("http") {
            format!("{}/v1/models", normalized_endpoint)
        } else {
            format!("http://{}/v1/models", normalized_endpoint)
        };

        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("OpenAI endpoint not accessible"))
        }
    }

    pub async fn fetch_local_models(&self, endpoint: &str) -> Result<Vec<LocalModel>> {
        let provider = self.detect_provider_type(endpoint).await;

        match provider {
            LocalProvider::Ollama => {
                let ollama_models = self.fetch_ollama_models(endpoint).await?;
                let local_models = ollama_models
                    .into_iter()
                    .map(|model| LocalModel {
                        name: model.name.clone(),
                        id: model.name,
                        provider: LocalProvider::Ollama,
                        size: model.size,
                        modified: model.modified,
                    })
                    .collect();
                Ok(local_models)
            }
            LocalProvider::LMStudio | LocalProvider::OpenAI => {
                self.fetch_openai_models(endpoint).await
            }
        }
    }

    pub async fn fetch_ollama_models(&self, endpoint: &str) -> Result<Vec<OllamaModel>> {
        let url = if endpoint.starts_with("http") {
            format!("{}/api/tags", endpoint)
        } else {
            format!("http://{}/api/tags", endpoint)
        };

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama endpoint not accessible"));
        }

        let ollama_response: OllamaListResponse = response.json().await?;

        let models = ollama_response
            .models
            .into_iter()
            .map(|raw| OllamaModel {
                name: raw.name,
                size: format_size(raw.size),
                modified: format_relative_time(&raw.modified_at),
            })
            .collect();

        Ok(models)
    }

    pub async fn fetch_openrouter_models(&self, api_key: &str) -> Result<Vec<OpenRouterModel>> {
        let url = "https://openrouter.ai/api/v1/models";

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "OpenRouter API not accessible or invalid API key"
            ));
        }

        let openrouter_response: OpenRouterListResponse = response.json().await?;

        let mut models: Vec<OpenRouterModel> = openrouter_response
            .data
            .into_iter()
            .map(|raw| OpenRouterModel {
                id: raw.id,
                name: raw.name,
                description: raw
                    .description
                    .unwrap_or_else(|| "No description available".to_string()),
                pricing: ModelPricing {
                    prompt: raw.pricing.prompt,
                    completion: raw.pricing.completion,
                },
                context_length: raw.context_length,
            })
            .collect();

        // Sort models: free models first, then paid models
        models.sort_by(|a, b| {
            let a_is_free = a.pricing.prompt == "0" && a.pricing.completion == "0";
            let b_is_free = b.pricing.prompt == "0" && b.pricing.completion == "0";

            match (a_is_free, b_is_free) {
                (true, false) => std::cmp::Ordering::Less, // Free comes first
                (false, true) => std::cmp::Ordering::Greater, // Paid comes after
                _ => a.name.cmp(&b.name),                  // Same type, sort by name
            }
        });

        Ok(models)
    }

    pub async fn fetch_openai_models(&self, endpoint: &str) -> Result<Vec<LocalModel>> {
        let url = if endpoint.starts_with("http") {
            format!("{}/v1/models", endpoint)
        } else {
            format!("http://{}/v1/models", endpoint)
        };

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenAI/LM Studio endpoint not accessible"));
        }

        let openai_response: OpenAIListResponse = response.json().await?;

        let provider = if endpoint.contains("1234") {
            LocalProvider::LMStudio
        } else {
            LocalProvider::OpenAI
        };

        let models = openai_response
            .data
            .into_iter()
            .map(|raw| LocalModel {
                name: raw.name.unwrap_or_else(|| raw.id.clone()),
                id: raw.id,
                provider: provider.clone(),
                size: "Unknown".to_string(),
                modified: "recently".to_string(),
            })
            .collect();

        Ok(models)
    }

    pub async fn validate_local_endpoint(&self, endpoint: &str, model: &str) -> Result<()> {
        let provider = self.detect_provider_type(endpoint).await;

        match provider {
            LocalProvider::Ollama => {
                let url = if endpoint.starts_with("http") {
                    format!("{}/api/tags", endpoint)
                } else {
                    format!("http://{}/api/tags", endpoint)
                };

                let response = self.client.get(&url).send().await?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Local endpoint not accessible"));
                }

                let models: Value = response.json().await?;
                if let Some(models_array) = models.get("models").and_then(|m| m.as_array()) {
                    let model_exists = models_array.iter().any(|m| {
                        m.get("name")
                            .and_then(|name| name.as_str())
                            .map(|name| name == model)
                            .unwrap_or(false)
                    });

                    if model_exists {
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!(
                            "Model '{}' not found on local endpoint",
                            model
                        ))
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid response format from local endpoint"
                    ))
                }
            }
            LocalProvider::LMStudio | LocalProvider::OpenAI => {
                let url = if endpoint.starts_with("http") {
                    format!("{}/v1/models", endpoint)
                } else {
                    format!("http://{}/v1/models", endpoint)
                };

                let response = self.client.get(&url).send().await?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Local endpoint not accessible"));
                }

                let models: Value = response.json().await?;
                if let Some(models_array) = models.get("data").and_then(|m| m.as_array()) {
                    let model_exists = models_array.iter().any(|m| {
                        m.get("id")
                            .and_then(|id| id.as_str())
                            .map(|id| id == model)
                            .unwrap_or(false)
                    });

                    if model_exists {
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!(
                            "Model '{}' not found on local endpoint",
                            model
                        ))
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid response format from local endpoint"
                    ))
                }
            }
        }
    }

    pub async fn validate_cloud_endpoint(&self, api_key: &str, model: &str) -> Result<()> {
        let url = "https://openrouter.ai/api/v1/models";

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Cloud API key invalid or endpoint not accessible"
            ));
        }

        let models: Value = response.json().await?;

        if let Some(models_array) = models.get("data").and_then(|m| m.as_array()) {
            let model_exists = models_array.iter().any(|m| {
                m.get("id")
                    .and_then(|id| id.as_str())
                    .map(|id| id == model)
                    .unwrap_or(false)
            });

            if model_exists {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Model '{}' not found in OpenRouter", model))
            }
        } else {
            Err(anyhow::anyhow!("Invalid response format from OpenRouter"))
        }
    }

    pub async fn test_local_generation(&self, endpoint: &str, model: &str) -> Result<()> {
        let url = if endpoint.starts_with("http") {
            format!("{}/api/generate", endpoint)
        } else {
            format!("http://{}/api/generate", endpoint)
        };

        let payload = serde_json::json!({
            "model": model,
            "prompt": "Hello",
            "stream": false,
            "options": {
                "num_predict": 1
            }
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to generate response from local model"
            ))
        }
    }

    pub async fn test_cloud_generation(&self, api_key: &str, model: &str) -> Result<()> {
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let payload = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 1
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to generate response from cloud model"
            ))
        }
    }
}

#[derive(Serialize)]
struct LocalGenerationRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct LocalGenerationResponse {
    response: String,
}

pub async fn call_local_model(
    endpoint: &str,
    model: &str,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let validator = ModelValidator::new();
    let provider = validator.detect_provider_type(endpoint).await;

    match provider {
        LocalProvider::Ollama => call_ollama_model(endpoint, model, prompt).await,
        LocalProvider::LMStudio | LocalProvider::OpenAI => {
            call_openai_model(endpoint, model, prompt).await
        }
    }
}

pub async fn call_ollama_model(
    endpoint: &str,
    model: &str,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let client = Client::new();
    let url = if endpoint.starts_with("http") {
        format!("{}/api/generate", endpoint)
    } else {
        format!("http://{}/api/generate", endpoint)
    };

    let payload = LocalGenerationRequest {
        model,
        prompt,
        stream: false,
    };

    let response = client.post(&url).json(&payload).send().await?;

    if response.status().is_success() {
        let gen_response: LocalGenerationResponse = response.json().await?;
        Ok(gen_response.response)
    } else {
        Err(anyhow::anyhow!(
            "Failed to get response from local model. Status: {}",
            response.status()
        ))
    }
}

#[derive(Serialize)]
struct OpenAIGenerationRequest<'a> {
    model: &'a str,
    messages: Vec<serde_json::Value>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenAIGenerationResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIMessage {
    content: String,
}

pub async fn call_openai_model(
    endpoint: &str,
    model: &str,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let client = Client::new();
    let url = if endpoint.starts_with("http") {
        format!("{}/v1/chat/completions", endpoint)
    } else {
        format!("http://{}/v1/chat/completions", endpoint)
    };

    let payload = OpenAIGenerationRequest {
        model,
        messages: vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })],
        max_tokens: 2000,
        temperature: 0.7,
    };

    let response = client.post(&url).json(&payload).send().await?;

    if response.status().is_success() {
        let gen_response: OpenAIGenerationResponse = response.json().await?;
        if let Some(choice) = gen_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI model"))
        }
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(anyhow::anyhow!(
            "Failed to get response from OpenAI model. Status: {}. Error: {}",
            status,
            error_text
        ))
    }
}

impl Default for ModelValidator {
    fn default() -> Self {
        Self::new()
    }
}

fn format_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as i64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn format_relative_time(_iso_time: &str) -> String {
    // For now, just return a simple format
    // Parse ISO time and return relative time
    "recently".to_string()
}
