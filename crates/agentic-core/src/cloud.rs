use crate::models::AtomicNote;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloudError {
    #[error("The cloud provider rejected the API key. It might have expired or been disabled.")]
    ApiKey,

    #[error("The cloud model returned a response that could not be understood.")]
    ParseError,

    #[error("The cloud provider returned an unexpected error: {status}: {text}")]
    ApiError { status: u16, text: String },

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}

const SYNTHESIZER_PROMPT: &str = r#"You are an expert-level AI Synthesizer. Your task is to answer the user's prompt by generating a concise, "atomic note" of knowledge.

CRITICAL OUTPUT CONSTRAINTS:

Header (Metadata): You MUST generate a set of 3-5 semantic keywords or tags that capture the absolute essence of the topic. These tags are for a knowledge graph.

Body (Content): The main response MUST be a maximum of four (4) sentences. It must be a dense, self-contained summary of the most critical information.

OUTPUT FORMAT (JSON):
Your final output MUST be a single, valid JSON object with two keys: header_tags and body_text.

{
  "header_tags": ["keyword1", "keyword2", "keyword3"],
  "body_text": "Your concise, 3-4 sentence summary goes here."
}

USER PROMPT:
{prompt}
"#;

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize)]
struct OpenRouterRequest<'a> {
    model: String,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: String,
    content: &'a str,
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
) -> Result<AtomicNote, CloudError> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let synthesizer_prompt = SYNTHESIZER_PROMPT.replace("{prompt}", prompt);

    let request_body = OpenRouterRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: &synthesizer_prompt,
        }],
        max_tokens: 1024,
        response_format: ResponseFormat {
            r#type: "json_object".to_string(),
        },
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
        if status == 401 {
            return Err(CloudError::ApiKey);
        }
        let error_text = response.text().await.unwrap_or_default();
        return Err(CloudError::ApiError {
            status: status.as_u16(),
            text: error_text,
        });
    }

    let openrouter_response: OpenRouterResponse = match response.json().await {
        Ok(res) => res,
        Err(_) => return Err(CloudError::ParseError),
    };

    let message_content = openrouter_response
        .choices
        .first()
        .map(|choice| &choice.message.content)
        .ok_or(CloudError::ParseError)?;

    // Extract JSON from markdown code blocks if present
    let clean_content = if message_content.contains("```json") {
        // Extract content between ```json and ```
        if let Some(json_start) = message_content.find("```json") {
            let after_start = &message_content[json_start + 7..]; // Skip "```json"
            if let Some(json_end) = after_start.find("```") {
                after_start[..json_end].trim()
            } else {
                message_content
            }
        } else {
            message_content
        }
    } else {
        message_content
    };

    let atomic_note: AtomicNote =
        serde_json::from_str(clean_content).map_err(|_| CloudError::ParseError)?;

    Ok(atomic_note)
}
