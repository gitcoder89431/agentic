use crate::models::call_local_model;
use serde::Deserialize;

const ORCHESTRATOR_PROMPT: &str = r#"You are Ruixen, an inquisitive AI partner. Your job is to analyze the user's request and deconstruct it into three distinct lines of inquiry.

**Your Persona and Tone:**
- Your tone should be that of a collaborative partner.
- Each proposal should have a context statement followed by a curious question.
- Use phrases like "I wonder..." or "I'm wondering if..." for questions.

**The Query to Explore:**
"{query}"

**Output Format:**
Generate exactly 3 proposals. Each proposal should be 2 sentences: a context statement followed by a curious question. Use a dash to separate them like this pattern:

"Context statement here - I wonder about this question?"

Your response must be valid JSON:
{
  "proposals": [
    "First context statement - I wonder about this?",
    "Second context statement - I'm wondering if that?",
    "Third context statement - I wonder about something else?"
  ]
}
"#;

const REVISE_PROMPT: &str = r#"You are an expert prompt engineer. A user wants to revise a prompt proposal.

Original Proposal: "{proposal}"
User's Revision: "{revision}"

Your task is to integrate the user's revision into the original proposal to create a new, single, improved prompt.
The new prompt should be self-contained and ready to use.

Format your response as a JSON object with a single key "proposal" which is a string.
Example:
{
  "proposal": "This is the new, revised prompt."
}
"#;

#[derive(Deserialize, Debug)]
struct ProposalsResponse {
    proposals: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ReviseResponse {
    proposal: String,
}

pub async fn generate_proposals(
    query: &str,
    endpoint: &str,
    model: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let prompt = ORCHESTRATOR_PROMPT.replace("{query}", query);
    
    // Debug: Write the prompt to a file so we can see what's being sent
    std::fs::write("/tmp/debug_prompt.txt", &prompt).ok();
    
    let response_str = call_local_model(endpoint, model, &prompt).await?;
    
    // Debug: Write the response to a file so we can see what came back
    std::fs::write("/tmp/debug_response.txt", &response_str).ok();

    // Attempt to find the start of the JSON object
    if let Some(json_start) = response_str.find("{") {
        let json_str = &response_str[json_start..];
        match serde_json::from_str::<ProposalsResponse>(json_str) {
            Ok(response) => Ok(response.proposals),
            Err(e) => {
                // Debug: Write the JSON we tried to parse
                std::fs::write("/tmp/debug_json.txt", json_str).ok();
                Err(anyhow::anyhow!("Failed to parse proposals JSON: {} | JSON: {}", e, json_str))
            },
        }
    } else {
        Err(anyhow::anyhow!("No JSON object found in model response: {}", response_str))
    }
}

pub async fn revise_proposal(
    proposal: &str,
    revision: &str,
    endpoint: &str,
    model: &str,
) -> Result<String, anyhow::Error> {
    let prompt = REVISE_PROMPT
        .replace("{proposal}", proposal)
        .replace("{revision}", revision);
    let response_str = call_local_model(endpoint, model, &prompt).await?;

    // Attempt to find the start of the JSON object
    if let Some(json_start) = response_str.find("{") {
        let json_str = &response_str[json_start..];
        match serde_json::from_str::<ReviseResponse>(json_str) {
            Ok(response) => Ok(response.proposal),
            Err(e) => Err(anyhow::anyhow!("Failed to parse revision JSON: {}", e)),
        }
    } else {
        Err(anyhow::anyhow!("No JSON object found in model response"))
    }
}
