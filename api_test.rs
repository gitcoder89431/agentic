// Test our struct definitions against the real OpenRouter API
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pricing: ModelPricing,
    pub context_length: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_provider: Option<ProviderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub context_length: Option<u32>,
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_moderated: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterResponse {
    pub data: Vec<OpenRouterModel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await?;
    
    let text = response.text().await?;
    
    match serde_json::from_str::<OpenRouterResponse>(&text) {
        Ok(data) => {
            println!("✅ SUCCESS: Parsed {} models from OpenRouter API", data.data.len());
            println!("✅ Models range from '{}' to '{}'", 
                data.data.first().map(|m| m.id.as_str()).unwrap_or("none"),
                data.data.last().map(|m| m.id.as_str()).unwrap_or("none"));
        },
        Err(e) => {
            println!("❌ FAILED: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
