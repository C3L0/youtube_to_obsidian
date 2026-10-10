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
            Self::NetworkError(e) => write!(f, "Network error communicating with Ollama: {}", e),
            Self::SerializationError(e) => write!(f, "Failed to parse JSON response: {}", e),
            Self::ModelError(msg) => write!(f, "Model returned an error: {}", msg),
        }
    }
}

impl std::error::Error for SynthesisError {}

// Domain output
#[derive(Debug, Serialize, Deserialize)]
pub struct QaItem {
    pub question: String,
    pub answer: String,
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SynthesisReport {
    pub main_topic: String,
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

#[derive(Serialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

pub async fn synthesize_transcript(
    transcript_text: &str,
    model_name: &str,
) -> Result<SynthesisReport, SynthesisError> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        r#"You are an expert technical interviewer and educator. 
        Analyze this veideo transcript and extract all interview questions, clear technical explanations, and core concepts.

        Format your output strictly as a JSON object matching this schema:
        {{
            "main_topic": "General topic of the interview",
            "items": [
                {{
                    "question": "The question asked or topic discussed",
                    "answer": "Clear, concise technical explanation synthesized from the transcript",
                    "key_concept": ["concept1", "concept2"]
                }}
            ]
        }}
        
        Transcript:
        {}
        "#,
            transcript_text
    );

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
            "Ollama returned Http status {}",
            response.status(),
        )));
    }

    let parsed_body: OllamaResponse = response
        .json()
        .await
        .map_err(SynthesisError::NetworkError)?;

    let report: SynthesisReport = serde_json::from_str(&parsed_body.response)
        .map_err(SynthesisError::SerializationError)?;

    Ok(report)
}
