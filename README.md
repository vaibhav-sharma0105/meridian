# Meridian

> Every minute before and after a meeting should be worth more than the meeting itself.

Meridian is a **local-first, AI-powered meeting intelligence desktop application** built for top 1% operators. It turns any meeting transcript into structured tasks, project context, and AI-powered outputs — in seconds.

## Features

- **Transcript Ingestion** — Paste any transcript (Zoom, Google Meet, Teams, manual) and extract structured tasks with AI
- **Smart Task Management** — List, Kanban, and Table views with inline editing, bulk actions, and filters
- **Confidence Scoring** — Know which assignees and due dates were explicitly stated vs. inferred from context
- **Document Intelligence** — Upload PDFs, DOCX, CSV, PPTX, or URLs; query them with AI via FTS5 + Ollama semantic search
- **AI Chat Panel** — 5 output templates: 2×2 Leadership Update, Jira Ticket, Next Agenda, Status Report, Free-form
- **Meeting Health Score** — 0–100 score measuring agenda clarity, task distribution, follow-through, and efficiency
- **Analytics Dashboard** — Velocity charts, assignee workload heatmap, follow-through timeline
- **Secure by Default** — API keys stored in OS keychain (Windows Credential Manager / macOS Keychain / Linux Secret Service)
- **Multi-language** — English, Hindi, Gujarati
- **Export / Import** — JSON, CSV, Markdown formats
- **Auto-backup** — Database backed up before every migration

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) v18+
- Windows: [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Development

```bash
git clone https://github.com/yourusername/meridian.git
cd meridian
npm install
npm run dev
```

### Production Build

```bash
npm run build
# Installer: src-tauri/target/release/bundle/
```

### Optional: Ollama for Semantic Search

```bash
# Install from https://ollama.ai/download
ollama pull nomic-embed-text
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 (Rust + WebView2) |
| Frontend | React 18 + TypeScript 5 + Vite |
| Styling | Tailwind CSS v3 |
| State | Zustand + @tanstack/react-query |
| Database | SQLite with FTS5 (rusqlite) |
| AI gateway | LiteLLM (HTTP) |
| Local embeddings | Ollama (optional) |
| Secrets | OS keychain (keyring crate) |

## Architecture

```
Frontend (React + TypeScript)
    ↕  Tauri IPC commands / events
Backend (Rust + Tokio async)
    ↕  rusqlite + FTS5
SQLite (~/.meridian/meridian.db)
    ↕  HTTP
LiteLLM / Ollama / OpenAI / Anthropic / Gemini
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Command palette |
| `Ctrl+N` | New task |
| `Ctrl+Shift+N` | New project |
| `Ctrl+M` | New meeting |
| `Ctrl+/` | Focus AI chat |
| `Ctrl+E` | Export project |
| `Ctrl+1/2/3` | Switch task view |
| `]` | Toggle right panel |
| `[` | Toggle sidebar |

## Data & Privacy

All data is stored locally in `%APPDATA%\meridian\` (Windows) or `~/.meridian/` (macOS/Linux). No data is sent to any server except your configured AI provider for inference.

## Contributing

See [docs/INSTALL.md](docs/INSTALL.md) for setup instructions and [agents/agent.md](agents/agent.md) for architecture overview.

## License

MIT
