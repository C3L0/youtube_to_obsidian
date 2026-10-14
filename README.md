# yt2obsidian

A local-first, zero-cost CLI pipeline written in Rust to ingest YouTube videos and Shorts, extract interview questions and technical concepts via local LLMs running on Ollama, and compile an interconnected knowledge graph in Obsidian.

---

## Architecture Overview

```
                                      [ CLI Target URL ]
                                              │
                      ┌───────────────────────┴───────────────────────┐
                      ▼                                               ▼
             [ YouTube Shorts Tab ]                          [ Single Video URL ]
                      │                                               │
                      ▼                                               │
           yt-dlp (--flat-playlist)                                   │
                      │                                               │
                      ▼                                               │
             [ Video URLs Stream ] <──────────────────────────────────┘
                      │
                      ▼
            [ Checkpoint Gate ] ────(Already In Ledger?)───► [ Skip / 0ms ]
                      │ (No)
                      ▼
             yt-dlp Subtitle Dump (VTT / JSON3)
                      │
                      ▼
            [ Text Normalization Engine ]
                      │
                      ▼
         Ollama Local Daemon (/api/generate)
         (Schema Enforcement: JSON Format Mode)
                      │
                      ▼
           [ Deserialization Layer ] (Serde Defenses & Aliases)
                      │
                      ▼
          [ Obsidian Storage Writer ]
          ├── Frontmatter (YAML metadata)
          └── Knowledge Graph Linking ([[Wikilinks]])
                      │
                      ▼
         [ Checkpoint Ledger Commit ] (.processed_shorts.txt)
```

---

## Key Technical Decisions

* **Zero-Cost & Local-First:** Runs entirely on commodity hardware without external API keys or recurring cloud fees.
* **Deterministic Structured Extraction:** Uses low-temperature sampling (`temp: 0.1`) and native JSON mode in Ollama, backed by defensive Serde deserialization (`#[serde(default)]`, field aliases) to withstand schema deviations.
* **Resilient Batch Processing:** Implements an $O(1)$ amortized append-only ledger (`.processed_shorts.txt`) using `HashSet<String>` to prevent duplicate processing across interrupted batch runs.
* **Graph Emergence:** Employs Obsidian `[[Wikilinks]]` for key concepts, allowing an interconnected concept map to emerge organically in Obsidian's Graph View.
* **Decoupled Architecture:** Built around an asynchronous `Summarizer` trait (`async-trait`), enabling test mocking and modular backend substitution without altering pipeline orchestration.

---

## Prerequisites

Ensure the following tools are installed and accessible in your system `$PATH`:

1. **Rust Toolchain:** Stable compiler ($\ge$ 1.80) & Cargo.
2. **`yt-dlp`:** Required for subtitle and playlist extraction.
3. **Ollama:** Running locally with a compatible instruction-tuned model:
   ```bash
   ollama pull qwen2.5:7b
   ```

---

## Installation & Setup

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/](https://github.com/)<your-username>/yt2obsidian.git
   cd yt2obsidian
   ```

2. **Configure Environment Defaults (Optional):**
   Copy the example environment configuration:
   ```bash
   cp .env.example .env
   ```
   Edit `.env` to match your local setup:
   ```ini
   OLLAMA_MODEL=qwen2.5:7b
   VAULT_PATH=C:/Users/username/Documents/MyVault
   PROMPT_TEMPLATE_PATH=./prompts/synthesis.txt
   ```

3. **Build the binary:**
   ```bash
   cargo build --release
   ```

---

## Usage

### Process an Entire Channel's Shorts
The tool automatically appends `/shorts`, scans the playlist metadata in flat mode, and iterates through each item:
```bash
cargo run -- "[https://www.youtube.com/@ChannelName/shorts](https://www.youtube.com/@ChannelName/shorts)"
```

### Process a Single Video
Target any standard YouTube video URL directly:
```bash
cargo run -- "[https://www.youtube.com/watch?v=VIDEO_ID](https://www.youtube.com/watch?v=VIDEO_ID)"
```

### CLI Overrides
You can override configured environment variables on the fly using command-line arguments:
```bash
# Override the model
cargo run -- "[https://www.youtube.com/watch?v=VIDEO_ID](https://www.youtube.com/watch?v=VIDEO_ID)" -m llama3.1:8b

# Override the destination vault directory
cargo run -- "[https://www.youtube.com/watch?v=VIDEO_ID](https://www.youtube.com/watch?v=VIDEO_ID)" -v "./test_vault"

# View full documentation
cargo run -- --help
```

---

## Output Format Example

Notes are exported directly as Obsidian-ready `.md` files containing YAML frontmatter and linked wikilinks:

```markdown
---
title: "Distributed Inference Techniques for Large AI Models"
source: "[https://www.youtube.com/shorts/](https://www.youtube.com/shorts/)..."
type: interview-qna
tags:
  - interview
  - tech
---

# Distributed Inference Techniques for Large AI Models

*Source: [Link](https://www.youtube.com/shorts/...)*

## Questions & Answers

### 1. What is Tensor Parallelism and how does it function?

Tensor parallelism splits each individual model layer across multiple GPUs, computing partial tensor mathematics concurrently before all-reducing intermediate states.

**Related Concepts:** [[Tensor-Parallelism]] [[All-Reduce]] [[GPU-Memory]]

---
```

---

## License

MIT