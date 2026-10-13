use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct CheckpointManager {
    file_path: PathBuf,
    processed: HashSet<String>,
}

impl CheckpointManager {
    /// Loads existing checkpoint records from disk, or initializes an empty set if the file does not exist.
    pub fn init<P: AsRef<Path>>(vault_dir: P) -> io::Result<Self> {
        let file_path = vault_dir.as_ref().join(".processed_shorts.txt");
        let mut processed = HashSet::new();

        if file_path.exists() {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let entry = line?.trim().to_string();
                if !entry.is_empty() {
                    processed.insert(entry);
                }
            }
        }

        Ok(Self {
            file_path,
            processed,
        })
    }

    /// Checks in O(1) time if the target has already been successfully processed.
    pub fn is_processed(&self, id_or_url: &str) -> bool {
        self.processed.contains(id_or_url)
    }

    /// Appends the newly completed target to the file on disk and updates the in-memory set.
    pub fn mark_completed(&mut self, id_or_url: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", id_or_url)?;
        self.processed.insert(id_or_url.to_string());
        Ok(())
    }

    pub fn total_processed(&self) -> usize {
        self.processed.len()
    }
}