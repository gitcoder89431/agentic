use crate::models::call_local_model;
use serde::Deserialize;

const ORCHESTRATOR_PROMPT: &str = r#"You are an expert prompt engineer. Your task is to help a user craft the perfect prompt for a powerful AI model.
The user has provided the following query: "{query}"

Analyze the user's query and generate three distinct proposals for a better prompt.
Each proposal should be a self-contained, ready-to-use prompt.
Use the 5W method (What, Who, When, Where, How) to explore different angles of the user's request.
Rank the proposals by your internal confidence, from least confident to most confident.

Format your response as a JSON object with a single key "proposals" which is an array of three strings.
Example:
{
  "proposals": [
    "Proposal 1 (least confident)",
    "Proposal 2 (medium confident)",
    "Proposal 3 (most confident)"
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
    let response_str = call_local_model(endpoint, model, &prompt).await?;

    // Attempt to find the start of the JSON object
    if let Some(json_start) = response_str.find("{") {
        let json_str = &response_str[json_start..];
        match serde_json::from_str::<ProposalsResponse>(json_str) {
            Ok(response) => Ok(response.proposals),
            Err(e) => Err(anyhow::anyhow!("Failed to parse proposals JSON: {}", e)),
        }
    } else {
        Err(anyhow::anyhow!("No JSON object found in model response"))
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
