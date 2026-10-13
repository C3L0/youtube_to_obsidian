pub mod ingestion;
pub mod synthesis;
pub mod storage;
pub mod checkpoint;

pub use ingestion::{fetch_transcript, fetch_channel_shorts, IngestionError, Transcript};
pub use synthesis::{synthesize_transcript, QaItem, SynthesisError, SynthesisReport};
pub use storage::ObsidianWriter;
pub use checkpoint::CheckpointManager;