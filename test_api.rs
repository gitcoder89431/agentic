use serde::{Deserialize, Serialize};

/// OpenRouter model information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hugging_face_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub context_length: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<serde_json::Value>,  // Complex nested object
    pub pricing: ModelPricing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_provider: Option<ProviderInfo>,
}

/// Model pricing information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_reasoning: Option<String>,
}

/// Provider information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    pub max_completion_tokens: Option<u32>,  // Can be null!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_moderated: Option<bool>,
}

/// OpenRouter API response
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterResponse {
    pub data: Vec<OpenRouterModel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing OpenRouter API with updated structs...");
    
    let client = reqwest::Client::new();
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Accept", "application/json")
        .send()
        .await?;

    let response_text = response.text().await?;
    println!("📊 Raw API response length: {} chars", response_text.len());
    
    // Try to parse with our updated structs
    match serde_json::from_str::<OpenRouterResponse>(&response_text) {
        Ok(parsed) => {
            println!("✅ JSON parsed successfully!");
            println!("📈 Total models from API: {}", parsed.data.len());
            
            // Count free vs paid models
            let free_count = parsed.data.iter().filter(|m| m.id.ends_with(":free")).count();
            let paid_count = parsed.data.len() - free_count;
            
            println!("🆓 Free models: {}", free_count);
            println!("💰 Paid models: {}", paid_count);
            
            // Show first few model IDs
            println!("\n🔍 First 10 model IDs:");
            for (i, model) in parsed.data.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, model.id);
            }
            
            println!("\n🎯 All models would be available in the UI!");
        }
        Err(e) => {
            println!("❌ JSON parsing failed: {}", e);
            println!("🔍 First 1000 chars: {}", &response_text[..1000.min(response_text.len())]);
        }
    }
    
    Ok(())
}
