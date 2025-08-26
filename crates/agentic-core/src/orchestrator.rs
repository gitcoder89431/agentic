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

    let response_str = call_local_model(endpoint, model, &prompt).await?;

    // Debug: Write the response to a file so we can see what came back
    std::fs::write("/tmp/debug_response.txt", &response_str).ok();

    // Attempt to find the start of the JSON object
    if let Some(json_start) = response_str.find("{") {
        let json_str = &response_str[json_start..];
        match serde_json::from_str::<ProposalsResponse>(json_str) {
            Ok(response) => {
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
                Ok(proposals)
            }
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
