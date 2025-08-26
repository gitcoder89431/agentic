use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

pub async fn call_cloud_model(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Optimize prompt for concise responses
    let optimized_prompt = format!(
        "Please provide a concise, well-structured response to this inquiry. Keep it informative but focused:\n\n{}",
        prompt
    );

    let request_body = OpenRouterRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: optimized_prompt,
        }],
        max_tokens: 1024, // Reduced from 2048 for more concise responses
    };

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "OpenRouter API error {}: {}",
            status,
            error_text
        ));
    }

    let openrouter_response: OpenRouterResponse = response.json().await?;

    if let Some(choice) = openrouter_response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(anyhow::anyhow!("No response choices from OpenRouter API"))
    }
}
