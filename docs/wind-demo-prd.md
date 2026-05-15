# wind-demo — PRD v0.5
Owner: @ruler | Date: 2026-05-16 | Status: Draft for review

## 1. 背景
wind-demo 是面向金融客户的**产品级演示工具**：展示"自然语言 + AI Agent + wind-cli + LLM Wiki"的人机协作体验。
wind-cli = 统一入口（单一 Rust binary）; LLM Wiki SDK = crates.io crate

## 2. 目录结构（简化）
```
~/.local/share/wind/
├── workspace/   # 单一工作目录（wind-cli 管理）
│   ├── greeting.txt
│   ├── notes/
│   └── *.md
└── wiki/       # LLM Wiki 知识沉淀（Ingest Pipeline 产物）
```

**Ingest Pipeline**：用户上传文档 → AI 从 /workspace 读取 → 编撰产物写入 /wiki/

## 3. UI 布局
```
┌─ Header ─────────────────────────────┬──────────────────┐
│ Left: 工作文件树（workspace/）        │ Right: 安装入口 │
│ - 实时更新                          │                   │
│ - 展示 /wiki 目录结构（知识沉淀）       │                   │
│                                      │                   │
├─ Chat ─────────────────────────────────────────────────┤
│ AI 回复内嵌 Protocol Chain（不单独面板）                     │
│ → AI 思考过程（inline）                              │
│ → 调用的 wind 命令（inline）                         │
│ → 执行结果（inline）                                │
│ 快捷按钮：ls / mkdir / write / wft / wiki query         │
└─────────────────────────────────────────────────────┘
```

**关键变化**：
- Protocol Chain 内嵌到对话气泡中，不单独右侧面板
- 工作区/知识库在左侧栏，不在对话里
- 界面更干净，对话为主

## 4. UI 风格
- Tailwind CSS + Lucide 图标（coder 提供）
- Fintech 蓝白配色，非暗黑
- 全中文

## 5. wind-cli 新增命令
```
wind wiki ingest <file>   # Ingest Pipeline
wind wiki query <text>    # Query Pipeline
wind wiki lint           # Lint Pipeline
wind wiki status         # 状态
```

## 6. LLM Wiki SDK
- crates.io: `wind-wiki`
- 独立 AI 配置（方案 B：可与 demo AI 分别配置）
- 三目录：workspace/（工作）+ wiki/（知识）+ SYSTEM.md（准则）
