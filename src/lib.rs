pub mod checkpoint;
pub mod ingestion;
pub mod storage;
pub mod synthesis;
pub mod traits;

pub use checkpoint::CheckpointManager;
pub use ingestion::{IngestionError, Transcript, fetch_channel_shorts, fetch_transcript};
pub use storage::ObsidianWriter;
pub use synthesis::{QaItem, SynthesisError, SynthesisReport, synthesize_transcript};
pub use traits::{OllamaSummarizer, Summarizer};
