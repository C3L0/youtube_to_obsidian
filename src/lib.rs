pub mod ingestion;
pub mod synthesis;
pub mod storage;

pub use ingestion::{fetch_transcript, IngestionError, Transcript};
pub use synthesis::{synthesize_transcript, QaItem, SynthesisError, SynthesisReport};
pub use storage::ObsidianWriter;