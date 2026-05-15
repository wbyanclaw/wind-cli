# LLM Wiki SDK — Rust Crate Design v0.1
Owner: @ruler | Date: 2026-05-16 | Status: Draft

## Repository
- crates.io: `wind-wiki`
- Repository: TBD (github.com/wbyanclaw/wind-wiki)
- Language: Rust 2021

## Dependencies
- `anyhow` for error handling
- `serde` for config
- `reqwest` for HTTP (AI API calls)
- `tokio` for async runtime
- `walkdir` for file traversal
- `chrono` for timestamps
- `url` for URL/wikilink parsing

## Crate Structure
```
wind-wiki/
├── src/
│   ├── lib.rs         — public API
│   ├── ingest.rs      — Ingest Pipeline
│   ├── query.rs       — Query Pipeline
│   ├── lint.rs        — Lint Pipeline
│   ├── config.rs      — Config loading (shared with wind-cli)
│   ├── wiki.rs        — File operations (read/write/parse Markdown)
│   └── llm.rs        — LLM API client abstraction
├── Cargo.toml
└── examples/
    └── cli.rs         — Standalone CLI tool
```

## Config (各自独立)

**方案 B（独立配置）**：LLM Wiki SDK 有自己的 AI 配置，和 wind-demo 的对话 AI 独立。

wind-demo 配置 Claude 用于对话；LLM Wiki 配置独立 AI（可以更便宜/快的模型用于知识编撰）。

```toml
# ~/.config/wind/config.toml

[demo]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key = "sk-ant-..."

[wiki]
provider = "anthropic"       # 独立配置
model = "claude-haiku-4-20250514"  # 可用更便宜的模型
api_key = "sk-ant-..."    # 独立 key
wiki_path = "~/.local/share/wind/wiki"
raw_path = "~/.local/share/wind/raw"
system_md = "~/.config/wind/SYSTEM.md"
```

wind-demo UI 展示两套配置面板：[demo AI] 和 [wiki AI]。
```toml
# ~/.config/wind/config.toml
[llm]
provider = "anthropic"  # or "openai"
model = "claude-sonnet-4-20250514"
api_key = "sk-..."

[wiki]
raw_path = "~/.local/share/wind/raw"
wiki_path = "~/.local/share/wind/wiki"
system_md = "~/.config/wind/SYSTEM.md"
```

## Public API

```rust
use wind_wiki::{Wiki, Config};

let wiki = Wiki::new(Config::from_default()?)?;

/// Ingest Pipeline: source file → AI edit → wiki Markdown
wiki.ingest("docs/report.pdf").await?;

/// Query Pipeline: user question → AI read wiki → answer
let answer = wiki.query("Q1营收情况如何？").await?;

/// Lint Pipeline: audit wiki → fix deadlinks/conflicts
wiki.lint().await?;

/// Status
let stats = wiki.status()?;
```

## Pipeline Designs

### Ingest Pipeline
1. Read source file (PDF/HTML/Markdown)
2. Extract text content
3. Call LLM with prompt: "编写/更新 wiki 页面，参考 SYSTEM.md"
4. Write result to `wiki_path/`
5. Update wikilinks if needed

### Query Pipeline
1. Parse user question
2. Call LLM: read relevant wiki pages + answer
3. Return integrated answer

### Lint Pipeline
1. Walk all `.md` files in wiki_path
2. Call LLM: check for deadlinks, logical contradictions
3. Auto-fix in place
4. Report changes

## Command-Line Interface (example)
```bash
wind-wiki ingest docs/report.pdf
wind-wiki query "Q1营收如何"
wind-wiki lint
wind-wiki status
```
