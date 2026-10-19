use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::synthesis::{SynthesisError, SynthesisReport};

#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(
        &self,
        transcript_text: &str,
        prompt_template: &str,
    ) -> Result<SynthesisReport, SynthesisError>;
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

pub struct OllamaSummarizer {
    pub model: String,
    pub base_url: String,
    client: reqwest::Client,
}

impl OllamaSummarizer {
    pub fn new(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Summarizer for OllamaSummarizer {
    async fn summarize(
        &self,
        transcript_text: &str,
        prompt_template: &str,
    ) -> Result<SynthesisReport, SynthesisError> {
        let prompt = prompt_template.replace("{transcript}", transcript_text);

        let request_payload = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
            format: "json",
            options: OllamaOptions {
                temperature: 0.1,
                num_ctx: 8192,
            },
        };

        let endpoint = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&endpoint)
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
}