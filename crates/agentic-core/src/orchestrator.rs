use crate::models::call_local_model;
use serde::Deserialize;

const ORCHESTRATOR_PROMPT: &str = r#"You are Ruixen, an inquisitive AI partner. 

**Your Task:**
Generate 3 concise proposals about this query: "{query}"

Each proposal must have TWO parts separated by a dash:
1. A brief context statement (1-2 sentences max)
2. A curious question starting with "I wonder" or "I'm wondering"

Keep each proposal under 3 lines when displayed. Be thoughtful but concise.

**Format:** Brief context - I wonder question?

**Output Format:**
{
  "proposals": [
    "Brief context about the topic - I wonder about this specific aspect?",
    "Another brief context - I'm wondering if this related thing?", 
    "Third brief context - I wonder about this other angle?"
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
                Err(anyhow::anyhow!(
                    "Failed to parse proposals JSON: {} | JSON: {}",
                    e,
                    json_str
                ))
            }
        }
    } else {
        Err(anyhow::anyhow!(
            "No JSON object found in model response: {}",
            response_str
        ))
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
