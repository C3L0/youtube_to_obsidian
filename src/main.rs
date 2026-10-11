use std::env;
use std::process;
use yt2obsidian::{fetch_transcript, synthesize_transcript};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <YOUTUBE_URL> [OLLAMA_MODEL]", args[0]);
        eprintln!("Example: {} \"https://www.youtube.com/watch?v=...\" llama3.2", args[0]);
        process::exit(1);
    }

    let url = &args[1];
    let model = if args.len() >= 3{
        &args[2]
    } else {
        "llama3.2"
    };

    println!(" Stage 1: Ingesting transcript from YouTube...");
    let transcript = match fetch_transcript(url) {
        Ok(t) => {
            println!("  ingestion succeeded({} characters).", t.full_text.len());
            t
        }
        Err(err) => {
            eprintln!(" ingestion failed: {}", err);
            process::exit(1);
        }
    };

    println!(" Stage 2: Synthesizing with Ollama (model: {})...", model);
    println!("  (This may take a moment depending on transcript length)");    

    match synthesize_transcript(&transcript.full_text, model).await {
        Ok(report) => {
            println!("Synthesis Succeeded");
            println!("Main Topic: {}\n", report.main_topic);
        
            for (idx, item) in report.items.iter().enumerate() {
                println!("[{}] Question: {}", idx+1, item.question);
                println!("  Answer: {}", item.answer);
                println!("  Key Concepts: {}\n", item.key_concepts.join(", "));
            }
        }
        Err(err) => {
            eprintln!("\nSynthesis failed: {}", err);
            process::exit(1);
        }
    }
}