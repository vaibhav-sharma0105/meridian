# Google Meet Integration Guide

Meridian does not have a direct Google Workspace OAuth integration in v0.1. This guide explains how to export Meet transcripts and meeting notes.

---

## Option A — Google Meet transcription (Workspace)

Google Meet provides automatic transcription for **Google Workspace Business Standard** and above.

1. In Meet, click the three-dot menu → **Activities → Transcripts → Start transcript**.
2. After the meeting ends, the transcript is saved to the organiser's **Google Drive** under `Meet Recordings`.
3. Open the `.docx` transcript file from Drive.
4. Select all (Ctrl+A / Cmd+A), copy, and paste into Meridian.

### Tip
The transcript includes speaker names in the format `Speaker Name: text`. Meridian uses these to assign tasks to the correct person.

---

## Option B — Gemini AI notes (Workspace Business / Enterprise)

If your organisation uses **Gemini in Google Meet**:

1. After the meeting, Gemini generates a summary with action items in Google Docs.
2. Open the auto-generated Doc from Drive.
3. Copy the full text and paste it into Meridian.

Even a Gemini-generated summary (without the raw transcript) works well — Meridian will extract any tasks mentioned.

---

## Option C — Manual notes

If transcription is not available:

1. Open **Google Docs** or any notes app during the meeting.
2. After the meeting, copy your notes and paste them into Meridian.
3. The minimum for Meridian to extract tasks is 50 words.

---

## Pasting into Meridian

1. Open Meridian and select or create a project.
2. Click **Meetings → Paste transcript**.
3. Set **Platform** to `Google Meet`.
4. Paste the text. Include any speaker labels for better task assignment.
5. Click **Extract tasks**.

---

## Planned: Google Calendar / Drive sync (v0.2)

A planned integration will:
- Surface upcoming meetings from Google Calendar
- Auto-import transcripts from Drive after a meeting ends
- Sync extracted tasks back to Google Tasks (optional)
