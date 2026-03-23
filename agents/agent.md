# Meridian AI Agent

## What is the Meridian agent?

The Meridian agent is a conversational AI assistant embedded in the app (AI Chat panel). It has read-only access to your project data — tasks, meetings, documents — and can answer questions, generate structured outputs, and help you prepare for upcoming meetings.

The agent runs entirely through your configured AI provider (OpenAI, Anthropic, Gemini, Groq, or any LiteLLM-compatible endpoint). No data leaves your machine except the API call to your chosen provider.

---

## What the agent can do

| Capability | Example prompt |
|-----------|----------------|
| Summarise project status | "What's the current state of this project?" |
| Find overdue tasks | "Which tasks are overdue and who owns them?" |
| Generate a 2×2 update | "Write a 2×2 update for my stakeholders" |
| Draft next meeting agenda | "Create an agenda for our next sprint planning" |
| Write Jira tickets | "Turn the open tasks into Jira ticket descriptions" |
| Answer document questions | "What does the spec say about authentication?" |
| Identify blockers | "Are there any blockers or dependencies I should know about?" |
| Analyse meeting health | "How have our meeting health scores trended this month?" |

---

## Context the agent receives

For every conversation turn, the agent is given:

1. **Project name**
2. **Open tasks** (up to 50) — title, assignee, due date
3. **Recently completed tasks** (up to 20)
4. **Recent meeting summaries** (up to 3)
5. **Relevant document chunks** — retrieved via FTS5 keyword search and (if Ollama is running) semantic similarity search

The agent does **not** receive:
- Raw transcripts (only AI-generated summaries)
- Document content beyond the top-5 matching chunks
- Data from other projects

---

## Output templates

Templates pre-fill the system and user prompts for common use cases. You can view, edit, and create templates in **Settings → AI & Models → Prompt Templates**.

Built-in templates:

| Template | Output format |
|---------|--------------|
| `2x2_update` | Four quadrants: Accomplished, Next, Risks, Decisions |
| `jira_tickets` | Markdown table of Jira-ready issue descriptions |
| `next_agenda` | Numbered agenda with time allocations |
| `status_report` | Executive summary paragraph + bullet points |

---

## Conversation history

Chat messages are saved to the `chat_history` table, scoped to the project (and optionally the meeting). History is included in subsequent turns in the same session so the agent can refer back to earlier messages.

History is not currently displayed between app sessions — this will be added in v0.2.

---

## Privacy

- Your conversation messages are sent to the AI provider you configured. Review that provider's privacy policy.
- No data is sent to Anthropic or any other third party by Meridian itself.
- API keys are stored in the OS keychain, not in the database or any log file.
