# wind-demo — Product Design Specification

> AI Agent 智能文件管理演示平台 · Natural Language Interface

## Overview

wind-demo is a client-facing demo application that demonstrates how AI Agents use wind-cli for secure file management through natural language interaction.

**Core value proposition**: Zero-configuration wind-cli experience, visualized Agent → CLI → File System interaction chain.

---

## Product Vision

> "Any user — technical or not — can interact with wind-cli through natural language, seeing in real-time how AI Agent translates intent into secure file operations."

---

## User Stories

| As a... | I want to... | So that... |
|---------|-------------|-----------|
| Client | See how AI Agent manages files | I can evaluate wind-cli capabilities |
| Developer | Test wind-cli with natural language | I can quickly prototype Agent workflows |
| Sales | Demo wind-cli to prospects | I can show value in under 2 minutes |

---

## Design Language

### Theme: Fintech Light (Client-Facing)

**Rationale**: Professional, trustworthy, accessible to non-technical clients. Dark themes feel "hacker-y"; fintech light says "enterprise-ready."

| Token | Value | Usage |
|-------|-------|-------|
| `--bg` | `#f4f6f9` | Page background |
| `--surface` | `#ffffff` | Cards, panels |
| `--accent` | `#0052cc` | Primary actions, links |
| `--green` | `#00875a` | Success, connected status |
| `--purple` | `#6554c0` | Wiki, knowledge base |
| `--yellow` | `#ff8b00` | Folders, warnings |
| `--text` | `#172b4d` | Primary text |
| `--shadow` | `0 1px 3px rgba(9,30,66,.08)` | Card elevation |

**Typography**: System fonts (PingFang SC, Helvetica Neue) — no external font loading needed.

---

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Header: wind-demo · AI Agent · 文件管理演示  [API状态] [v]│
├──────────┬──────────────────────────────────┬──────────────┤
│          │                                  │              │
│  工作区   │      对话面板                    │  知识库      │
│  文件树   │  (ChatGPT-style chat UI)       │  Wiki       │
│          │                                  │              │
│  • files │  [user] 列出工作区的文件         │  • 使用指南  │
│  • dirs  │  [agent] 好的...                │  • API规范   │
│          │       ⚡ windcli ls              │  • WFT集成   │
│          │       ✓ result                   │              │
│          ├──────────────────────────────────┤              │
│  工具链   │  [输入框: 用自然语言描述需求...] │  协议链路    │
│  • wind  │                                  │  API配置     │
│  • WFT   │                                  │  安装入口    │
└──────────┴──────────────────────────────────┴──────────────┘
```

### Left Sidebar — Workspace File Tree

- Real-time directory structure of current workspace
- Updates after each AI Agent action
- Clicking a file shows its metadata
- Sections: "目录结构" (files/dirs), "AI 工具链" (wind-cli, WFT Terminal)

### Center — Conversation Panel

- ChatGPT-style interface
- User inputs natural language
- AI Agent responds with thought process + tool call trace
- Tool call cards show: command → JSON result → human-readable summary

### Right Sidebar — Knowledge & Config

- **LLM Wiki 知识库**: Clickable cards linking to documentation
- **协议链路**: Visual flow diagram
- **API 配置**: Model, tools, connection status
- **安装入口**: Copy install command

---

## Core Features

### 1. Natural Language Conversation

User types in Chinese or English → AI Agent parses intent → calls wind-cli → returns result.

**Example flow**:
```
User: 帮我创建一个 notes 目录
AI: 好的，正在创建 notes 目录。
     ⚡ windcli mkdir notes
     ✓ exit 0
     { ok: true, path: "notes/", created: true }
AI: 已成功创建 notes/ 目录！
```

### 2. Workspace File Tree

- Displays live directory structure
- Updates after each command execution
- Hierarchical: workspace root → files/directories
- Shows file size, type icons

### 3. LLM Wiki Knowledge Base

**Architecture**:
```
用户提问
  ↓
AI Agent（单次请求，同时检索）
  ├── 知识库检索 → 本地 Wiki Markdown 索引（~/.local/share/wind/wiki/）
  └── wind-cli 工作区 → 文件系统操作
  ↓
