# Hermes Newsletter Script

A zero-config Rust CLI that aggregates tech news from 11 sources, filters by your topics of interest, and outputs clean JSON. Built to run as a Hermes Agent cronjob skill — works as a standalone binary with no runtime dependencies.

## Quick Start

```bash
# 1. Create your tags file
mkdir -p ~/.hermes
echo '{ "tags": ["rust", "kubernetes", "ai"] }' > ~/.hermes/newsletter-tags.json

# 2. Fetch everything
cargo run

# 3. Pick items from the output, then get summaries
FETCH_TLDR="https://dev.to/user/article-1,https://arstechnica.com/..." cargo run
```

## Installation

### One-liner install

```bash
./install.sh
```

This will:
- Build the binary (uses local Rust if available, otherwise builds in Docker)
- Copy it to `/usr/local/bin/`
- Create a default tags file at `~/.hermes/newsletter-tags.json` if missing

### Manual build

```bash
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/hermes_newsletter_script /usr/local/bin/
```

### Docker only (no Rust needed)

```bash
docker build -t hermes-newsletter .
docker cp $(docker create hermes-newsletter):/usr/local/bin/hermes_newsletter_script ./hermes_newsletter_script
```

## Usage

### Pass 1 — Discover

Fetch all sources, filter by tags, get a brief list:

```bash
cargo run
```

Output (stdout):

```json
[
  {
    "id": "https://dev.to/user/building-a-rag-system-in-rust-1a2b",
    "title": "Building a RAG System in Rust with Qdrant",
    "tags": ["rust", "ai", "rag"]
  },
  {
    "id": "https://arstechnica.com/ai/2026/06/new-llm-benchmark/",
    "title": "New Benchmark Shows LLMs Improving at Code Generation",
    "tags": ["AI", "LLMs", "benchmarking"]
  },
  {
    "id": "https://huggingface.co/google/gemma-3-9b",
    "title": "google/gemma-3-9b",
    "tags": ["transformers", "safetensors"],
    "source": "huggingface",
    "summary": "Pipeline: text-generation | Author: google | Likes: 4200 | Downloads: 150000"
  }
]
```

Review the list, pick the IDs you want to read.

### Pass 2 — Enrich

Get full summaries for the items you selected:

```bash
FETCH_TLDR="https://dev.to/user/building-a-rag-system-in-rust-1a2b" cargo run
```

Output (stdout):

```json
[
  {
    "id": "https://dev.to/user/building-a-rag-system-in-rust-1a2b",
    "title": "Building a RAG System in Rust with Qdrant",
    "tags": ["rust", "ai", "rag"],
    "source": "dev.to",
    "summary": "In this post I walk through building a retrieval-augmented generation pipeline using Qdrant as the vector store and Rig as the orchestration framework. The goal was to keep everything in Rust for performance and type safety..."
  }
]
```

### Writing to a file

```bash
OUTPUT_FILE=output.json cargo run
```

Logs always go to stderr, so you can safely pipe stdout:

```bash
cargo run | jq '.[] | .title'
```

## Configuration

### Tags

Create `~/.hermes/newsletter-tags.json`:

```json
{ "tags": ["rust", "kubernetes", "ai", "security"] }
```

- Tags are matched case-insensitively
- Against item tags: exact match (`"ai"` matches `["AI", "ML"]`)
- Against titles: whole-word match (`"ai"` matches `"AI Agents"` but not `"Air Quality"`)

#### Use a local tags file (for development)

```bash
TAGS_SOURCE=local cargo run
```

This reads from `./newsletter-tags.json` in the current directory instead of `~/.hermes/`.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TAGS_SOURCE` | `~/.hermes/` | Set to `local` to read `./newsletter-tags.json` |
| `FETCH_TLDR` | *(none)* | Comma-separated IDs for pass 2 (enrichment) |
| `OUTPUT_FILE` | stdout | Path to write JSON output |

## Sources

### RSS Feeds

| Source | Content |
|--------|---------|
| Ars Technica | Tech, science, policy |
| The New Stack | Cloud-native, DevOps |
| dev.to | Community articles (filtered by your tags) |
| Schneier on Security | Security analysis |
| Hacker News (RSS) | Frontpage stories |
| Krebs on Security | Cybersecurity journalism |

### REST APIs

| Source | Content | Tag filtering |
|--------|---------|:---:|
| CISA KEV | Actively exploited vulnerabilities | bypassed |
| NVD CVE | HIGH/CRITICAL severity CVEs | bypassed |
| HuggingFace Papers | Daily trending ML papers | matched |
| HuggingFace Models | Trending models (pipeline, likes, downloads) | bypassed |
| Hacker News API | Top 20 stories | bypassed |

Sources marked "bypassed" always appear regardless of your tags — they're inherently relevant.

## Output Format

### Pass 1 (brief)

```json
[{ "id": "...", "title": "...", "tags": [...] }]
```

### Pass 2 (enriched)

```json
[{
  "id": "...",
  "title": "...",
  "tags": [...],
  "source": "dev.to",
  "summary": "..."
}]
```

Summaries are:
- **Sanitized** — HTML tags stripped, entities decoded (`&#8217;` → `'`)
- **Truncated** — capped at 600 chars with `...` suffix
- **Enriched** — dev.to articles re-fetch the full body for meaningful excerpts

## Hermes Integration

The binary is fully static (musl) — just copy it to the Hermes container. No Docker, no runtime dependencies.

### 1. Build & copy

```bash
# on your machine
./install.sh

# copy to the VPS
scp /usr/local/bin/hermes_newsletter_script user@vps:/usr/local/bin/
```

### 2. Set up tags on the VPS

```bash
ssh user@vps
mkdir -p ~/.hermes
echo '{ "tags": ["rust", "kubernetes", "ai"] }' > ~/.hermes/newsletter-tags.json
```

### 3. Reference in docker-compose

In your Hermes compose file, mount the binary:

```yaml
services:
  hermes:
    # ... your existing hermes config ...
    volumes:
      - /usr/local/bin/hermes_newsletter_script:/usr/local/bin/newsletter:ro
      - ~/.hermes:/root/.hermes:ro
```

### 4. Customize & reinstall

```bash
# edit source
vim src/fetchers/api.rs

# rebuild & copy
./install.sh
scp /usr/local/bin/hermes_newsletter_script user@vps:/usr/local/bin/
```

### Typical workflow

1. Hermes (or you) runs the binary
2. Script fetches all sources, outputs the brief JSON list to stdout
3. You review and select relevant IDs
4. Re-run with `FETCH_TLDR=<ids>` to get enriched summaries
5. Summaries go into the newsletter

## Tech Stack

- **reqwest** — HTTP client
- **feed-rs** — RSS/Atom parser
- **tokio** — async runtime
- **serde** / **serde_json** — serialization
