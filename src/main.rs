use clap::Parser;
use std::path::PathBuf;
use std::process;
use yt2obsidian::{
    fetch_channel_shorts, fetch_transcript, synthesize_transcript, CheckpointManager,
    ObsidianWriter,
};

#[derive(Parser, Debug)]
#[command(
    name = "yt2obsidian",
    about = "Extracts YouTube transcripts and generates structured Obsidian notes using a local LLM."
)]
struct Cli {
    /// YouTube channel URL (e.g. https://www.youtube.com/@Channel) or single video URL
    #[arg(required = true)]
    target: String,

    /// Ollama model to use for synthesis
    #[arg(short, long, default_value = "qwen2.5:7b")]
    model: String,

    /// Target Obsidian vault directory
    #[arg(short, long, default_value = r"C:\Users\antoi\OneDrive\Documents\ytb2Obsidian")]
    vault: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

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
                println!("    ✓ Transcript ingested ({} characters).", t.full_text.len());
                t
            }
            Err(err) => {
                eprintln!("    ⚠ Skipping video (ingestion error): {}", err);
                continue;
            }
        };

        // Stage 2: Synthesis
        println!("    Synthesizing with Ollama (model: {})...", args.model);
        let report = match synthesize_transcript(&transcript.full_text, &args.model).await {
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
        let write_result = writer.write_report(&report, url);
        match write_result {
            Ok(file_path) => {
                println!("    ✓ Saved note: {}", file_path.display());
                // Stage 4: Commit checkpoint to ledger
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