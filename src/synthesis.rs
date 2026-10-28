use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum SynthesisError {
    NetworkError(reqwest::Error),
    SerializationError(serde_json::Error),
    ModelError(String),
}

impl fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::SerializationError(e) => write!(f, "Failed to parse JSON: {}", e),
            Self::ModelError(msg) => write!(f, "Model returned error: {}", msg),
        }
    }
}

impl std::error::Error for SynthesisError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct QaItem {
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub answer: String,
    #[serde(
        default,
        alias = "keyConcepts",
        alias = "concepts",
        alias = "tags",
        alias = "keywords"
    )]
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SynthesisReport {
    #[serde(default, alias = "topic", alias = "title")]
    pub main_topic: String,
    #[serde(default, alias = "questions", alias = "qa_pairs")]
    pub items: Vec<QaItem>,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    format: &'a str,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_ctx: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    #[serde(default)]
    done: bool,
}

pub async fn synthesize_transcript(
    transcript_text: &str,
    model_name: &str,
    prompt_template: &str,
) -> Result<SynthesisReport, SynthesisError> {
    let client = reqwest::Client::new();

    let prompt = prompt_template.replace("{transcript}", transcript_text);

    let request_payload = OllamaRequest {
        model: model_name,
        prompt,
        stream: false,
        format: "json",
        options: OllamaOptions {
            temperature: 0.1,
            num_ctx: 8192,
        },
    };

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_payload)
        .send()
        .await
        .map_err(SynthesisError::NetworkError)?;

    if !response.status().is_success() {
        return Err(SynthesisError::ModelError(format!(
            "Ollama returned HTTP {}",
            response.status()
        )));
    }

    let parsed_body: OllamaResponse = response
        .json()
        .await
        .map_err(SynthesisError::NetworkError)?;

    let report: SynthesisReport = match serde_json::from_str(&parsed_body.response) {
        Ok(rep) => rep,
        Err(err) => {
            eprintln!("\n[DEBUG] Raw JSON from model:\n{}", parsed_body.response);
            return Err(SynthesisError::SerializationError(err));
        }
    };

    Ok(report)
}
