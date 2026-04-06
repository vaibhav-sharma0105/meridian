use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct ZoomMeeting {
    pub id: u64,
    pub uuid: String,
    pub topic: String,
    pub start_time: String,
    pub duration: u32,
    pub join_url: Option<String>,
}

#[derive(Deserialize)]
struct ZoomMeetingList {
    meetings: Option<Vec<ZoomMeeting>>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
struct ZoomMeetingSummary {
    summary_details: Option<ZoomSummaryDetails>,
}

#[derive(Deserialize)]
struct ZoomSummaryDetails {
    summary_overview: Option<String>,
    next_steps: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ZoomRecordingFile {
    file_type: String,
    download_url: String,
    status: String,
}

#[derive(Deserialize)]
struct ZoomRecordingList {
    recording_files: Option<Vec<ZoomRecordingFile>>,
}

// parse_code_from_http_request is defined in commands/connections.rs
// Keeping it here would be dead code; removed.

pub fn generate_pkce_pair() -> (String, String) {
    let verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    (verifier, challenge)
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let zoom_client_id =
        option_env!("ZOOM_CLIENT_ID").unwrap_or("placeholder_zoom_client_id");
    let zoom_client_secret =
        option_env!("ZOOM_CLIENT_SECRET").unwrap_or("placeholder_zoom_client_secret");

    Client::new()
        .post("https://zoom.us/oauth/token")
        .basic_auth(zoom_client_id, Some(zoom_client_secret))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("Refresh parse failed: {}", e))
}

pub async fn list_past_meetings(
    access_token: &str,
    since: &str,
) -> Result<Vec<ZoomMeeting>, String> {
    let client = Client::new();
    let mut all_meetings = Vec::new();
    let mut next_page_token: Option<String> = None;

    let mut page = 0u32;
    loop {
        page += 1;
        // Hard cap: max 20 pages (1000 meetings) to prevent unbounded loops
        if page > 20 {
            eprintln!("Zoom sync: reached page limit (20), stopping pagination");
            break;
        }

        let mut url = format!(
            "https://api.zoom.us/v2/users/me/meetings?type=previous_meetings&from={}&page_size=50",
            since
        );
        if let Some(ref token) = next_page_token {
            url.push_str(&format!("&next_page_token={}", token));
        }

        let resp: ZoomMeetingList = client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("List meetings failed: {}", e))?
            .json()
            .await
            .map_err(|e| format!("List meetings parse failed: {}", e))?;

        if let Some(meetings) = resp.meetings {
            all_meetings.extend(meetings);
        }

        match resp.next_page_token {
            Some(t) if !t.is_empty() => next_page_token = Some(t),
            _ => break,
        }
    }

    Ok(all_meetings)
}

pub async fn get_meeting_summary(
    access_token: &str,
    meeting_id: u64,
) -> Result<Option<String>, String> {
    let client = Client::new();
    let resp = client
        .get(format!(
            "https://api.zoom.us/v2/meetings/{}/meeting_summary",
            meeting_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Summary request failed: {}", e))?;

    let status = resp.status();
    if status.as_u16() == 404 || status.as_u16() == 403 {
        return Ok(None);
    }
    if status.as_u16() == 429 {
        eprintln!("Zoom rate limit hit for meeting {}", meeting_id);
        return Ok(None);
    }
    if !status.is_success() {
        return Ok(None);
    }

    let text = resp.text().await.map_err(|e| e.to_string())?;
    let summary: ZoomMeetingSummary = match serde_json::from_str(&text) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    Ok(summary.summary_details.map(|d| {
        let mut out = String::new();
        if let Some(overview) = d.summary_overview {
            out.push_str(&overview);
        }
        if let Some(steps) = d.next_steps {
            if !steps.is_empty() {
                out.push_str("\n\nNext Steps:\n");
                for step in steps {
                    out.push_str(&format!("- {}\n", step));
                }
            }
        }
        out
    }))
}

pub async fn get_meeting_transcript(
    access_token: &str,
    meeting_id: u64,
) -> Result<Option<String>, String> {
    let client = Client::new();
    let resp = client
        .get(format!(
            "https://api.zoom.us/v2/meetings/{}/recordings",
            meeting_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Recordings request failed: {}", e))?;

    let status = resp.status();
    if status.as_u16() == 404 || status.as_u16() == 403 {
        return Ok(None);
    }
    if !status.is_success() {
        return Ok(None);
    }

    let recordings: ZoomRecordingList = match resp.json().await {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let transcript_file = recordings
        .recording_files
        .unwrap_or_default()
        .into_iter()
        .find(|f| f.file_type == "TRANSCRIPT" && f.status == "completed");

    match transcript_file {
        None => Ok(None),
        Some(file) => {
            // Use Bearer header — never put tokens in URL query params
            let content = client
                .get(&file.download_url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| format!("Transcript download failed: {}", e))?
                .text()
                .await
                .map_err(|e| format!("Transcript read failed: {}", e))?;
            Ok(Some(content))
        }
    }
}
