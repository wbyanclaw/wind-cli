# wind-demo — Product Requirements Document v0.3

## 1. 背景

wind-demo 是面向金融客户的**产品级演示工具**：展示"自然语言 + AI Agent + wind-cli + 本地 LLM Wiki 知识库"的人机协作体验。

## 2. 交互架构

```
用户 ←→ AI Agent（单次请求，双路检索）
            ├── wind-cli（9个工具 + WikiQuery）
            │     ├── 文件操作（ls/read/write/mkdir/rm/extract）
            │     ├── WFT 集成（wft file/search/app/settings）
            │     └── 工作区查询（workspace_info）
            └── 本地 LLM Wiki（新增 WikiQuery 工具）
```

wind-cli 新增第10个工具：`wind wiki query <text>` — 查询本地 Markdown 知识库

## 3. UI 布局

```
┌─ Header: wind-demo + API状态 + 版本 ─────────────────┐
├─ 左侧栏 ───────┬─ 中央对话 ───────┬─ 右侧栏 ────────┤
│ 📂 工作区文件树   │  对话气泡        │ ⚡ 协议链路      │
│ （实时更新）     │ NL输入+AI回复    │ 📋 执行链路      │
│               │               │ 📦 安装入口     │
│ 🧠 LLM Wiki   │ 场景按钮        │               │
│ 知识库面板      │               │               │
└───────────────┴───────────────┴───────────────┘
```

## 4. UI 风格
- 金融科技风：浅底蓝绿配色，适合客户演示
- 全中文界面
- 非暗黑

## 5. 核心模块

### 5.1 工作区文件树
- 实时显示 workspace 目录结构
- AI 执行命令后树自动更新
- 目录/文件图标区分

### 5.2 本地 LLM Wiki 知识库
- 来源：`~/.local/share/wind/wiki/*.md`（Markdown 文件）
- 启动时预加载到内存（或按需加载）
- `wind wiki query <text>` 模糊匹配标题/内容，返回相关片段
- 作为 wind-cli 第10个工具接入 AI Agent

### 5.3 对话界面
- 全中文自然语言输入
- AI 回复气泡（thought + tool call + result）
- 预设演示场景按钮

### 5.4 执行链路
- Human → AI → wind CLI / Wiki → 结果
- JSON 高亮格式化

### 5.5 协议链路图
- 静态四层：Human ↔ AI Agent ↔ wind CLI ↔ Workspace / windlocal:// → WFT

### 5.6 安装入口

## 6. wind-cli 新增功能
- `wind wiki query <text>` — 第10个工具，查询本地知识库
- 知识库目录：`~/.local/share/wind/wiki/`（可配置）

## 7. 验收标准
1. 文件树实时更新（mkdir 后显示新目录）
2. LLM Wiki 双路检索（"结合报告说工作区文件" → 同时返回两者）
3. `wind wiki query` 执行成功
4. 全中文 Fintech 风格 UI

## 8. 开放问题
- Wiki 目录路径是否可配置？
- 索引格式：纯 Markdown 还是 front-matter？
- Wiki 调用走 `tools call wiki_query`？

## 9. 范围外
- 语音输入
- 向量数据库（纯 Markdown 检索）
- 流式输出

## 10. Mockup
- Fintech 风格 Mockup v2: https://gist.github.com/wbyanclaw/5f144b6082719997980c9d6b9c93b066