结果聚合 → 统一回复给用户
```

**Implementation**:
- Local Markdown files as knowledge source (pre-compiled index)
- `windcli wiki query "Q1营收报告"` — fuzzy search titles/content → return relevant snippets
- Integrated as 10th wind-cli tool: `WikiQuery { query: String }`
- Can also be called via `windcli tools call wiki_query --params '{"query":"..."}'`

**Knowledge Base API**:
- Endpoint: local HTTP server (e.g., `localhost:8765`)
- Methods: `GET /search?q=...`, `GET /doc/{id}`
- Also callable through wind-cli's `tools call` interface

### 4. Protocol Flow Visualization

```
用户 (自然语言) ↔ AI Agent ↔ wind CLI
                              ↓
              工作区文件系统  或  windlocal:// → WFT 终端
```

---

## Component Specifications

### Conversation Bubbles

| Type | Style |
|------|-------|
| User | Right-aligned, accent blue background, white text |
| AI Agent | Left-aligned, white card with shadow, thinking indicator |
| Tool Call Card | Nested inside agent bubble, shows `$ cmd` + JSON result |

### Tool Call Card

```
┌─────────────────────────────────┐
│ ⚡ windcli — mkdir              │  ← header
├─────────────────────────────────┤
│ ✓ exit 0                       │  ← status badge
│ $ windcli mkdir notes           │  ← command
│ { ok: true, path: "notes/",    │  ← JSON result
│   created: true }               │
└─────────────────────────────────┘
```

### Preset Scenarios (v0.1)

1. **列出文件** — `windcli ls`
2. **创建目录** — `windcli mkdir`
3. **写入文件** — `echo "..." | windcli put ... --overwrite`
4. **WFT 打开** — `windcli wft file ...`

---

## Technical Architecture

### Stack

- **Runtime**: Tauri v2 (cross-platform, ~5MB)
- **Frontend**: Vanilla HTML/CSS/JS (no framework, minimal dependencies)
- **AI Integration**: Remote AI API (Claude/GPT-4o, user-configurable)
- **File Operations**: `std::process::Command` (Rust side) executing `windcli` CLI
- **Demo Workspace**: `$TMPDIR/wind-demo-{timestamp}/`, auto-cleanup on exit

### System Prompt (Draft)

```
You are an AI Agent powered by wind-cli.
You have access to the following tools:

1. ls — list directory contents
2. read — read file (≤10MB)
3. write — write file via stdin (requires --overwrite flag)
4. mkdir — create directory
5. rm — delete file or directory
6. extract — parse document content (PDF, Excel, PPT, HTML, IMG)
7. wft — dispatch windlocal action to WFT Terminal
8. workspace_info — get current workspace root
9. version_check — get version info
10. wiki_query — search local knowledge base (Markdown files)

All paths are confined to the workspace directory.
Never perform operations outside the workspace.
Output structured JSON for all tool results.
```

### AI API Configuration

| Field | Default |
|-------|---------|
| Provider | Anthropic (Claude) |
| Model | claude-sonnet-4-20250514 |
| Tools | windcli (10 functions) |
| API Key | User-provided, stored locally |

---

## v0.1 Scope

### In Scope
- ✅ NL conversation (single-turn)
- ✅ 4 preset demo scenarios
- ✅ Workspace file tree (display only)
- ✅ LLM Wiki panel (static cards → future: live search)
- ✅ Protocol flow diagram
- ✅ API configuration panel
- ✅ Install entry

### Out of Scope (Future)
- ❌ Real-time streaming log output
- ❌ Command history / session persistence
- ❌ WFT Terminal simulation
- ❌ Live wiki search (static cards only in v0.1)
- ❌ Voice input

---

## Mockup Versions

| Version | Commit | Description |
|---------|--------|-------------|
| v1 | fb98f74 | Dark theme, button-based |
| v2 | ec0afc6 | Dark theme, NL interface |
| v3 | d24fd77 | Fintech light, Chinese, file tree, wiki |
| **v4** | current | Fintech light, refined layout |

URL: <https://github.com/wbyanclaw/wind-cli/blob/main/docs/wind-demo-mockup.html>

---

## Open Questions

- [ ] Wiki directory path: `~/.local/share/wind/wiki/` or configurable?
- [ ] Wiki index format: pure Markdown or front-matter?
- [ ] Wiki API interface: HTTP server or subprocess?
- [ ] Streaming output vs. batch result?

---

*Last updated: 2026-05-16*
*Maintained by: @coder, @ruler*
