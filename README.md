# Meridian

> Every minute before and after a meeting should be worth more than the meeting itself.

Meridian is a **local-first, AI-powered meeting intelligence desktop application**. It turns any meeting transcript into structured tasks, project context, and AI-powered outputs — in seconds. All your data stays on your machine.

---

## Table of Contents

1. [Features](#features)
2. [Prerequisites](#prerequisites)
3. [Installation — macOS](#installation--macos)
4. [Installation — Windows](#installation--windows)
5. [First Run & Onboarding](#first-run--onboarding)
6. [Configuring Your AI Provider](#configuring-your-ai-provider)
7. [Using Meridian](#using-meridian)
8. [Connector: Zoom](#connector-zoom)
9. [Connector: Google Sheets Relay](#connector-google-sheets-relay)
10. [Moving to a New Device](#moving-to-a-new-device)
11. [Keyboard Shortcuts](#keyboard-shortcuts)
12. [Data & Privacy](#data--privacy)
13. [Troubleshooting](#troubleshooting)

---

## Features

- **Transcript Ingestion** — Paste any transcript (Zoom, Google Meet, Teams, or manual notes) and extract structured tasks with AI in seconds
- **Smart Task Management** — List, Kanban, and Table views with inline editing, priority indicators, bulk actions, and filters (status, priority, assignee, meeting, created date)
- **Confidence Scoring** — Know which assignees and due dates were explicitly stated vs. inferred from context
- **Zoom Connector** — Automatically sync past meetings, AI companion summaries, and transcripts directly from Zoom
- **Google Sheets Relay** — Pull meeting summaries from Gmail via a Google Apps Script bridge (no Gmail OAuth required)
- **Document Intelligence** — Upload PDFs, DOCX, CSV, PPTX files or URLs; query them with AI via full-text search and semantic embeddings
- **AI Chat Panel** — Contextual AI chat with 5 output templates: Leadership Update, Jira Ticket, Next Agenda, Status Report, Free-form
- **Meeting Health Score** — 0–100 score measuring agenda clarity, task distribution, follow-through, and efficiency
- **Analytics Dashboard** — Velocity charts, assignee workload heatmap, follow-through timeline per project
- **Multi-language** — English, Hindi, Gujarati
- **Export** — JSON, CSV, Markdown formats
- **Auto-backup** — Database is backed up automatically before every schema migration

---

## Prerequisites

### Both platforms

| Tool | Minimum version | How to check |
|---|---|---|
| Rust (stable toolchain) | 1.77+ | `rustc --version` |
| Node.js | 18.x or 20.x LTS | `node --version` |
| npm | 9+ (ships with Node) | `npm --version` |

### macOS only

| Tool | Notes |
|---|---|
| Xcode Command Line Tools | Required by Rust. Install with: `xcode-select --install` |
| macOS | 11 (Big Sur) or later recommended |

### Windows only

| Tool | Notes |
|---|---|
| Visual Studio C++ Build Tools | Required by Rust. See Windows installation section below. |
| WebView2 Runtime | Usually pre-installed on Windows 10/11. See below if missing. |

---

## Installation — macOS

Follow every step in order. Do not skip steps.

### Step 1 — Install Xcode Command Line Tools

Open **Terminal** and run:

```bash
xcode-select --install
```

A dialog will appear asking you to install. Click **Install** and wait for it to complete (5–10 minutes). If it says "already installed", proceed to Step 2.

To verify:
```bash
xcode-select -p
# Expected output: /Library/Developer/CommandLineTools
```

### Step 2 — Install Homebrew (if not already installed)

Homebrew is a package manager for macOS. Check if it's installed:

```bash
brew --version
```

If the command is not found, install Homebrew:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the on-screen instructions. On Apple Silicon (M1/M2/M3), Homebrew installs to `/opt/homebrew/`. The installer will prompt you to add it to your PATH — follow those instructions.

### Step 3 — Install Node.js

Install Node.js 20 LTS via Homebrew:

```bash
brew install node@20
```

Verify:
```bash
node --version   # Should show v20.x.x
npm --version    # Should show 9.x.x or higher
```

> **Note:** If you already have Node via `nvm`, `fnm`, or another manager, ensure it's version 18 or 20. Run `node --version` to confirm.

### Step 4 — Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, choose option `1` (default installation). After it completes:

```bash
source "$HOME/.cargo/env"
rustc --version   # Should show rustc 1.77.x or higher
```

Close and reopen your terminal for the PATH changes to take effect permanently.

### Step 5 — Clone the repository

```bash
git clone https://github.com/yourusername/meridian.git
cd meridian
```

### Step 6 — Install JavaScript dependencies

```bash
npm install
```

This installs all frontend dependencies listed in `package.json`. It typically takes 1–2 minutes.

### Step 7 — Run Meridian

```bash
npm run dev
```

The first run compiles the Rust backend, which takes **3–5 minutes** on a fresh machine. Subsequent starts take about 10–15 seconds.

You will see output like:
```
VITE v5.x.x ready in 1368 ms
  ➜  Local: http://localhost:1420/
Running DevCommand (cargo run ...)
Finished `dev` profile in 180s
Running `target/debug/meridian`
```

The Meridian window will open automatically.

---

## Installation — Windows

Follow every step in order. Do not skip steps.

### Step 1 — Install Visual Studio C++ Build Tools

Rust requires C++ build tools on Windows. Visit:

```
https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

1. Click **Download Build Tools**
2. Run the installer (`vs_BuildTools.exe`)
3. In the installer, select **"Desktop development with C++"**
4. Ensure these components are checked (they should be by default):
   - MSVC v143 (or latest) C++ build tools
   - Windows 10/11 SDK
   - C++ CMake tools for Windows
5. Click **Install** — this takes 10–20 minutes and requires ~5 GB

**Restart your computer** after installation completes.

### Step 2 — Verify WebView2 Runtime

Tauri uses the WebView2 runtime (Edge-based) to render the UI. On Windows 10 (version 1803+) and Windows 11, it is pre-installed.

To verify: open **Settings → Apps → Apps & features** and search for "Microsoft Edge WebView2 Runtime". If it appears, you're good.

If it's missing:
1. Visit `https://developer.microsoft.com/en-us/microsoft-edge/webview2/`
2. Download and run the **Evergreen Bootstrapper**

### Step 3 — Install Node.js

1. Visit `https://nodejs.org/`
2. Click **"LTS"** (the left button, currently v20.x)
3. Run the `.msi` installer — accept all defaults
4. **Important:** On the "Tools for Native Modules" screen, check **"Automatically install the necessary tools"** if prompted

Open a new **Command Prompt** or **PowerShell** and verify:
```
node --version    # Should show v20.x.x
npm --version     # Should show 9.x.x or higher
```

### Step 4 — Install Rust

1. Visit `https://rustup.rs/`
2. Download and run `rustup-init.exe`
3. When prompted, press `1` and then `Enter` (default installation)
4. **Restart your terminal** (or open a new Command Prompt)

Verify:
```
rustc --version    # Should show rustc 1.77.x or higher
cargo --version    # Should show cargo 1.77.x or higher
```

### Step 5 — Clone the repository

If you have Git installed:
```
git clone https://github.com/yourusername/meridian.git
cd meridian
```

If you don't have Git, download the repository as a ZIP from GitHub and extract it.

### Step 6 — Install JavaScript dependencies

Open Command Prompt or PowerShell in the `meridian` folder:
```
npm install
```

### Step 7 — Run Meridian

```
npm run dev
```

The first run compiles the Rust backend — this takes **5–10 minutes** on Windows. A progress bar appears in the terminal. The Meridian window opens when compilation is complete.

> **Windows Defender / Antivirus**: If you see a "Windows protected your PC" dialog, click **More info → Run anyway**. This is expected for unsigned development builds.

---

## First Run & Onboarding

When Meridian opens for the first time, you will see the **Onboarding Wizard**. It has four steps:

1. **Welcome** — Overview of what Meridian does
2. **AI Setup** — Configure your AI provider (see next section)
3. **First Project** — Create your first project (give it a meaningful name)
4. **First Transcript** — Paste a sample transcript to see task extraction in action

You must complete the AI Setup step to use Meridian's core features. You can skip the transcript step and come back to it later.

---

## Configuring Your AI Provider

Meridian uses AI for task extraction, health scoring, and the chat panel. It supports multiple providers.

### Option A — Direct provider (simplest)

Click **Settings (gear icon)** → **AI** tab.

| Provider | Base URL | Model example |
|---|---|---|
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` or `gpt-4o-mini` |
| Anthropic | `https://api.anthropic.com/v1` | `claude-3-5-sonnet-20241022` |
| OpenRouter | `https://openrouter.ai/api/v1` | `anthropic/claude-3.5-sonnet` |

Fill in:
- **Base URL**: from the table above
- **API Key**: your API key from the provider's dashboard
- **Model**: the model name (e.g., `gpt-4o-mini` for lower cost)

Click **Save & Test** — if the connection works, you'll see a green confirmation.

### Option B — Local AI with Ollama (no API key needed)

Ollama lets you run models locally for free. No API key required.

1. Install Ollama from `https://ollama.ai/download`
2. Pull a model:
   ```bash
   ollama pull llama3.2       # Recommended: fast, good quality
   ollama pull mistral        # Alternative
   ```
3. In Meridian Settings → AI:
   - **Base URL**: `http://localhost:11434/v1`
   - **API Key**: `ollama` (any non-empty string)
   - **Model**: `llama3.2` (must match the pulled model name)

> **Note on Ollama quality**: Local models are slower and produce less precise task extraction than GPT-4o or Claude. For best results, use a cloud provider.

### Option C — Self-hosted LiteLLM (advanced)

LiteLLM is an open-source proxy that normalizes multiple AI providers behind a single OpenAI-compatible API.

1. Install: `pip install litellm`
2. Create a config file (`litellm_config.yaml`):
   ```yaml
   model_list:
     - model_name: gpt-4o
       litellm_params:
         model: openai/gpt-4o
         api_key: sk-your-openai-key
     - model_name: claude-sonnet
       litellm_params:
         model: anthropic/claude-3-5-sonnet-20241022
         api_key: sk-ant-your-anthropic-key
   ```
3. Start the proxy:
   ```bash
   litellm --config litellm_config.yaml --port 8000
   ```
4. In Meridian Settings → AI:
   - **Base URL**: `http://localhost:8000/v1`
   - **API Key**: any non-empty string (LiteLLM handles the real keys)
   - **Model**: `gpt-4o` (must match a `model_name` in your config)

---

## Using Meridian

### Creating a Project

1. In the left sidebar, click the **+** icon next to "Projects"
2. Enter a project name and choose a color
3. Click **Create**

The project appears in the sidebar. Click it to open it.

### Ingesting a Meeting Transcript

1. Select a project in the sidebar
2. Click the **Meetings** tab
3. Click **+ New Meeting** (top right)
4. In the dialog:
   - **Meeting title**: the meeting name (e.g., "Sprint Planning April 14")
   - **Platform**: select Zoom, Google Meet, Teams, or Manual
   - **Paste transcript**: paste your meeting transcript (minimum 50 words)
   - Optional: attendees, meeting date, duration
5. Click **Ingest Meeting**

Meridian sends the transcript to your AI provider, extracts tasks, scores meeting health, and creates task items in the project — typically in 5–15 seconds.

### Managing Tasks

After ingesting a meeting, tasks appear in the **Tasks** tab.

**Views:**
- **List** — card-based view, shows title, description, assignee, due date, tags, linked meeting
- **Kanban** — drag-and-drop columns: Open → In Progress → Done
- **Table** — spreadsheet-like view for dense overview

**Filtering:**
- **Search**: keyword search across title and description
- **Status**: Open, In Progress, Done, Cancelled
- **Priority**: Critical, High, Medium, Low
- **Assignee**: filter by person name
- **By meeting**: filter tasks to a specific meeting (multi-select)
- **Created date**: Today, Last 7 days, Last 30 days, Last 3 months, Last year, or custom range

**Editing a task:**
- Click any task card to open it in the right panel
- All fields auto-save as you type (no save button needed)
- Click the task title to rename it inline

**Bulk actions:**
- Hover over a task and click the checkbox to select it
- Select multiple tasks and use the bulk action bar (mark done, delete)

### The AI Chat Panel

The right panel contains an AI chat scoped to your active project. It has access to the project's meetings and tasks as context.

**Output templates** (click the template icons):
- 📊 **2×2 Leadership Update** — impact/confidence grid for stakeholders
- 🎫 **Jira Ticket** — formatted ticket with acceptance criteria
- 📅 **Next Agenda** — follow-up meeting agenda based on open items
- 📈 **Status Report** — progress summary for a project update
- 💬 **Free-form** — open-ended chat with project context

---

## Connector: Zoom

The Zoom connector automatically syncs your past Zoom meetings, AI companion summaries, and transcripts into Meridian.

### Step 1 — Create a Zoom OAuth App

1. Go to [https://marketplace.zoom.us/develop/create](https://marketplace.zoom.us/develop/create)
2. Click **Create** next to **General App**
3. Name it anything (e.g., `Meridian`)

### Step 2 — Configure OAuth Settings

In your new app:
1. Under **App Credentials**, copy your **Client ID** and **Client Secret**
2. Under **OAuth**, set the **Redirect URI** to exactly:
   ```
   http://127.0.0.1:19274/callback
   ```
3. Set **OAuth app type** to `User-managed app`

### Step 3 — Add Required Scopes

In the **Scopes** section, add all three:
- `meeting:read:list_past_meetings`
- `meeting:read:summary`
- `cloud_recording:read:list_recording_files`

### Step 4 — Add Yourself as a Test User

Under **App Users** → add your Zoom email address. This is required for apps in development mode.

### Step 5 — Save and Activate the App

Click **Save** and confirm the app status shows **Active**.

### Step 6 — Set Credentials

Before starting Meridian, set your credentials as environment variables.

**macOS / Linux:**
```bash
export ZOOM_CLIENT_ID="paste_your_client_id_here"
export ZOOM_CLIENT_SECRET="paste_your_client_secret_here"
npm run dev
```

**Windows (Command Prompt):**
```
set ZOOM_CLIENT_ID=paste_your_client_id_here
set ZOOM_CLIENT_SECRET=paste_your_client_secret_here
npm run dev
```

**Windows (PowerShell):**
```powershell
$env:ZOOM_CLIENT_ID="paste_your_client_id_here"
$env:ZOOM_CLIENT_SECRET="paste_your_client_secret_here"
npm run dev
```

To persist credentials across sessions, add them to your shell profile (`~/.zshrc`, `~/.bash_profile`, or Windows Environment Variables in System Settings).

### Step 7 — Connect in Meridian

1. Click the **connection icon** (link icon) in the sidebar bottom strip
2. Click **Connect Zoom**
3. Your browser opens the Zoom authorization page — approve the permissions
4. After approval, Meridian shows "Connected as your@email.com"
5. Click **Sync Now** (or wait for the automatic 15-minute sync) to import meetings from the last 14 days

---

## Connector: Google Sheets Relay

The Sheets Relay connector lets Meridian receive meeting summaries from Gmail without direct Gmail OAuth access. It works through a Google Apps Script that watches your Gmail for Zoom AI summary emails and writes them to a Google Sheet. Meridian polls that sheet.

### Why this approach?

Gmail OAuth for desktop apps requires Google's app verification (a review process). The Sheets Relay bypasses this entirely — a Workspace automation writes to a Sheet, and Meridian reads that Sheet via a simple HTTPS endpoint.

### What you need

- A Google account (personal or Workspace)
- The ability to create Google Sheets and run Apps Script

---

### Part 1 — Create the Google Sheet

1. Go to [https://sheets.google.com](https://sheets.google.com) and create a new blank spreadsheet
2. Name it something like `Meridian Meeting Relay`
3. In **Row 1**, create these column headers exactly (spelling matters):

   | A | B | C | D | E | F | G |
   |---|---|---|---|---|---|---|
   | `import_id` | `created_at` | `title` | `meeting_date` | `summary` | `action_items` | `source_subject` |

4. Leave all other rows blank for now

### Part 2 — Create the Apps Script

1. In your Sheet, click **Extensions → Apps Script**
2. Delete all existing code in the editor
3. Paste the following script:

```javascript
// Meridian Sheets Relay — Apps Script
// Reads rows from this sheet and exposes them as JSON via a web endpoint.
// Supports incremental sync (since parameter) and secret key authentication.

var SHEET_NAME = "Sheet1"; // Change if your sheet tab is named differently
var SECRET_KEY_PROPERTY = "MERIDIAN_SECRET";

/**
 * Call this function ONCE to set your secret key.
 * Replace "your-secret-key-here" with any strong random string.
 * Run it from the Apps Script editor (Run → setSecretKey).
 */
function setSecretKey() {
  PropertiesService.getScriptProperties().setProperty(
    SECRET_KEY_PROPERTY,
    "your-secret-key-here"  // ← Change this before running
  );
  Logger.log("Secret key set successfully.");
}

function doGet(e) {
  var params = e.parameter;
  var secret = PropertiesService.getScriptProperties().getProperty(SECRET_KEY_PROPERTY);

  // Auth check
  if (!secret || params.key !== secret) {
    return ContentService
      .createTextOutput(JSON.stringify({ error: "unauthorized" }))
      .setMimeType(ContentService.MimeType.JSON);
  }

  // Connection test
  if (params.test === "1") {
    return ContentService
      .createTextOutput(JSON.stringify({ ok: true }))
      .setMimeType(ContentService.MimeType.JSON);
  }

  // Read rows
  var sinceMs = params.since ? parseInt(params.since, 10) : 0;
  var sheet = SpreadsheetApp.getActiveSpreadsheet().getSheetByName(SHEET_NAME);
  var data = sheet.getDataRange().getValues();
  var headers = data[0];

  var rows = [];
  for (var i = 1; i < data.length; i++) {
    var row = {};
    for (var j = 0; j < headers.length; j++) {
      row[headers[j]] = data[i][j];
    }

    // Filter by created_at if since parameter provided
    if (sinceMs > 0 && row.created_at) {
      var rowMs = new Date(row.created_at).getTime();
      if (isNaN(rowMs) || rowMs <= sinceMs) continue;
    }

    // Skip rows with no import_id
    if (!row.import_id) continue;

    rows.push(row);
  }

  return ContentService
    .createTextOutput(JSON.stringify({ ok: true, rows: rows }))
    .setMimeType(ContentService.MimeType.JSON);
}
```

4. Click **Save** (Ctrl+S / Cmd+S)
5. **Set your secret key:**
   - In the script editor, change `"your-secret-key-here"` to any strong random string (e.g., `"m3r1d14n-secret-2024"`)
   - Click the function dropdown (it shows "doGet") and change it to `setSecretKey`
   - Click **Run** (▶ button)
   - You will be asked to authorize the script — click **Review permissions → Allow**
   - The log should show: `Secret key set successfully`
   - **Important:** Remember this secret key — you will enter it in Meridian

### Part 3 — Deploy the Apps Script as a Web App

1. In the Apps Script editor, click **Deploy → New deployment**
2. Click the gear icon next to "Type" and select **Web app**
3. Configure:
   - **Description**: `Meridian Relay v1`
   - **Execute as**: `Me` (your Google account)
   - **Who has access**: `Anyone`
4. Click **Deploy**
5. A URL appears that looks like:
   ```
   https://script.google.com/macros/s/AKfycby.../exec
   ```
   Copy this URL — you will enter it in Meridian.

### Part 4 — Set Up Google Workspace Automation (optional)

To automatically populate the sheet from Gmail Zoom AI summaries, use Google Workspace's no-code automation:

1. Go to [https://workspace.google.com/products/apps-script/](https://workspace.google.com/products/apps-script/) or use Google AppSheet / Workspace Add-ons
2. Create an automation that:
   - **Trigger**: New email in Gmail matching `from:no-reply@zoom.us subject:"Meeting assets"`
   - **Action**: Append a row to your Sheet with the email's subject and body parsed into the columns

The sheet columns Meridian reads:
- `import_id` — a unique ID for deduplication (can be the email message ID)
- `created_at` — ISO-8601 timestamp of when the row was created
- `title` — meeting title
- `meeting_date` — when the meeting occurred
- `summary` — full meeting summary text
- `action_items` — extracted action items (newline-separated)
- `source_subject` — the email subject line (Meridian uses this as the canonical title)

> **Tip**: The automation can also put the full email body as a JSON object in the `import_id` cell — Meridian automatically detects and parses JSON blobs in cells.

### Part 5 — Connect in Meridian

1. Click the **connection icon** in the Meridian sidebar
2. Under **Google Sheets Relay**, click **Configure**
3. Enter:
   - **Apps Script URL**: the URL you copied in Part 3
   - **Secret Key**: the key you set in Part 2
4. Click **Test Connection**
5. If successful, you'll see "Connected! Found N rows in the sheet."
6. Click **Sync Now** to import all rows immediately

Meridian automatically syncs every 15 minutes when the app is open.

---

## Moving to a New Device

Follow these steps to move Meridian to a new computer without losing any data.

### Step 1 — Export your data (on old device)

On your old machine:

1. Open Meridian
2. Click **Settings (gear icon) → Export**
3. Choose **JSON (full export)** — this includes all projects, meetings, tasks, and documents
4. Save the file somewhere accessible (USB drive, cloud storage, etc.)

Alternatively, you can copy the raw database directly (see Step 2).

### Step 2 — Copy your database (recommended for a full migration)

The database contains everything. Its location is:

| Platform | Path |
|---|---|
| macOS / Linux | `~/.meridian/meridian.db` |
| Windows | `%APPDATA%\meridian\meridian.db` |

Copy this file to your new machine. **Do not rename it.**

Also copy `~/.meridian/backups/` if you want to keep backup history.

### Step 3 — Install Meridian on the new device

Follow the full [macOS](#installation--macos) or [Windows](#installation--windows) installation instructions above on your new device.

### Step 4 — Restore your database

Before launching Meridian for the first time on the new device, place your copied `meridian.db` file at:

| Platform | Destination |
|---|---|
| macOS / Linux | `~/.meridian/meridian.db` |
| Windows | `%APPDATA%\meridian\meridian.db` |

Create the `.meridian` folder if it doesn't exist:

**macOS/Linux:**
```bash
mkdir -p ~/.meridian
cp /path/to/your/backup/meridian.db ~/.meridian/meridian.db
```

**Windows (PowerShell):**
```powershell
New-Item -ItemType Directory -Force -Path "$env:APPDATA\meridian"
Copy-Item "C:\path\to\backup\meridian.db" "$env:APPDATA\meridian\meridian.db"
```

### Step 5 — Re-enter credentials on the new device

API keys and OAuth tokens are stored in the OS keychain — they are **not** included in the database file. You must re-enter them on the new device:

- **AI provider API key**: Settings → AI → enter your API key again
- **Zoom OAuth**: re-run the Zoom OAuth flow (Connections → Connect Zoom)
- **Sheets Relay**: Connections → Configure → enter the Apps Script URL and secret key again

The Zoom access token will expire and need re-authorization on the new device. This is expected — OAuth tokens are non-transferable for security reasons.

### Step 6 — Launch and verify

```bash
npm run dev
```

Your projects, meetings, and tasks should all be present. Check:
- All projects appear in the sidebar
- Meeting history is intact
- Tasks are visible in each project

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+K` / `⌘+K` | Open command palette |
| `Ctrl+N` / `⌘+N` | New task |
| `Ctrl+Shift+N` / `⌘+Shift+N` | New project |
| `Ctrl+M` / `⌘+M` | New meeting |
| `Ctrl+/` / `⌘+/` | Focus AI chat |
| `Ctrl+E` / `⌘+E` | Export current project |
| `Ctrl+1` / `⌘+1` | Switch to List view |
| `Ctrl+2` / `⌘+2` | Switch to Kanban view |
| `Ctrl+3` / `⌘+3` | Switch to Table view |
| `]` | Toggle right context panel |
| `[` | Toggle left sidebar |
| `Escape` | Close modal / cancel edit |

---

## Data & Privacy

- **All data is local.** Nothing is sent to any Meridian server — there is no Meridian server.
- **Database location:**
  - macOS/Linux: `~/.meridian/meridian.db`
  - Windows: `%APPDATA%\meridian\meridian.db`
- **AI inference:** Your meeting transcripts are sent to your configured AI provider (OpenAI, Anthropic, Ollama, etc.) when you ingest a meeting or use the chat panel. If you use Ollama, data never leaves your machine.
- **API keys** are stored in your OS keychain (macOS Keychain, Windows Credential Manager) — never in the database or log files.
- **Backups** are stored at `~/.meridian/backups/` and created automatically before schema migrations.

---

## Troubleshooting

### App won't start — "Zoom credentials not configured"

The binary was compiled without the `ZOOM_CLIENT_ID` environment variable. Set it before running:

```bash
export ZOOM_CLIENT_ID=your_id
export ZOOM_CLIENT_SECRET=your_secret
npm run dev
```

### Rust compilation fails on macOS — "xcrun: error"

Xcode Command Line Tools need to be reinstalled:

```bash
sudo xcode-select --reset
xcode-select --install
```

### Rust compilation fails on Windows — "LINK: fatal error"

Visual Studio Build Tools are either missing or incomplete. Re-run the installer and ensure **"Desktop development with C++"** is selected. Restart after installation.

### "Port 1420 already in use"

Another Vite process is running. Find and kill it:

```bash
# macOS/Linux:
lsof -ti:1420 | xargs kill

# Windows (PowerShell):
Get-Process -Id (Get-NetTCPConnection -LocalPort 1420).OwningProcess | Stop-Process
```

### AI extraction returns no tasks

1. Verify your AI settings: Settings → AI → Test Connection
2. Ensure the transcript is at least 50 words long
3. Check that your API key is valid and has credits
4. For Ollama: ensure the Ollama service is running (`ollama serve`) and the model is pulled (`ollama list`)

### Zoom sync shows no meetings

1. Confirm Zoom is connected: Connections shows "Connected as your@email.com"
2. Check that you have past meetings (within the last 14 days)
3. Verify your Zoom app has the 3 required scopes
4. Try disconnecting and reconnecting Zoom

### Sheets Relay "unauthorized" error

The secret key in Meridian doesn't match the one set in the Apps Script. Re-run `setSecretKey()` in the Apps Script editor and re-enter the key in Meridian Connections.

### Sheets Relay "non-JSON" error

The Apps Script deployment settings are wrong. In the Apps Script editor:
- **Deploy → Manage deployments → Edit**
- Confirm: Execute as = **Me**, Who has access = **Anyone**
- Re-deploy

### macOS — "Meridian is damaged and can't be opened"

For unsigned development builds, run:

```bash
xattr -d com.apple.quarantine /path/to/Meridian.app
```

### Windows Defender blocks the app

Right-click the `.exe` → **Properties → Unblock → Apply**. Or run from a terminal to bypass SmartScreen.

### OAuth timed out (Zoom)

The browser window didn't complete within 2 minutes. Click **Connect Zoom** again and approve the permissions promptly.

### Port 19274 already in use (Zoom OAuth callback)

```bash
# macOS/Linux:
lsof -ti:19274 | xargs kill
```

### Database appears empty after moving to new device

Verify the file is at the correct path before launching:
- macOS: `ls ~/.meridian/meridian.db` — should show a file ≥ 100KB
- Windows: check `%APPDATA%\meridian\meridian.db` exists in File Explorer

If the path is correct but data is missing, the app may have created a fresh database before you placed the file. Close Meridian, delete the empty `meridian.db`, place your backup there, and restart.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 (Rust + WebView2/WebKit) |
| Frontend | React 18 + TypeScript 5 + Vite 5 |
| Styling | Tailwind CSS v3 |
| State | Zustand + @tanstack/react-query v5 |
| Database | SQLite with FTS5 (rusqlite) |
| AI gateway | LiteLLM-compatible HTTP (OpenAI, Anthropic, Ollama, etc.) |
| Local embeddings | Ollama (optional, for semantic document search) |
| Secrets | OS keychain (keyring crate) |

## License

MIT
