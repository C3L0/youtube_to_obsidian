//! Ingestion Logic

use std::env;
use std::fmt;
use std::fs;
use std::process::Command;

#[derive(Debug)]
pub enum IngestionError {
    ProcessSpawnFailed(std::io::Error),
    ExecutionFailed { status_code: Option<i32>, stderr: String },
    FileReadFailed(std::io::Error),
    EmptyTranscript,
}

impl fmt::Display for IngestionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessSpawnFailed(err) => write!(f, "Failed to spawn yt-dlp: {}", err),
            Self::ExecutionFailed { status_code, stderr } => {
                write!(f, "yt-dlp failed (code {:?}): {}", status_code, stderr.trim())
            }
            Self::FileReadFailed(err) => write!(f, "Failed to read subtitle file: {}", err),
            Self::EmptyTranscript => write!(f, "No subtitle content was extracted or matched"),
        }
    }
}

impl std::error::Error for IngestionError {}

#[derive(Debug, Clone)]
pub struct Transcript {
    pub video_url: String,
    pub full_text: String,
}

pub fn fetch_transcript(video_url: &str) -> Result<Transcript, IngestionError> {
    let raw_vtt = run_ytdlp(video_url)?;
    let clean_text = clean_vtt(&raw_vtt)?;

    Ok(Transcript {
        video_url: video_url.to_string(),
        full_text: clean_text,
    })
}

fn run_ytdlp(url: &str) -> Result<String, IngestionError> {
    // 1. Create a unique file stem in the OS temp directory
    let temp_dir = env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let out_template = temp_dir.join(format!("ytsub_{}", unique_id));
    let template_str = out_template.to_string_lossy().to_string();

    // 2. Run yt-dlp pointing to the temporary output template
    let output = Command::new("yt-dlp")
        .args([
            "--skip-download",
            "--write-auto-sub",
            "--write-sub",
            "--sub-lang", "en.*,en",
            "--sub-format", "vtt",
            "-o", &template_str,
            url,
        ])
        .output()
        .map_err(IngestionError::ProcessSpawnFailed)?;

    if !output.status.success() {
        return Err(IngestionError::ExecutionFailed {
            status_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    // 3. Locate the generated .vtt file (e.g. ytsub_123.en.vtt or ytsub_123.en-orig.vtt)
    let prefix = format!("ytsub_{}", unique_id);
    let mut found_file = None;

    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(&prefix) && filename.ends_with(".vtt") {
                found_file = Some(entry.path());
                break;
            }
        }
    }

    let file_path = match found_file {
        Some(path) => path,
        None => return Err(IngestionError::EmptyTranscript),
    };

    // 4. Read the file into memory and remove it from disk
    let vtt_content = fs::read_to_string(&file_path)
        .map_err(IngestionError::FileReadFailed)?;
    let _ = fs::remove_file(file_path);

    Ok(vtt_content)
}

fn clean_vtt(vtt_data: &str) -> Result<String, IngestionError> {
    let mut collected_lines: Vec<String> = Vec::new();
    let mut last_line = String::new();

    for line in vtt_data.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with("WEBVTT")
            || trimmed.starts_with("Kind:")
            || trimmed.starts_with("Language:")
            || trimmed.contains("-->")
        {
            continue;
        }

        let sanitized = strip_tags(trimmed);
        let cleaned = sanitized.trim();

        if cleaned.is_empty() {
            continue;
        }

        if cleaned != last_line {
            collected_lines.push(cleaned.to_string());
            last_line = cleaned.to_string();
        }
    }

    if collected_lines.is_empty() {
        return Err(IngestionError::EmptyTranscript);
    }

    Ok(collected_lines.join(" "))
}

fn strip_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside_tag = false;

    for c in input.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(c),
            _ => {}
        }
    }

    result
}