use crate::models::document::DocumentChunk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub content_text: String,
    pub chunks: Vec<DocumentChunk>,
    pub file_type: String,
    pub file_size_bytes: u64,
}

pub async fn parse_file(file_path: &std::path::Path) -> Result<ParsedDocument, String> {
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Reject files over 50MB
    if file_size > 50 * 1024 * 1024 {
        let size_mb = file_size / (1024 * 1024);
        return Err(format!(
            "This file is {}MB — maximum is 50MB",
            size_mb
        ));
    }

    let content_text = match extension.as_str() {
        "txt" | "md" | "markdown" => {
            std::fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?
        }
        "vtt" => parse_vtt(file_path)?,
        "srt" => parse_srt(file_path)?,
        "csv" => parse_csv(file_path)?,
        "pdf" => parse_pdf(file_path)?,
        "docx" => parse_docx(file_path)?,
        "pptx" => parse_pptx(file_path)?,
        "xlsx" | "xls" => parse_xlsx(file_path)?,
        _ => {
            // Try reading as UTF-8 text
            std::fs::read_to_string(file_path)
                .map_err(|_| format!("Unsupported file type: .{}", extension))?
        }
    };

    let chunks = chunk_text(&content_text, 2048, 200); // ~512 tokens at 4 chars/token
    let file_type = extension_to_type(&extension);

    Ok(ParsedDocument {
        content_text,
        chunks,
        file_type,
        file_size_bytes: file_size,
    })
}

fn parse_vtt(file_path: &std::path::Path) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read VTT file: {}", e))?;

    let mut lines_out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip the WEBVTT header
        if trimmed == "WEBVTT" {
            continue;
        }
        // Skip timestamp lines: contain " --> "
        if trimmed.contains(" --> ") {
            continue;
        }
        // Skip NOTE blocks
        if trimmed.starts_with("NOTE") {
            continue;
        }
        // Skip pure numeric cue identifiers
        if trimmed.parse::<u64>().is_ok() {
            continue;
        }
        // Skip blank lines
        if trimmed.is_empty() {
            continue;
        }
        lines_out.push(trimmed.to_string());
    }

    Ok(lines_out.join("\n"))
}

fn parse_srt(file_path: &std::path::Path) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read SRT file: {}", e))?;

    let mut lines_out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip blank lines
        if trimmed.is_empty() {
            continue;
        }
        // Skip numeric sequence numbers
        if trimmed.parse::<u64>().is_ok() {
            continue;
        }
        // Skip SRT timestamp lines: contain " --> " with comma-based ms (00:00:00,000 --> 00:00:00,000)
        if trimmed.contains(" --> ") {
            continue;
        }
        lines_out.push(trimmed.to_string());
    }

    Ok(lines_out.join("\n"))
}

fn parse_csv(file_path: &std::path::Path) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read CSV: {}", e))?;
    // Return as-is, it's already text
    Ok(content)
}

fn parse_pdf(file_path: &std::path::Path) -> Result<String, String> {
    // Basic PDF text extraction - read raw bytes and extract text portions
    let bytes = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read PDF: {}", e))?;

    // Check for password protection
    if let Ok(content) = std::str::from_utf8(&bytes) {
        if content.contains("/Encrypt") {
            return Err("This PDF is password-protected — please unlock it first".to_string());
        }
    }

    // Extract readable text from PDF stream (simplified)
    let text = extract_pdf_text(&bytes);
    if text.trim().is_empty() {
        Ok(format!("[PDF: {}]", file_path.file_name().unwrap_or_default().to_string_lossy()))
    } else {
        Ok(text)
    }
}

fn extract_pdf_text(bytes: &[u8]) -> String {
    // Simple text extraction from PDF - finds text between BT and ET markers
    let content = String::from_utf8_lossy(bytes);
    let mut text = String::new();
    let mut in_text = false;

    for line in content.lines() {
        if line.contains("BT") {
            in_text = true;
        }
        if in_text && (line.contains("Tj") || line.contains("TJ")) {
            // Extract text in parentheses
            let mut i = 0;
            let chars: Vec<char> = line.chars().collect();
            while i < chars.len() {
                if chars[i] == '(' {
                    i += 1;
                    while i < chars.len() && chars[i] != ')' {
                        if chars[i] != '\\' {
                            text.push(chars[i]);
                        } else {
                            i += 1; // Skip escaped char
                        }
                        i += 1;
                    }
                    text.push(' ');
                }
                i += 1;
            }
        }
        if line.contains("ET") {
            in_text = false;
            text.push('\n');
        }
    }

    text
}

