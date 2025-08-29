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

    // Debug: Write the synthesis prompt to see what we're sending
    std::fs::write("/tmp/debug_synthesis_prompt.txt", &synthesizer_prompt).ok();

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

    let response_text = response.text().await?;

    // Debug: Write the raw cloud response to see what we got back
    std::fs::write("/tmp/debug_cloud_response.txt", &response_text).ok();

    let openrouter_response: OpenRouterResponse = match serde_json::from_str(&response_text) {
        Ok(res) => res,
        Err(e) => {
            let debug_info = format!(
                "Cloud API Response Parse Error: {}\nRaw Response: {}",
                e, response_text
            );
            std::fs::write("/tmp/debug_cloud_api_error.txt", &debug_info).ok();
            return Err(CloudError::ParseError);
        }
    };

    let message_content = openrouter_response
        .choices
        .first()
        .map(|choice| &choice.message.content)
        .ok_or(CloudError::ParseError)?;

    // Try multiple parsing strategies for cloud model response
    parse_atomic_note_with_fallbacks(message_content)
}

fn parse_atomic_note_with_fallbacks(message_content: &str) -> Result<AtomicNote, CloudError> {
    // Strategy 1: Extract from markdown code blocks
    let clean_content = extract_json_from_cloud_markdown(message_content);

    // Debug: Write the cleaned content we're trying to parse
    std::fs::write("/tmp/debug_synthesis_json.txt", clean_content).ok();

    // Strategy 2: Try direct JSON parsing
    if let Ok(note) = serde_json::from_str::<AtomicNote>(clean_content) {
        return Ok(note);
    }

    // Strategy 3: Try to find just the JSON object
    if let Some(json_start) = clean_content.find("{") {
        let json_str = &clean_content[json_start..];
        if let Some(json_end) = json_str.rfind("}") {
            let json_only = &json_str[..=json_end];
            if let Ok(note) = serde_json::from_str::<AtomicNote>(json_only) {
                return Ok(note);
            }
        }
    }

    // Strategy 4: Try to manually extract header_tags and body_text
    if let Some(note) = try_extract_atomic_note_fields(message_content) {
        return Ok(note);
    }

    // All strategies failed - write comprehensive debug info
    let debug_info = format!(
        "All cloud synthesis parsing strategies failed\nRaw Message: {}\nCleaned Content: {}",
        message_content, clean_content
    );
    std::fs::write("/tmp/debug_synthesis_parse_failure.txt", &debug_info).ok();

    Err(CloudError::ParseError)
}

fn extract_json_from_cloud_markdown(content: &str) -> &str {
    // Try different markdown formats
    if content.contains("```json") {
        if let Some(json_start) = content.find("```json") {
            let after_start = &content[json_start + 7..];
            if let Some(json_end) = after_start.find("```") {
                return after_start[..json_end].trim();
            }
        }
    }

    // Try just ```
    if content.contains("```") {
        if let Some(first_tick) = content.find("```") {
            let after_first = &content[first_tick + 3..];
            if let Some(second_tick) = after_first.find("```") {
                let content = after_first[..second_tick].trim();
                // Skip the language identifier line if present
                if let Some(newline) = content.find('\n') {
                    let potential_json = content[newline..].trim();
                    if potential_json.starts_with('{') {
                        return potential_json;
                    }
                }
                return content;
            }
        }
    }

    content
}

fn try_extract_atomic_note_fields(content: &str) -> Option<AtomicNote> {
    let mut header_tags = Vec::new();
    let mut body_text = String::new();

    // Look for header_tags patterns
    if let Some(tags_start) = content.find("\"header_tags\"") {
        let after_tags = &content[tags_start..];
        if let Some(array_start) = after_tags.find('[') {
            if let Some(array_end) = after_tags.find(']') {
                let array_content = &after_tags[array_start + 1..array_end];
                // Simple parsing of comma-separated quoted strings
                for tag in array_content.split(',') {
                    let cleaned_tag = tag.trim().trim_matches('"').trim();
                    if !cleaned_tag.is_empty() {
                        header_tags.push(cleaned_tag.to_string());
                    }
                }
            }
        }
    }

    // Look for body_text patterns
    if let Some(body_start) = content.find("\"body_text\"") {
        let after_body = &content[body_start..];
        if let Some(quote_start) = after_body.find('"') {
            let after_quote = &after_body[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                body_text = after_quote[..quote_end].to_string();
            }
        }
    }

    // Only return if we found both fields with reasonable content
    if !header_tags.is_empty() && !body_text.is_empty() && body_text.len() > 10 {
        Some(AtomicNote {
            header_tags,
            body_text,
        })
    } else {
        None
    }
}
