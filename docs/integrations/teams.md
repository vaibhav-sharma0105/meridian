# Microsoft Teams Integration Guide

Meridian does not have a direct Teams integration in v0.1. This guide explains how to export Teams transcripts and use them with Meridian.

---

## Exporting a Teams transcript

### Requirements
- Teams meeting recording saved to **OneDrive** or **SharePoint**
- Transcription must have been enabled before the meeting started
- You must be the meeting organiser or have access to the recording

### Steps

1. Go to **Teams → Chat → the meeting chat** (or **Calendar → the meeting**).
2. Click the recording thumbnail → open in **Microsoft Stream** (or OneDrive).
3. On the Stream video page, click the **Transcript** tab on the right.
4. Click **Download** → choose **`.docx`** or **`.vtt`** format.
5. Open the file, select all text, copy, and paste into Meridian.

### Alternative: Copy from the transcript panel
In the Stream player, you can select all text in the Transcript panel directly and copy it without downloading.

---

## Using Teams Meeting Notes / Copilot

If your organisation has **Microsoft 365 Copilot**:

1. After the meeting, Copilot generates a summary under **Recap** in the Teams meeting chat.
2. Click **Copy** on any section (Summary, Action items, etc.).
3. Paste the full Recap text into Meridian — action items already identified by Copilot will be re-extracted and linked to the correct project.

---

## Pasting into Meridian

1. Open Meridian and select or create a project.
2. Click **Meetings → Paste transcript**.
3. Set **Platform** to `Microsoft Teams`.
4. Paste the transcript or Copilot summary.
5. Click **Extract tasks**.

---

## Planned: Teams App integration (v0.2)

A planned Teams App will:
- Surface a Meridian tab inside Teams channels
- Import transcripts directly from Stream without manual export
- Push extracted tasks to Microsoft Planner or Azure DevOps (optional)