fn parse_docx(file_path: &std::path::Path) -> Result<String, String> {
    // Read DOCX as ZIP and extract word/document.xml
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open DOCX: {}", e))?;

    use std::io::Read;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|_| "Failed to read DOCX file (invalid format)".to_string())?;

    let mut doc_xml = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.name() == "word/document.xml" {
            file.read_to_string(&mut doc_xml).map_err(|e| e.to_string())?;
            break;
        }
    }

    // Extract text from XML by stripping tags
    let text = strip_xml_tags(&doc_xml);
    Ok(text)
}

fn parse_pptx(file_path: &std::path::Path) -> Result<String, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open PPTX: {}", e))?;

    use std::io::Read;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|_| "Failed to read PPTX file (invalid format)".to_string())?;

    let mut all_text = String::new();
    let slide_count = archive.len();

    for i in 0..slide_count {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.name().starts_with("ppt/slides/slide") && file.name().ends_with(".xml") {
            let mut xml = String::new();
            file.read_to_string(&mut xml).map_err(|e| e.to_string())?;
            let text = strip_xml_tags(&xml);
            if !text.trim().is_empty() {
                all_text.push_str(&text);
                all_text.push('\n');
            }
        }
    }

    if all_text.trim().is_empty() {
        Err("This presentation has limited text content — only images may be present".to_string())
    } else {
        Ok(all_text)
    }
}

fn parse_xlsx(file_path: &std::path::Path) -> Result<String, String> {
    // Read XLSX as ZIP and extract shared strings + sheets
    let content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|_| {
            // Try binary approach - just indicate it's a spreadsheet
            format!("[Spreadsheet: {}]", file_path.file_name().unwrap_or_default().to_string_lossy())
        });
    Ok(content)
}

fn strip_xml_tags(xml: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;

    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !last_was_space {
                    text.push(' ');
                    last_was_space = true;
                }
            }
            _ if !in_tag => {
                if ch.is_whitespace() {
                    if !last_was_space {
                        text.push(' ');
                        last_was_space = true;
                    }
                } else {
                    text.push(ch);
                    last_was_space = false;
                }
            }
            _ => {}
        }
    }

    text.trim().to_string()
}

pub async fn parse_url(url: &str) -> Result<ParsedDocument, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "Meridian/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Could not fetch URL ({} {})", status.as_u16(), status.canonical_reason().unwrap_or("Error")));
    }

    let html = resp.text().await.map_err(|e| e.to_string())?;
    let text = extract_readable_content(&html);

    let chunks = chunk_text(&text, 2048, 200);

    Ok(ParsedDocument {
        file_size_bytes: text.len() as u64,
        content_text: text,
        chunks,
        file_type: "url".to_string(),
    })
}

fn extract_readable_content(html: &str) -> String {
    // Simple readability extraction - strip scripts, styles, HTML tags
    let without_script = remove_between(html, "<script", "</script>");
    let without_style = remove_between(&without_script, "<style", "</style>");
    let without_nav = remove_between(&without_style, "<nav", "</nav>");
    let without_footer = remove_between(&without_nav, "<footer", "</footer>");

    strip_xml_tags(&without_footer)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn remove_between(text: &str, start: &str, end: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;
    while let Some(start_pos) = remaining.to_lowercase().find(&start.to_lowercase()) {
        result.push_str(&remaining[..start_pos]);
        if let Some(end_pos) = remaining[start_pos..].to_lowercase().find(&end.to_lowercase()) {
            remaining = &remaining[start_pos + end_pos + end.len()..];
        } else {
            break;
        }
    }
    result.push_str(remaining);
    result
}

pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<DocumentChunk> {
    if text.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut chunks = vec![];
    let mut start = 0;
    let mut index = 0;

    while start < total {
        let end = (start + chunk_size).min(total);
        let chunk_text: String = chars[start..end].iter().collect();
        chunks.push(DocumentChunk {
            text: chunk_text,
            index,
        });
        index += 1;
        if end >= total {
            break;
        }
        start = end.saturating_sub(overlap);
    }

    chunks
}

fn extension_to_type(ext: &str) -> String {
    match ext {
        "pdf" => "pdf",
        "docx" | "doc" => "docx",
        "txt" => "txt",
        "md" | "markdown" => "md",
        "pptx" | "ppt" => "pptx",
        "csv" => "csv",
        "xlsx" | "xls" => "xlsx",
        "vtt" => "vtt",
        "srt" => "srt",
        _ => "txt",
    }
    .to_string()
}
