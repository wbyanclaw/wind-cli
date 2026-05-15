# wind-demo — Product Requirements Document v0.4
Owner: @ruler | Date: 2026-05-16 | Status: Ready for review

## 1. 背景

wind-demo 是面向金融客户的**产品级演示工具**：展示"自然语言 + AI Agent + wind-cli（统一入口）+ LLM Wiki SDK"的人机协作体验。

**架构关系**：
```
wind-cli（统一入口，单一 Rust 二进制）
  ├── 文件管理工具（ls/read/write/mkdir/rm/extract/wft/workspace_info/version）
  └── LLM Wiki SDK（wind-wiki crate，crates.io）→ 三 Pipeline
       ├── Ingest：源文件 → AI 编撰 → wiki Markdown
       ├── Query：用户问题 → AI 读 wiki → 融会贯通回答
       └── Lint：定时审计 → 修复死链/矛盾
```

---

## 2. UI 布局

```
┌─ Header: wind-demo + 导航 + API状态 + 版本 ─────────────────┐
├─ 左侧栏 ───────┬─ 中央对话 ───────┬─ 右侧栏 ──────────┤
│ 📂 工作区文件树  │  对话气泡        │ ⚡ 协议链路      │
│ （实时更新）      │ NL输入+AI回复    │ 📋 执行链路      │
│               │               │ 📦 安装入口     │
│ 🧠 LLM Wiki   │ 快捷场景按钮      │               │
│ 三大目录文件树   │               │               │
│ （展示Wiki结构） │               │               │
└───────────────┴───────────────┴───────────────┘
```

**UI 风格**：金融科技风（蓝白配色，非暗黑），全中文。

---

## 3. 核心模块

### 3.1 工作区文件树
- 实时显示 wind-cli workspace 目录结构
- AI 执行命令后树自动更新
- 目录/文件图标区分

### 3.2 LLM Wiki 三大目录
- `/raw`（只读原始资料）、`/wiki`（AI 编撰知识）、`SYSTEM.md`（行为准则）
- 文件树展示 Wiki 目录结构
- 展示 Ingest/Query/Lint Pipeline 状态

### 3.3 对话界面
- 全中文 NL 输入
- AI 回复气泡（thought + tool call + result）
- 快捷场景按钮（ls/mkdir/write/wft）
- wind wiki 子命令结果展示

### 3.4 执行链路（右侧）
- Human → AI → wind CLI/wiki → 结果
- JSON 高亮格式化

### 3.5 协议链路图（静态）
- Human ↔ AI Agent ↔ wind CLI ↔ Workspace / windlocal:// → WFT

### 3.6 安装入口

---

## 4. wind-cli 新增命令

### 4.1 wind wiki query
- `wind wiki query <text>` — 查询本地知识库
- 返回相关 Markdown 内容

### 4.2 wind wiki ingest（LLM Wiki SDK Pipeline）
- `wind wiki ingest <file>` — 源文件 → AI 编撰 → wiki/
- 依赖 wind-wiki crate

### 4.3 wind wiki lint
- `wind wiki lint` — 审计 wiki/ 修复死链/矛盾

### 4.4 wind wiki status
- `wind wiki status` — 显示 wiki 统计

---

## 5. LLM Wiki SDK（wind-wiki crate）

- **语言**：Rust 2021
- **位置**：独立 crates.io 包（wind-cli dependency）
- **三目录**：`/raw`（源资料）、`/wiki`（知识）、`SYSTEM.md`（准则）
- **三 Pipeline**：Ingest / Query / Lint
- **AI 配置**：独立（可不同于 wind-demo 的对话 AI）

---

## 6. 验收标准

| # | 检查项 | 标准 |
|---|--------|------|
| 1 | 工作区文件树实时更新 | mkdir 后显示新目录 |
| 2 | LLM Wiki 目录树展示 | raw/wiki/SYSTEM.md 三目录可见 |
| 3 | wind wiki query 执行成功 | 返回相关 Markdown 内容 |
| 4 | wind wiki ingest Pipeline 可视 | 展示编撰过程 |
| 5 | 全中文 Fintech 风格 UI | 金融科技风非暗黑 |
| 6 | API 配置后正常对话 | 可配置 Claude/GPT key |

---

## 7. 范围外
- 语音输入
- 向量数据库（纯 Markdown 检索）
- 流式输出

## 8. Mockup
- Fintech 风格 v4：https://github.com/wbyanclaw/wind-cli/blob/main/docs/wind-demo-mockup.html
