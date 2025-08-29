use crate::models::call_local_model;
use serde::Deserialize;

const ORCHESTRATOR_PROMPT: &str = r#"You are Ruixen, an inquisitive AI partner. 

**CRITICAL INSTRUCTION:**
You MUST generate EXACTLY 3 proposals about this query: "{query}"

**MANDATORY FORMAT FOR EACH PROPOSAL:**
[Context statement] - I wonder [question]?

**RULES - NO EXCEPTIONS:**
1. EVERY proposal MUST have a brief context (1-2 sentences) followed by " - I wonder" 
2. EVERY proposal MUST end with a question starting with "I wonder" or "I'm wondering"
3. NO proposals should be just statements or just questions
4. ALWAYS use the exact format: "Context - I wonder/I'm wondering [question]?"

**EXAMPLE OF CORRECT FORMAT:**
"Philosophy has debated this for centuries - I wonder what new perspectives we might discover?"

**Your EXACT output must be valid JSON:**
{
  "proposals": [
    "Brief context statement - I wonder about this specific aspect?",
    "Another context statement - I'm wondering if this could be true?", 
    "Third context statement - I wonder about this different angle?"
  ]
}
"#;

#[derive(Deserialize, Debug)]
struct ProposalObject {
    context: String,
    question: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ProposalItem {
    StringFormat(String),
    ObjectFormat(ProposalObject),
}

#[derive(Deserialize, Debug)]
struct ProposalsResponse {
    proposals: Vec<ProposalItem>,
}

pub async fn generate_proposals(
    query: &str,
    endpoint: &str,
    model: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let prompt = ORCHESTRATOR_PROMPT.replace("{query}", query);

    // Debug: Write the prompt to a file so we can see what's being sent
    std::fs::write("/tmp/debug_prompt.txt", &prompt).ok();

    let response_str = match call_local_model(endpoint, model, &prompt).await {
        Ok(response) => response,
        Err(e) => {
            // Enhanced error with more context
            let error_msg = format!(
                "Local model API call failed for endpoint '{}' with model '{}': {}",
                endpoint, model, e
            );
            std::fs::write("/tmp/debug_error.txt", &error_msg).ok();
            return Err(anyhow::anyhow!(error_msg));
        }
    };

    // Debug: Write the response to a file so we can see what came back
    std::fs::write("/tmp/debug_response.txt", &response_str).ok();

    // Try multiple JSON extraction strategies
    parse_proposals_with_fallbacks(&response_str, endpoint, model)
}

fn parse_proposals_with_fallbacks(
    response_str: &str,
    endpoint: &str,
    model: &str,
) -> Result<Vec<String>, anyhow::Error> {
    // Strategy 1: Try to extract JSON from markdown code blocks
    let clean_response = extract_json_from_markdown(response_str);

    // Strategy 2: Try to find and parse the JSON object
    if let Some(json_start) = clean_response.find("{") {
        let json_str = &clean_response[json_start..];
        if let Ok(response) = serde_json::from_str::<ProposalsResponse>(json_str) {
            let proposals = response
                .proposals
                .into_iter()
                .map(|item| match item {
                    ProposalItem::StringFormat(s) => s,
                    ProposalItem::ObjectFormat(obj) => {
                        format!("{} - {}", obj.context, obj.question)
                    }
                })
                .collect();
            return Ok(proposals);
        }
    }

    // Strategy 3: Try to parse just the proposals array
    if let Some(proposals) = try_parse_proposals_array(clean_response) {
        return Ok(proposals);
    }

    // Strategy 4: Try to extract proposals from text patterns
    if let Some(proposals) = try_extract_text_proposals(response_str) {
        return Ok(proposals);
    }

    // All strategies failed - write debug info and return error
    let debug_info = format!(
        "All JSON parsing strategies failed\nEndpoint: {}\nModel: {}\nFull Response: {}\nCleaned Response: {}", 
        endpoint, model, response_str, clean_response
    );
    std::fs::write("/tmp/debug_parse_failure.txt", &debug_info).ok();

    Err(anyhow::anyhow!(
        "Local model '{}' at '{}' did not return parseable proposals. Response was: '{}'",
        model,
        endpoint,
        response_str.chars().take(200).collect::<String>()
    ))
}

fn extract_json_from_markdown(response: &str) -> &str {
    // Try different markdown formats
    if response.contains("```json") {
        if let Some(json_start) = response.find("```json") {
            let after_start = &response[json_start + 7..];
            if let Some(json_end) = after_start.find("```") {
                return after_start[..json_end].trim();
            }
        }
    }

    // Try just ```
    if response.contains("```") {
        if let Some(first_tick) = response.find("```") {
            let after_first = &response[first_tick + 3..];
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

    response
}

fn try_parse_proposals_array(response: &str) -> Option<Vec<String>> {
    // Look for a proposals array directly
    if let Some(proposals_start) = response.find("\"proposals\"") {
        let after_proposals = &response[proposals_start..];
        if let Some(array_start) = after_proposals.find('[') {
            if let Some(array_end) = after_proposals.find(']') {
                let array_content = &after_proposals[array_start..=array_end];
                let full_json = format!("{{\"proposals\":{}}}", array_content);
                if let Ok(parsed) = serde_json::from_str::<ProposalsResponse>(&full_json) {
                    return Some(
                        parsed
                            .proposals
                            .into_iter()
                            .map(|item| match item {
                                ProposalItem::StringFormat(s) => s,
                                ProposalItem::ObjectFormat(obj) => {
                                    format!("{} - {}", obj.context, obj.question)
                                }
                            })
                            .collect(),
                    );
                }
            }
        }
    }
    None
}

fn try_extract_text_proposals(response: &str) -> Option<Vec<String>> {
    let mut proposals = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();

        // Look for lines that match our expected proposal format
        if (trimmed.contains(" - I wonder") || trimmed.contains(" - I'm wondering"))
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("#")
        {
            // Clean up common prefixes/suffixes
            let cleaned = trimmed
                .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ')
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_end_matches(',')
                .to_string();

            if !cleaned.is_empty() && cleaned.len() > 20 {
                // Reasonable minimum length
                proposals.push(cleaned);
            }
        }
    }

    if proposals.len() >= 2 {
        Some(proposals)
    } else {
        None
    }
}
