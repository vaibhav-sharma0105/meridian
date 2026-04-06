# Zoom & Gmail OAuth Credentials Setup

This guide walks you through creating the OAuth credentials required to enable the Zoom and Gmail connectors in Meridian.

---

## What You Need

| Variable | Where it comes from |
|---|---|
| `ZOOM_CLIENT_ID` | Zoom Marketplace app |
| `ZOOM_CLIENT_SECRET` | Zoom Marketplace app |
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

## Part 2: Gmail OAuth App *(coming soon)*

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

## Part 3: Setting the credentials

### Option A — Shell environment (quick)

Export the variables before running `npm run tauri dev`:

```bash
export ZOOM_CLIENT_ID="your_zoom_client_id_here"
export ZOOM_CLIENT_SECRET="your_zoom_client_secret_here"
export GMAIL_CLIENT_ID="your_gmail_client_id_here"
export GMAIL_CLIENT_SECRET="your_gmail_client_secret_here"

npm run tauri dev
```

### Option B — `.env.local` file (recommended for development)

1. Create `.env.local` in the project root (already in `.gitignore`):

```bash
ZOOM_CLIENT_ID=your_zoom_client_id_here
ZOOM_CLIENT_SECRET=your_zoom_client_secret_here
GMAIL_CLIENT_ID=your_gmail_client_id_here
GMAIL_CLIENT_SECRET=your_gmail_client_secret_here
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

## Part 4: Verifying the setup

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
