# Gmail → Sheets Relay Setup Guide

Automatically import Zoom AI meeting summaries into Meridian — no Zoom API or Google Cloud project required.

**How it works:**
```
New Zoom summary email in Gmail
        ↓
Google Workspace AI Studio (automation)
        ↓
Private Google Sheet (your relay buffer)
        ↓
Apps Script web app (tiny secure API)
        ↓
Meridian polls every 15 min → creates meetings + tasks
```

---

## What you need

- A Google Workspace account with access to Google Sheets
- Access to [Google Workspace AI Studio](https://studio.workspace.google.com)
- Meridian installed and running on your Mac

**Time to set up:** ~15 minutes

---

## Part 1 — Create the Google Sheet

1. Go to [sheets.google.com](https://sheets.google.com) and click **+ New spreadsheet**

2. Rename the spreadsheet to something memorable, e.g. `Meridian Imports`

3. Rename the first tab (bottom of screen) from `Sheet1` to `meridian_imports`
   - Right-click the tab → **Rename**

4. In **row 1**, add these column headers (one per cell, A through G):

   | A | B | C | D | E | F | G |
   |---|---|---|---|---|---|---|
   | `import_id` | `created_at` | `title` | `meeting_date` | `summary` | `action_items` | `source_subject` |

   > **Tip:** Copy this row and paste it directly into cell A1:
   > `import_id	created_at	title	meeting_date	summary	action_items	source_subject`

5. **Keep the sheet private** (don't change sharing settings — it stays visible only to you)

---

## Part 2 — Add the Apps Script

The Apps Script acts as a tiny, secure API that lets Meridian read your private Sheet.

1. In your Sheet, click **Extensions → Apps Script**
   - A new browser tab opens with the Apps Script editor

2. **Delete all existing code** in the editor (select all, delete)

3. **Paste the following script:**

```javascript
// ── Meridian Sheets Relay ──────────────────────────────────────────
// Paste this into Extensions → Apps Script, then run setSecretKey()
// once to store your secret, then deploy as a Web App.

var SECRET_PROP = 'MERIDIAN_KEY';

function doGet(e) {
  try {
    var secret = PropertiesService.getScriptProperties()
                   .getProperty(SECRET_PROP);
    if (!secret || !e.parameter.key || e.parameter.key !== secret) {
      return out({ error: 'unauthorized' });
    }
    if (e.parameter.test === '1') {
      return out({ ok: true, message: 'Connected!' });
    }
    var since = parseFloat(e.parameter.since || '0');
    var ss    = SpreadsheetApp.getActiveSpreadsheet();
    var sheet = ss.getSheetByName('meridian_imports') || ss.getSheets()[0];
    var data  = sheet.getDataRange().getValues();
    if (data.length <= 1) return out({ rows: [] });
    var headers = data[0].map(function(h){ return String(h).trim(); });
    var rows = [];
    for (var i = 1; i < data.length; i++) {
      var row = {};
      for (var j = 0; j < headers.length; j++) {
        var val = data[i][j];
        // Convert Date objects to ISO 8601 — Google Sheets stores dates as
        // Date objects; String() produces locale formats that break parsing.
        row[headers[j]] = (val instanceof Date) ? val.toISOString() : (val !== undefined ? String(val) : '');
      }
      var ts = row['created_at'] ? new Date(row['created_at']).getTime() : NaN;
      if (isNaN(ts) || ts > since) rows.push(row);
    }
    return out({ rows: rows });
  } catch(err) {
    return out({ error: err.toString() });
  }
}

/** Run this once to set your secret key, then never again. */
function setSecretKey() {
  // Replace the value below with your secret from Meridian
  PropertiesService.getScriptProperties()
    .setProperty(SECRET_PROP, 'PASTE_YOUR_SECRET_HERE');
  Logger.log('Secret key saved.');
}

function out(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj))
    .setMimeType(ContentService.MimeType.JSON);
}
```

4. Click **Save** (the floppy disk icon or Ctrl/Cmd+S)

---

## Part 3 — Set the secret key

The secret key prevents anyone else from reading your Sheet through the API. Meridian generates one for you automatically.

1. In Meridian, open **Settings → Connections → Set up Sheets Relay**
2. You'll see a pre-generated secret key — click **Copy** next to it
3. Go back to the Apps Script editor
4. In the `setSecretKey()` function, replace `PASTE_YOUR_SECRET_HERE` with your copied key:
   ```javascript
   .setProperty(SECRET_PROP, 'abc123-your-actual-key-here');
   ```
5. In the Apps Script toolbar, make sure `setSecretKey` is selected in the function dropdown
6. Click **Run** (▶ button)
7. You'll be asked to authorise — click **Review Permissions → Allow**
8. Check the **Execution log** at the bottom — you should see `Secret key saved.`

> **Important:** The key is now stored securely in Apps Script's Properties Service, not in the code. You can delete the value from the code after this step if you want.

---

## Part 4 — Deploy as a Web App

1. In the Apps Script editor, click **Deploy → New deployment**
2. Click the gear icon ⚙ next to "Type" and select **Web app**
3. Set the fields:
   - **Description:** `Meridian Relay`
   - **Execute as:** `Me`
   - **Who has access:** `Anyone`
4. Click **Deploy**
5. Click **Authorise access** if prompted and complete the permission flow
6. **Copy the Web App URL** — it looks like:
   ```
   https://script.google.com/macros/s/AKfycby.../exec
   ```

> **Note on "Anyone" access:** The URL is a long unguessable identifier. The secret key provides authentication — without it, the API returns an `unauthorized` error. The Sheet itself remains private.

---

## Part 5 — Connect in Meridian

1. In Meridian, go to **Settings → Connections → Set up Sheets Relay**
2. Paste the Web App URL into the URL field
3. Click **Save & Test Connection**
4. You should see: *"Connected! Meridian will start pulling Zoom summaries from your Sheet on the next sync."*

If you see an error, check the [troubleshooting section](#troubleshooting) below.

---

## Part 6 — Create the Studio workflow

This is what automatically captures Zoom summary emails and writes them to your Sheet.

1. Open [Google Workspace AI Studio](https://studio.workspace.google.com)
2. Click **+ New workflow** (or equivalent button)
3. Set up the **trigger:**
   - Service: **Gmail**
   - Event: **New email received**
   - Filter: `from:zoom.us` (to only trigger on Zoom emails)

4. Add an **action:**
   - Service: **Google Sheets**
   - Action: **Append row**
   - Sheet: select your `Meridian Imports` spreadsheet
   - Tab: `meridian_imports`

5. **Map the fields** (Studio will show email variables on the left):

   | Sheet column | Value to map |
   |---|---|
   | `import_id` | Email message ID (or a unique formula like `=CONCATENATE("s-",TEXT(NOW(),"yyyyMMddHHmmssSSS"))`) |
   | `created_at` | Current timestamp (use Studio's "now" variable or date function) |
   | `title` | Email subject (Studio provides this as a variable) |
   | `meeting_date` | Email received date |
   | `summary` | Email body / plain text |
   | `action_items` | Leave blank (Meridian's AI will extract these) |
   | `source_subject` | Email subject (same as title) |

6. **Save and activate** the workflow

7. **Test it:** Send yourself a Zoom summary email (or forward an existing one) and check that a new row appears in your Sheet within a few minutes.

---

## How Meridian processes summaries

Once rows appear in your Sheet, Meridian will:

1. **Detect them** on the next 15-minute sync (or click **Sync Now** in Connections)
2. **Create a Pending Import** — visible in the pending imports panel
3. **Wait for your approval** — you assign the meeting to a project and click Import
4. **Run the AI pipeline** — extracts action items, generates a structured summary, calculates a meeting health score
5. **Create tasks** from any action items found in the summary

---

## Troubleshooting

### "Could not reach the Apps Script URL"
- Make sure you copied the full URL ending in `/exec`, not `/dev`
- Check your internet connection
- The script must be deployed as a **Web App**, not just saved

### "Secret key rejected — unauthorized"
- Confirm you ran `setSecretKey()` in the Apps Script editor after pasting your key
- Make sure the key in Meridian matches exactly (no extra spaces)
- If you regenerated the key in Meridian, you need to run `setSecretKey()` again with the new value

### "Apps Script returned non-JSON"
- The script isn't deployed yet — click **Deploy → New deployment** in the Apps Script editor
- Make sure **Execute as: Me** and **Who has access: Anyone** are set correctly

### Studio isn't writing rows to the Sheet
- Check that the Studio workflow is **active** (not paused)
- Verify the trigger is set to `from:zoom.us` (not an empty filter)
- Send yourself a test Zoom email and watch if a row appears
- Check Studio's execution history for errors

### Meridian shows no pending imports after sync
- Click **Sync Now** in Settings → Connections to force an immediate sync
- Check that there are rows in your Sheet (at least one row below the headers)
- Make sure the `created_at` column contains a valid date/time value

### I got a Zoom email but it didn't appear in the Sheet
- Zoom AI summary emails come from `no-reply@zoom.us` or `support@zoom.us` — check your Studio trigger filter
- Some Zoom plans send summaries only to the meeting host; others send to all participants
- Check Studio's execution log for failures

---

## Manual import (no automation required)

If Studio isn't available or a meeting summary wasn't captured automatically, you can always import manually:

1. Go to **Settings → Connections → Manual Import**
2. Select a project
3. Either paste the Zoom AI summary text, or click **Upload File** to import a `.txt` or `.vtt` transcript file
4. Click **Import** — Meridian processes it immediately with AI

---

## Security notes

- The Google Sheet stays **private** — only your Google account can view it
- The Apps Script URL is a long random identifier — not guessable
- The secret key provides cryptographic authentication — requests without it return nothing
- Meridian stores the secret in the **macOS Keychain**, not in plain text
- OAuth tokens are never involved — no Google Cloud project is created
