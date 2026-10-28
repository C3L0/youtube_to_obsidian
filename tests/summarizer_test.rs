use async_trait::async_trait;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use yt2obsidian::{OllamaSummarizer, QaItem, Summarizer, SynthesisError, SynthesisReport};

/// Test 1 : test via trait contract using mock
struct MockSummarizer {
    pub should_fail: bool,
}

#[async_trait]
impl Summarizer for MockSummarizer {
    async fn summarize(
        &self,
        _transcript_text: &str,
        _prompt_template: &str,
    ) -> Result<SynthesisReport, SynthesisError> {
        if self.should_fail {
            return Err(SynthesisError::ModelError("Simulated failure".to_string()));
        }

        Ok(SynthesisReport {
            main_topic: "Distributed Inference".to_string(),
            items: vec![QaItem {
                question: "What is Tensor Parallelism?".to_string(),
                answer: "Splitting layers cross GPUs.".to_string(),
                key_concepts: vec!["Tensor-Prallelism".to_string(), "GPU".to_string()],
            }],
        })
    }
}

#[tokio::test]
async fn test_mock_summarizer_contract() {
    let mock = MockSummarizer { should_fail: false };
    let result = mock.summarize("test transcript", "portmpt template").await;

    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.main_topic, "Distributed Inference");
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].key_concepts[0], "Tensor-Prallelism");
}

/// Test 2 : testing OllamaSummarizer network & parsong via Wiremock
#[tokio::test]
async fn test_ollama_summarizer_wiremock() {
    let mock_server = MockServer::start().await;

    let mock_ollama_payload = serde_json::json!({
        "response": "{\"main_topic\": \"Rust Testing\", \"items\": [{\"question\": \"Why use wiremock?\", \"answer\": \"To simulate network calls deterministically.\", \"key_concepts\": [\"Testing\", \"Rust\"]}]}",
        "done": true
    });

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_ollama_payload))
        .mount(&mock_server)
        .await;

    let summarizer = OllamaSummarizer::new("qwen2.5:7b", mock_server.uri());

    let result = summarizer
        .summarize("sample transcript", "{transcript}")
        .await;

    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.main_topic, "Rust Testing");
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].question, "Why use wiremock?");
}
