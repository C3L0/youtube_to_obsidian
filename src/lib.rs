pub mod ingestion;
pub mod synthesis;

pub use ingestion::{fetch_transcript, IngestionError, Transcript};
pub use synthesis::{synthesize_transcript, QaItem, SynthesisError, SynthesisReport};