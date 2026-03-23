use crate::db::repositories::documents as repo;
use crate::models::document::Document;
use crate::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn upload_document(
    project_id: String,
    file_path: Option<String>,
    url: Option<String>,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    match (file_path, url) {
        (Some(path), None) => upload_file(project_id, path, state).await,
        (None, Some(url_str)) => upload_url(project_id, url_str, state).await,
        _ => Err("Provide either file_path or url, not both".to_string()),
    }
}

async fn upload_file(
    project_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    let path = std::path::Path::new(&file_path);

    let parsed = crate::utils::file_parser::parse_file(path).await?;
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Copy file to meridian documents dir
    let docs_dir = crate::db::connection::get_documents_dir().join(&project_id);
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;

    let dest_path = docs_dir.join(&filename);
    std::fs::copy(path, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let chunks_json = serde_json::to_string(&parsed.chunks).unwrap_or_default();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let doc = repo::create_document(
        &conn,
        &project_id,
        None,
        &filename,
        &dest_path.to_string_lossy(),
        &parsed.file_type,
        None,
        Some(&parsed.content_text),
        Some(&chunks_json),
        Some(parsed.file_size_bytes as i64),
    )?;

    Ok(doc)
}

async fn upload_url(
    project_id: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    // Check for duplicate URL
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        if repo::check_url_exists(&conn, &project_id, &url)? {
            return Err(format!(
                "duplicate_url:This URL is already in your documents — add anyway?"
            ));
        }
    }

    let parsed = crate::utils::file_parser::parse_url(&url).await?;

    // Use URL hostname as filename
    let filename = url
        .split('/')
        .filter(|s| !s.is_empty())
        .nth(1)
        .unwrap_or("web-page")
        .replace("?", "_")
        .chars()
        .take(50)
        .collect::<String>();
    let filename = format!("{}.url", filename);

    let chunks_json = serde_json::to_string(&parsed.chunks).unwrap_or_default();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let doc = repo::create_document(
        &conn,
        &project_id,
        None,
        &filename,
        &url,
        "url",
        Some(&url),
        Some(&parsed.content_text),
        Some(&chunks_json),
        Some(parsed.file_size_bytes as i64),
    )?;

    Ok(doc)
}

#[tauri::command]
pub async fn upload_text(
    project_id: String,
    content: String,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    let chunks = crate::utils::file_parser::chunk_text(&content, 500, 50);
    let chunks_json = serde_json::to_string(&chunks).unwrap_or_default();
    let filename = format!(
        "{}.txt",
        title.as_deref().unwrap_or("note").replace(' ', "_")
    );
    let file_size = content.len() as i64;

    // Store in documents dir
    let docs_dir = crate::db::connection::get_documents_dir().join(&project_id);
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;
    let file_path = docs_dir.join(&filename);
    std::fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write text file: {}", e))?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::create_document(
        &conn,
        &project_id,
        title.as_deref(),
        &filename,
        &file_path.to_string_lossy(),
        "text",
        None,
        Some(&content),
        Some(&chunks_json),
        Some(file_size),
    )
}

#[tauri::command]
pub async fn get_documents_for_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Document>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_documents_for_project(&conn, &project_id)
}

#[tauri::command]
pub async fn delete_document(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // Get file path to delete from filesystem
    if let Ok(Some(doc)) = repo::get_document(&conn, &id) {
        if doc.file_type != "url" {
            let _ = std::fs::remove_file(&doc.file_path);
        }
    }
    repo::delete_document(&conn, &id)
}

#[tauri::command]
pub async fn get_document_content(id: String, state: State<'_, AppState>) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let doc = repo::get_document(&conn, &id)?
        .ok_or_else(|| "Document not found".to_string())?;

    Ok(serde_json::json!({
        "content_text": doc.content_text,
        "chunks": doc.chunks,
    }))
}
