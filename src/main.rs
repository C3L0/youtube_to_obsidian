use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process;
use yt2obsidian::{
    CheckpointManager, ObsidianWriter, OllamaSummarizer, Summarizer, fetch_channel_shorts,
    fetch_transcript,
};

const DEFAULT_PROMPT: &str = r#"You are a technical educator. Extract interview Q&As from this transcript.
For every item, you MUST provide 2 to 4 technical keywords in "key_concepts".

Respond ONLY with valid JSON following this exact structure:
{
  "main_topic": "Topic of the video",
  "items": [
    {
      "question": "The question or concept being explained",
      "answer": "Detailed technical explanation synthesized from the transcript",
      "key_concepts": ["concept-1", "concept-2", "concept-3"]
    }
  ]
}

Transcript:
{transcript}
"#;

#[derive(Parser, Debug)]
#[command(
    name = "yt2obsidian",
    about = "Extracts YouTube transcripts and generates structured Obsidian notes using a local LLM."
)]
struct Cli {
    /// YouTube channel URL or single video URL
    #[arg(required = true)]
    target: String,

    /// Ollama model
    #[arg(short, long, env = "OLLAMA_MODEL", default_value = "qwen2.5:7b")]
    model: String,

    /// Obsidian vault directory
    #[arg(short, long, env = "VAULT_PATH", default_value = "./vault")]
    vault: PathBuf,

    /// Path to a custom prompt
    #[arg(short, long, env = "PROMPT_TEMPLATE_PATH")]
    prompt_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let args = Cli::parse();

    let prompt_template = match &args.prompt_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!(
                    "Could not read prompt file {:?}: {}. Using default prompt.",
                    path, err
                );
                DEFAULT_PROMPT.to_string()
            }
        },
        None => DEFAULT_PROMPT.to_string(),
    };

    let summarizer = OllamaSummarizer::new(&args.model, "http://localhost:11434");

    println!("Configuration Loaded:");
    println!("  Model: {}", args.model);
    println!("  Vault Path: {}", args.vault.display());
    println!(
        "  Prompt Source: {}",
        if args.prompt_path.is_some() {
            "Custom File"
        } else {
            "Default Template"
        }
    );

    // 1. Enumerate targets
    let video_urls: Vec<String> = if args.target.contains("/@")
        || args.target.contains("/channel/")
        || args.target.contains("/c/")
    {
        println!("==> Scanning channel for Shorts: {}", args.target);
        match fetch_channel_shorts(&args.target) {
            Ok(urls) => {
                println!("    Found {} Shorts to process.\n", urls.len());
                urls
            }
            Err(err) => {
                eprintln!("    Failed to list channel shorts: {}", err);
                process::exit(1);
            }
        }
    } else {
        vec![args.target.clone()]
    };

    // 2. Initialize Storage and Checkpoint Ledger
    let writer = ObsidianWriter::new(&args.vault);
    let mut checkpoint = match CheckpointManager::init(&args.vault) {
        Ok(cp) => {
            println!(
                "==> Checkpoint ledger loaded ({} previously completed).",
                cp.total_processed()
            );
            cp
        }
        Err(err) => {
            eprintln!("    Failed to initialize checkpoint file: {}", err);
            process::exit(1);
        }
    };

    let total = video_urls.len();

    // 3. Sequential processing loop with skip-checks
    for (i, url) in video_urls.iter().enumerate() {
        println!("============================================================");
        println!("[{}/{}] Checking: {}", i + 1, total, url);

        // Checkpoint gate
        if checkpoint.is_processed(url) {
            println!("    ⏭ Already processed in ledger. Skipping.");
            continue;
        }

        // Stage 1: Ingestion
        let transcript = match fetch_transcript(url) {
            Ok(t) => {
                println!(
                    "    ✓ Transcript ingested ({} characters).",
                    t.full_text.len()
                );
                t
            }
            Err(err) => {
                eprintln!("    ⚠ Skipping video (ingestion error): {}", err);
                continue;
            }
        };

        // Stage 2: Synthesis via the Summarizer trait
        println!("    Synthesizing with Ollama (model: {})...", args.model);
        let report = match summarizer
            .summarize(&transcript.full_text, &prompt_template)
            .await
        {
            Ok(rep) => {
                println!("    ✓ Synthesis complete ({} Q&A items).", rep.items.len());
                rep
            }
            Err(err) => {
                eprintln!("    ⚠ Synthesis failed for {}: {}", url, err);
                continue;
            }
        };

        // Stage 3: Storage
        match writer.write_report(&report, url) {
            Ok(file_path) => {
                println!("    ✓ Saved note: {}", file_path.display());
                if let Err(e) = checkpoint.mark_completed(url) {
                    eprintln!("    ⚠ Warning: Failed to write to checkpoint ledger: {}", e);
                } else {
                    println!("    ✓ Checkpoint recorded.");
                }
            }
            Err(err) => {
                eprintln!("    ⚠ Failed to write file: {}", err);
            }
        }
    }

    println!("\n==> Processing completed.");
}
