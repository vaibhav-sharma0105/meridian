# OAuth Credentials Setup

This guide walks you through creating the OAuth credentials required to enable integrations in Meridian.

---

## What You Need

| Variable | Where it comes from |
|---|---|
| `ZOOM_CLIENT_ID` | Zoom Marketplace app |
| `ZOOM_CLIENT_SECRET` | Zoom Marketplace app |
| `GITHUB_CLIENT_ID` | GitHub OAuth App |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App |
| `JIRA_CLIENT_ID` | Atlassian Developer Console |
| `JIRA_CLIENT_SECRET` | Atlassian Developer Console |
| `SLACK_CLIENT_ID` | Slack App settings |
| `SLACK_CLIENT_SECRET` | Slack App settings |
| `GMAIL_CLIENT_ID` | Google Cloud Console *(Gmail connector — coming soon)* |
| `GMAIL_CLIENT_SECRET` | Google Cloud Console *(Gmail connector — coming soon)* |

These are **app-level credentials** (they identify the Meridian app, not your personal account). OAuth tokens for your account are stored securely in the macOS Keychain at runtime — they never touch disk or logs.

---

## Part 1: Zoom OAuth App

### Step 1 — Create a Zoom app

1. Go to [https://marketplace.zoom.us/develop/create](https://marketplace.zoom.us/develop/create)
2. Click **Create** next to **General App** (not "Meeting SDK")
3. Name it `Meridian` (any name works)

### Step 2 — Configure OAuth

1. Under **App Credentials**, copy your **Client ID** and **Client Secret**
2. Under **OAuth**, add this redirect URL:
   ```
   http://127.0.0.1:19274/callback
   ```
3. Set **OAuth app type** to `User-managed app` (default)

### Step 3 — Add Scopes

In the **Scopes** section, add:
- `meeting:read:list_past_meetings` — list your past meetings
- `meeting:read:summary` — fetch AI companion summaries
- `cloud_recording:read:list_recording_files` — fetch transcript files

### Step 4 — Add yourself as a tester

Under **App Users** (or **Test**), add your Zoom account email as a tester. This is required for apps in development mode.

### Step 5 — Save & activate

Click **Save** and ensure the app shows as **Active** under your app list.

---

## Part 2: GitHub OAuth App

### Step 1 — Create a GitHub OAuth App

1. Go to [https://github.com/settings/developers](https://github.com/settings/developers)
2. Click **New OAuth App**
3. Fill in the details:
   - **Application name**: `Meridian`
   - **Homepage URL**: `http://localhost:1420`
   - **Authorization callback URL**: `http://127.0.0.1:19280/callback`
4. Click **Register application**

### Step 2 — Copy credentials

1. Copy the **Client ID** (shown on app page)
2. Click **Generate a new client secret**
3. Copy the **Client Secret** (shown only once)

### Step 3 — Required scopes

During OAuth, Meridian requests these scopes:
- `repo` — Read/write access to repositories, issues, and PRs
- `read:user` — Read user profile information

---

## Part 3: Jira OAuth App

### Step 1 — Create an Atlassian app

1. Go to [https://developer.atlassian.com/console/myapps/](https://developer.atlassian.com/console/myapps/)
2. Click **Create** → **OAuth 2.0 integration**
3. Name it `Meridian` and click **Create**

### Step 2 — Configure OAuth 2.0

1. In the left sidebar, click **Authorization**
2. Click **Add** next to **OAuth 2.0 (3LO)**
3. Set the callback URL: `http://127.0.0.1:19281/callback`
4. Click **Save changes**

### Step 3 — Add scopes

1. In the left sidebar, click **Permissions**
2. Click **Add** next to **Jira API**
3. Add these scopes:
   - `read:jira-work` — Read project and issue data
   - `write:jira-work` — Create and update issues
   - `read:jira-user` — Read user information

### Step 4 — Copy credentials

1. In the left sidebar, click **Settings**
2. Copy the **Client ID** and **Secret**

---

## Part 4: Slack App

### Step 1 — Create a Slack app

1. Go to [https://api.slack.com/apps](https://api.slack.com/apps)
2. Click **Create New App** → **From scratch**
3. Name it `Meridian` and select your workspace
4. Click **Create App**

### Step 2 — Configure OAuth

1. In the left sidebar, click **OAuth & Permissions**
2. Under **Redirect URLs**, add: `http://127.0.0.1:19282/callback`
3. Click **Add** then **Save URLs**

### Step 3 — Add scopes

Under **Scopes** → **Bot Token Scopes**, add:
- `channels:read` — View basic channel information
- `chat:write` — Send messages as the bot
- `app_mentions:read` — Receive mentions in channels

Under **Scopes** → **User Token Scopes**, add:
- `channels:read` — View channels on behalf of user

### Step 4 — Enable Socket Mode (optional, for real-time events)

> **Note**: Socket Mode requires an App-Level Token and is needed for real-time event handling. Skip if you only need channel posting.

1. In the left sidebar, click **Socket Mode**
2. Toggle **Enable Socket Mode** on
3. Click **Generate Token**, name it `meridian-socket`, select scopes `connections:write`
4. Copy the **App Token** (starts with `xapp-`)

### Step 5 — Copy credentials

1. In the left sidebar, click **Basic Information**
2. Copy the **Client ID** and **Client Secret**
3. Note the App Token if you enabled Socket Mode

---

## Part 5: Gmail OAuth App *(coming soon)*

> The Gmail connector is not yet implemented. You can skip this section for now.

### Step 1 — Create a Google Cloud project

1. Go to [https://console.cloud.google.com](https://console.cloud.google.com)
2. Click the project dropdown → **New Project**
3. Name it `Meridian`

### Step 2 — Enable Gmail API

1. Go to **APIs & Services → Enable APIs and Services**
2. Search for `Gmail API` → click **Enable**

### Step 3 — Create OAuth credentials

1. Go to **APIs & Services → Credentials**
2. Click **Create Credentials → OAuth client ID**
3. Application type: **Desktop app**
4. Name: `Meridian`
5. Add this Authorized redirect URI:
   ```
   http://127.0.0.1:19275/callback
   ```
6. Click **Create** — copy your **Client ID** and **Client Secret**

### Step 4 — Configure OAuth consent screen

1. Go to **APIs & Services → OAuth consent screen**
2. Set **User type** to **External** (unless you have a Google Workspace org)
3. Fill in the app name (`Meridian`) and your email
4. Under **Scopes**, add:
   - `https://www.googleapis.com/auth/gmail.readonly`
5. Under **Test users**, add your Gmail address

---

## Part 6: Setting the credentials

### Option A — Shell environment (quick)

Export the variables before running `npm run tauri dev`:

```bash
export ZOOM_CLIENT_ID="your_zoom_client_id_here"
export ZOOM_CLIENT_SECRET="your_zoom_client_secret_here"
export GITHUB_CLIENT_ID="your_github_client_id_here"
export GITHUB_CLIENT_SECRET="your_github_client_secret_here"
export JIRA_CLIENT_ID="your_jira_client_id_here"
export JIRA_CLIENT_SECRET="your_jira_client_secret_here"
export SLACK_CLIENT_ID="your_slack_client_id_here"
export SLACK_CLIENT_SECRET="your_slack_client_secret_here"

npm run tauri dev
```

### Option B — `.env.local` file (recommended for development)

1. Create `.env.local` in the project root (already in `.gitignore`):

```bash
ZOOM_CLIENT_ID=your_zoom_client_id_here
ZOOM_CLIENT_SECRET=your_zoom_client_secret_here
GITHUB_CLIENT_ID=your_github_client_id_here
GITHUB_CLIENT_SECRET=your_github_client_secret_here
JIRA_CLIENT_ID=your_jira_client_id_here
JIRA_CLIENT_SECRET=your_jira_client_secret_here
SLACK_CLIENT_ID=your_slack_client_id_here
SLACK_CLIENT_SECRET=your_slack_client_secret_here
```

2. Use `dotenv` to inject when running:

```bash
# Install dotenv-cli if not already present
npm install -g dotenv-cli

# Run with env vars injected
dotenv -e .env.local -- npm run tauri dev
```

### Option C — Shell profile (persistent)

Add the exports to `~/.zshrc` or `~/.bash_profile`:

```bash
echo 'export ZOOM_CLIENT_ID="your_id_here"' >> ~/.zshrc
echo 'export ZOOM_CLIENT_SECRET="your_secret_here"' >> ~/.zshrc
source ~/.zshrc
```

### Building for production

Pass the vars to `cargo build` or the Tauri build command:

```bash
ZOOM_CLIENT_ID=... ZOOM_CLIENT_SECRET=... npm run tauri build
```

---

## Part 7: Verifying the setup

1. Run `npm run tauri dev` with the env vars set
2. In Meridian, click **Connections** in the left sidebar
3. Click **Connect Zoom**
4. Your browser opens the Zoom OAuth authorization page
5. After approving, you see "Connected as your@email.com" in the Connections panel
6. On next app launch (or "Sync Now"), Meridian fetches meetings from the last 14 days

---

## Troubleshooting

### "Zoom credentials not configured" error
The `ZOOM_CLIENT_ID` env var is not set. The build used the placeholder value. Rebuild with the variable exported.

### Port 19274 already in use
Another process is using that port. Kill it:
```bash
lsof -ti:19274 | xargs kill
```

### OAuth timed out (no response in 2 minutes)
The browser didn't complete the OAuth flow in time. Try again — make sure to approve the Zoom permissions page before the timer expires.

### Token expired / "Reconnect required"
Zoom access tokens expire after 1 hour. Meridian auto-refreshes them on sync. If refresh fails (e.g., you revoked access in Zoom), click **Disconnect** in Connections, then reconnect.

### "placeholder" shown as client ID in error messages
The binary was compiled without the `ZOOM_CLIENT_ID` environment variable. Set it and rebuild.

### Scopes not approved
If Zoom shows an error about insufficient scopes, verify all 3 scopes are added to your Zoom app and that the app is saved/activated.

---

## Integration-Specific Troubleshooting

### GitHub

**"Bad credentials" error**
Your Client Secret may have been regenerated. Generate a new secret in GitHub settings and update your env vars.

**"Not Found" when syncing repos**
The `repo` scope may not have been granted. Disconnect and reconnect, ensuring you approve the repo scope.

**Rate limiting (403)**
GitHub limits API calls to 5,000/hour for authenticated apps. Wait an hour or reduce sync frequency.

### Jira

**"unauthorized_client" error**
Your app may not have OAuth 2.0 (3LO) configured. Check Authorization settings in Atlassian Developer Console.

**"Scope not found" error**
Ensure you've added Jira API permissions in the Permissions tab, not just OAuth scopes.

**Can't see Jira projects**
Your Atlassian account may not have access to the Jira site. Verify you can access Jira in a browser.

### Slack

**"invalid_client_id" error**
The Client ID is incorrect. Copy it again from Basic Information in your Slack app settings.

**"missing_scope" error**
Required scopes weren't added. Go to OAuth & Permissions and add the scopes listed above.

**Messages not appearing**
The bot may not be invited to the channel. In Slack, type `/invite @Meridian` in the channel.

**Socket Mode not connecting**
Verify the App Token starts with `xapp-` and that Socket Mode is enabled in your app settings.
