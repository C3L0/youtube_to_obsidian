use std::env;
use std::process;
use yt2obsidian::fetch_transcript;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <YOUTUBE_URL>", args[0]);
        process::exit(1);
    }

    let url = &args[1];
    println!("Fetching transcript for: {}", url);
    
    match fetch_transcript(url) {
        Ok(transcript) => {
            println!("\n---Ingestion Succeeded ---");
            println!("Characteres retrieved: {}", transcript.full_text.len());
            println!("Preview (first 300 chars):");
            let preview: String = transcript.full_text.chars().take(300).collect();
            println!("{}...", preview);
        }
        Err(err) => {
            eprintln!("\nError during ingestion: {}", err);
            process::exit(1);
        }
    }
}