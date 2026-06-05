# Hermes Newsletter Script

A zero-config Rust CLI that aggregates tech news from 11 sources, filters by your topics of interest, and outputs clean JSON. Built to run as a Hermes Agent cronjob skill — works as a standalone binary with no runtime dependencies.

## Installation

### Step 1 — Clone the repo on your VPS

```bash
ssh user@vps
git clone <repo-url> /opt/hermes-newsletter
```

### Step 2 — Build the binary

```bash
cd /opt/hermes-newsletter
docker build -t newsletter-builder .
mkdir -p /opt/hermes-newsletter/bin
docker cp $(docker create newsletter-builder):/usr/local/bin/hermes_newsletter_script /opt/hermes-newsletter/bin/newsletter
```

This compiles the Rust binary inside Docker. No Rust needed on the VPS.

### Step 3 — Create the tags file

```bash
mkdir -p /root/.hermes
echo '{ "tags": ["rust", "kubernetes", "ai"] }' > /root/.hermes/newsletter-tags.json
```

### Step 4 — Add to your docker-compose

```yaml
services:
  hermes:
    # ... your existing hermes config ...
    volumes:
      - /opt/hermes-newsletter/bin/newsletter:/usr/local/bin/newsletter:ro
      - /root/.hermes:/root/.hermes
```

Hermes can now run `newsletter` directly. It reads tags from `/root/.hermes/newsletter-tags.json`.

### Step 5 — Run

```bash
# from inside the hermes container
newsletter

# or with enrichment
FETCH_TLDR="id1,id2,id3" newsletter
```

### Updating

```bash
cd /opt/hermes-newsletter
git pull
docker build -t newsletter-builder .
docker cp $(docker create newsletter-builder):/usr/local/bin/hermes_newsletter_script /opt/hermes-newsletter/bin/newsletter
```

## Multiple Instances

Each Hermes instance can have its own tags by mounting different host paths. The binary is shared.

```yaml
services:
  hermes-1:
    volumes:
      - /opt/hermes-newsletter/bin/newsletter:/usr/local/bin/newsletter:ro
      - /root/.hermes-1:/root/.hermes

  hermes-2:
    volumes:
      - /opt/hermes-newsletter/bin/newsletter:/usr/local/bin/newsletter:ro
      - /root/.hermes-2:/root/.hermes
```

Set up tags per instance on the VPS:

```bash
# instance 1 — AI focused
mkdir -p /root/.hermes-1
echo '{ "tags": ["ai", "machinelearning", "llm"] }' > /root/.hermes-1/newsletter-tags.json

# instance 2 — DevOps focused
mkdir -p /root/.hermes-2
echo '{ "tags": ["kubernetes", "docker", "devops"] }' > /root/.hermes-2/newsletter-tags.json
```

Each instance runs the same binary but sees different tags. No shared state.

## Usage

### Pass 1 — Discover

Fetches all 11 sources, filters by your tags, outputs a brief JSON list:

```bash
newsletter
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

### Pass 2 — Enrich

Get full summaries for the items you selected:

```bash
FETCH_TLDR="https://dev.to/user/building-a-rag-system-in-rust-1a2b" newsletter
```

Output (stdout):

```json
[
  {
    "id": "https://dev.to/user/building-a-rag-system-in-rust-1a2b",
    "title": "Building a RAG System in Rust with Qdrant",
    "tags": ["rust", "ai", "rag"],
    "source": "dev.to",
    "summary": "In this post I walk through building a retrieval-augmented generation pipeline using Qdrant as the vector store and Rig as the orchestration framework..."
  }
]
```

### Writing output to a file

```bash
OUTPUT_FILE=/root/.hermes/output.json newsletter
```

## Configuration

### Tags

Edit `/root/.hermes/newsletter-tags.json` on the VPS (or whatever path is mounted):

```json
{ "tags": ["rust", "kubernetes", "ai", "security"] }
```

- Tags are matched case-insensitively
- Against item tags: exact match (`"ai"` matches `["AI", "ML"]`)
- Against titles: whole-word match (`"ai"` matches `"AI Agents"` but not `"Air Quality"`)

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FETCH_TLDR` | *(none)* | Comma-separated IDs for pass 2 (enrichment) |
| `OUTPUT_FILE` | stdout | Path to write JSON output |
| `TAGS_SOURCE` | `~/.hermes/` | Set to `local` to read `./newsletter-tags.json` (for local dev) |

## Sources

### RSS Feeds

| Source | Content | Tag filtering |
|--------|---------|:---:|
| Ars Technica | Tech, science, policy | matched |
| The New Stack | Cloud-native, DevOps | matched |
| dev.to | Community articles (filtered by your tags) | matched |
| Schneier on Security | Security analysis | matched |
| Hacker News (RSS) | Frontpage stories | matched |
| Krebs on Security | Cybersecurity journalism | matched |

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

## Local Development

```bash
# install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# create local tags file
echo '{ "tags": ["rust", "kubernetes", "ai"] }' > newsletter-tags.json

# run
TAGS_SOURCE=local cargo run

# run with enrichment
TAGS_SOURCE=local FETCH_TLDR="id1,id2" cargo run
```

## Tech Stack

- **reqwest** — HTTP client
- **feed-rs** — RSS/Atom parser
- **tokio** — async runtime
- **serde** / **serde_json** — serialization
