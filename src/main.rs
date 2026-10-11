use std::env;
use std::path::PathBuf;
use std::process;
use yt2obsidian::{fetch_transcript, synthesize_transcript, ObsidianWriter};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <YOUTUBE_URL> [OLLAMA_MODEL] [VAULT_PATH]", args[0]);
        eprintln!("Example: {} \"https://www.youtube.com/watch?v=...\" qwen2.5:7b ./vault", args[0]);
        process::exit(1);
    }

    let url = &args[1];
    let model = if args.len() >= 3 { &args[2] } else { "qwen2.5:7b" };
    let vault_path = if args.len() >= 4 {
        PathBuf::from(&args[3])
    } else {
        PathBuf::from("./vault")
    };

    println!("==> Stage 1: Ingesting transcript from YouTube...");
    let transcript = match fetch_transcript(url) {
        Ok(t) => {
            println!("    Ingestion succeeded ({} characters).", t.full_text.len());
            t
        }
        Err(err) => {
            eprintln!("    Ingestion failed: {}", err);
            process::exit(1);
        }
    };

    println!("\n==> Stage 2: Synthesizing with Ollama (model: {})...", model);
    let report = match synthesize_transcript(&transcript.full_text, model).await {
        Ok(rep) => {
            println!("    Synthesis succeeded: {} Q&A pairs extracted.\n", rep.items.len());
            for (idx, item) in rep.items.iter().enumerate() {
                println!("[{}] Question: {}", idx + 1, item.question);
                println!("    Answer: {}", item.answer);
                println!("    Key Concepts: {}\n", item.key_concepts.join(", "));
            }
            rep
        }
        Err(err) => {
            eprintln!("    Synthesis failed: {}", err);
            process::exit(1);
        }
    };

    println!("==> Stage 3: Exporting to Obsidian vault at {:?}...", vault_path);
    let writer = ObsidianWriter::new(&vault_path);
    match writer.write_report(&report, url) {
        Ok(file_path) => {
            println!("    Successfully generated Obsidian note:");
            println!("    --> {}", file_path.display());
        }
        Err(err) => {
            eprintln!("    Storage failed: {}", err);
            process::exit(1);
        }
    }
}