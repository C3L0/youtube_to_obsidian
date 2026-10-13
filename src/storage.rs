use std::fs;
use std::io;
use std::path::{Path,PathBuf};
use crate::synthesis::SynthesisReport;

pub struct ObsidianWriter {
    vault_dir: PathBuf,
}

impl ObsidianWriter {
    pub fn new<P: AsRef<Path>>(vault_dir: P) -> Self {
        Self { 
            vault_dir: vault_dir.as_ref().to_path_buf(),
        }
    }

    pub fn write_report(
        &self,
        report: &SynthesisReport,
        source_url: &str,
    ) -> Result<PathBuf, io::Error> {
        fs::create_dir_all(&self.vault_dir)?;

        let safe_name = sanitize_filename(&report.main_topic);
        let file_path = self.vault_dir.join(format!("{}.md", safe_name));

        let mut content = String::with_capacity(2048);
        content.push_str("--- \n");
        content.push_str(&format!("title: \"{}\"\n", report.main_topic));
        content.push_str(&format!("source: \"{}\"\n", source_url));
        content.push_str("type: interview-qna\n");
        content.push_str("tags:\n");
        content.push_str("  - interview\n");
        content.push_str("  - tech\n");
        content.push_str("--- \n\n");

        content.push_str(&format!("# {}\n\n", report.main_topic));
        content.push_str(&format!("*Source: [{}]({})*\n\n", source_url, source_url));
        content.push_str("## Questions and Answers\n\n");
        
        for(idx, item) in report.items.iter().enumerate() {
            content.push_str(&format!("# {}. {}\n\n",idx+1, item.question));
            content.push_str(&format!("#{}\n\n", item.answer));

            if !item.key_concepts.is_empty(){
                let wikilinks = item
                    .key_concepts
                    .iter()
                    .map(|c| format!("[[{}]]", c.trim().replace(' ', "-")))
                    .collect::<Vec<_>>()
                    .join(" ");
                content.push_str(&format!("**Concepts:** {}\n\n", wikilinks));
            }

            content.push_str("--- \n\n");
        }

        fs::write(&file_path, content)?;
        Ok(file_path)
    }
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c{
            '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*' => '-',
            _ => c,
        })
        .collect();

    let trimmed = sanitized.trim().trim_matches('-');
    if trimmed.is_empty() {
        "untitled-interview".to_string()
    } else {
        trimmed.to_string()
    }
}