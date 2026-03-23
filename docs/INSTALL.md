# Meridian — Installation Guide

## Requirements

| Platform | Minimum |
|----------|---------|
| Windows | 10 (1903) or later — WebView2 is required (bundled with Windows 11; auto-installed on Windows 10) |
| macOS | 10.15 Catalina or later |
| Linux | Ubuntu 20.04 / Debian 11 or equivalent (gtk3, webkit2gtk required) |

**Disk space:** ~100 MB installed
**RAM:** 256 MB minimum, 512 MB recommended
**Network:** Required only for AI calls (offline browsing of existing data works without network)

---

## Windows

### Option A — Installer (recommended)

1. Download `meridian_x.y.z_x64-setup.exe` from the [Releases page](https://github.com/your-org/meridian/releases).
2. Double-click the installer and follow the prompts.
3. Meridian installs to `%LOCALAPPDATA%\Programs\Meridian` by default.
4. A shortcut is created in the Start menu and (optionally) the Desktop.
5. Launch **Meridian** from the Start menu.

### Option B — Portable MSI

1. Download `meridian_x.y.z_x64.msi`.
2. Run `msiexec /i meridian_x.y.z_x64.msi`.
3. The app is registered as an installed program and appears in Add/Remove Programs.

### WebView2

Meridian requires the **Microsoft Edge WebView2 Runtime**. On Windows 11 it is pre-installed. On Windows 10 the installer automatically downloads and installs the WebView2 bootstrapper (~2 MB). If you are in an air-gapped environment, download the offline installer from Microsoft and run it before installing Meridian.

---

## macOS

1. Download `meridian_x.y.z_aarch64.dmg` (Apple Silicon) or `meridian_x.y.z_x64.dmg` (Intel).
2. Open the `.dmg` and drag **Meridian.app** to `/Applications`.
3. On first launch, macOS may display a Gatekeeper warning because the app is not yet notarized. To bypass:
   ```
   xattr -d com.apple.quarantine /Applications/Meridian.app
   ```
4. Launch Meridian from Launchpad or Spotlight.

---

## Linux

### Debian / Ubuntu — .deb

```bash
sudo dpkg -i meridian_x.y.z_amd64.deb
# Install any missing dependencies:
sudo apt-get install -f
```

### AppImage

```bash
chmod +x meridian_x.y.z_amd64.AppImage
./meridian_x.y.z_amd64.AppImage
```

### Required system libraries

```bash
# Debian / Ubuntu
sudo apt-get install libgtk-3-0 libwebkit2gtk-4.1-0 libayatana-appindicator3-1

# Fedora / RHEL
sudo dnf install gtk3 webkit2gtk4.1
```

---

## First-time Setup

1. **Launch Meridian.** The onboarding wizard opens automatically.
2. **Connect an AI provider** — enter your OpenAI, Anthropic, or other LiteLLM-compatible API key. The key is stored in the OS keychain, never on disk.
3. **Create a project** — give it a name and optional colour.
4. **Paste a transcript** — any meeting notes, AI-generated summary, or raw transcript (minimum 50 words).
5. Meridian extracts tasks and generates a meeting summary automatically.

---

## Data Location

All data is stored locally:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\com.meridian.app\` |
| macOS | `~/Library/Application Support/com.meridian.app/` |
| Linux | `~/.local/share/com.meridian.app/` |

The SQLite database is at `{data_dir}/meridian.db`. Uploaded documents are stored in `{data_dir}/documents/`.

---

## Uninstalling

### Windows
Use **Add or Remove Programs** → search "Meridian" → Uninstall.
To remove all data: delete `%APPDATA%\com.meridian.app\`.

### macOS
Drag `Meridian.app` from `/Applications` to the Trash.
To remove all data: `rm -rf ~/Library/Application\ Support/com.meridian.app/`

### Linux
```bash
sudo dpkg -r meridian        # .deb
# For AppImage: just delete the file
rm -rf ~/.local/share/com.meridian.app/
```

---

## Updating

Meridian checks for updates on launch. When an update is available a banner appears at the top of the window. Click **Install update** to download and apply it. The app restarts automatically.

For manual updates, download the new installer from the Releases page and run it over the existing installation.

---

## Troubleshooting

### The app window is blank / white
- Windows: Ensure WebView2 is installed (`winget install Microsoft.EdgeWebView2Runtime`).
- Linux: Ensure `libwebkit2gtk-4.1-0` is installed.

### AI calls fail immediately
- Check that your API key is correct and has sufficient quota.
- Verify your network can reach the AI provider endpoint.
- In Settings → AI & Models → click **Verify connection** for a live test.

### Database is locked / app won't start
- Another instance of Meridian may be running. Check the system tray.
- If the lock persists, run `meridian --reset-lock` from a terminal (or delete `{data_dir}/meridian.db-wal`).

### "Transcript too short" error
- Paste the full transcript text — minimum 50 words.
- Zoom, Google Meet, and Teams export transcripts as `.vtt` or `.txt`; paste the plain text content.
