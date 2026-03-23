# Zoom Integration Guide

Meridian does not have a direct OAuth integration with Zoom in v0.1. This guide shows you how to export transcripts from Zoom and import them into Meridian in under 60 seconds.

---

## Exporting a Zoom transcript

### Requirements
- Zoom account with **Cloud Recording** enabled (Zoom Pro or above)
- The meeting host must have enabled **Audio transcript** in Recording settings

### Steps

1. Sign in to [zoom.us](https://zoom.us) and go to **My Account → Recordings**.
2. Find the meeting you want and click on it.
3. Click the **Audio Transcript** link (`.vtt` file) to view it, or click the download icon to save it.
4. Open the downloaded `.vtt` file in any text editor.
5. Copy all the text — the timestamps and speaker names give Meridian extra context.

### Tips

- If the transcript is long, Meridian handles up to ~100,000 words. Paste the full text.
- The speaker tags (`John Smith: ...`) help Meridian assign tasks to the right people automatically.
- Zoom also offers a **Summary** (if AI Companion is enabled). You can paste the summary alone, but a full transcript gives better task extraction.

---

## Pasting into Meridian

1. Open Meridian and select or create a project.
2. Click **Meetings → Paste transcript**.
3. Set **Platform** to `Zoom`.
4. Paste the transcript text into the large text area.
5. Click **Extract tasks** — Meridian calls your configured AI model and extracts tasks within a few seconds.

---

## Planned: Zoom App / OAuth (v0.2)

A native Zoom app integration is planned for v0.2. It will:
- List recordings from Zoom directly in Meridian
- One-click import of transcript + recording metadata
- Auto-populate attendees, duration, and meeting date

To be notified when this ships, watch the [GitHub releases page](https://github.com/your-org/meridian/releases).
