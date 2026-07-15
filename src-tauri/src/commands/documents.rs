use crate::db::repositories::documents as repo;
use crate::db::repositories::jobs as jobs_repo;
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

    // Queue embedding job with normal priority
    let _ = jobs_repo::queue_embed_document_job(&conn, &doc.id, &project_id, 5);

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

    // Queue embedding job with normal priority
    let _ = jobs_repo::queue_embed_document_job(&conn, &doc.id, &project_id, 5);

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
    let doc = repo::create_document(
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
    )?;

    // Queue embedding job with normal priority
    let _ = jobs_repo::queue_embed_document_job(&conn, &doc.id, &project_id, 5);

    Ok(doc)
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

#[derive(serde::Serialize)]
pub struct OrphanedDocument {
    pub folder_id: String,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: i64,
}

#[tauri::command]
pub async fn find_orphaned_documents(
    state: State<'_, AppState>,
) -> Result<Vec<OrphanedDocument>, String> {
    let docs_dir = crate::db::connection::get_documents_dir();
    if !docs_dir.exists() {
        return Ok(vec![]);
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut orphans = Vec::new();

    for entry in std::fs::read_dir(&docs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let folder_path = entry.path();

        if !folder_path.is_dir() {
            continue;
        }

        let folder_name = folder_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for file_entry in std::fs::read_dir(&folder_path).map_err(|e| e.to_string())? {
            let file_entry = file_entry.map_err(|e| e.to_string())?;
            let file_path = file_entry.path();

            if !file_path.is_file() {
                continue;
            }

            let filename = file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let file_path_str = file_path.to_string_lossy().to_string();

            // Check if this file exists in the database
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM documents WHERE file_path = ?1 LIMIT 1",
                    rusqlite::params![file_path_str],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if !exists {
                let metadata = std::fs::metadata(&file_path).ok();
                orphans.push(OrphanedDocument {
                    folder_id: folder_name.clone(),
                    filename,
                    file_path: file_path_str,
                    file_size_bytes: metadata.map(|m| m.len() as i64).unwrap_or(0),
                });
            }
        }
    }

    Ok(orphans)
}

#[tauri::command]
pub async fn recover_orphaned_document(
    project_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    let path = std::path::Path::new(&file_path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    // Parse the file content
    let parsed = crate::utils::file_parser::parse_file(path).await?;
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Move file to the project's document folder
    let docs_dir = crate::db::connection::get_documents_dir().join(&project_id);
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;

    let dest_path = docs_dir.join(&filename);

    // Copy (not move) to preserve the original
    std::fs::copy(path, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let chunks_json = serde_json::to_string(&parsed.chunks).unwrap_or_default();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let doc = repo::create_document(
        &conn,
        &project_id,
        Some(&filename.trim_end_matches(".txt").replace("_", " ")),
        &filename,
        &dest_path.to_string_lossy(),
        &parsed.file_type,
        None,
        Some(&parsed.content_text),
        Some(&chunks_json),
        Some(parsed.file_size_bytes as i64),
    )?;

    // Queue embedding job with normal priority
    let _ = jobs_repo::queue_embed_document_job(&conn, &doc.id, &project_id, 5);

    // Clean up old orphan folder if empty
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir(parent);
    }

    Ok(doc)
}

#[derive(serde::Serialize)]
pub struct DocumentEmbeddingStatus {
    pub document_id: String,
    pub embeddings_ready: bool,
    pub embedding_model: Option<String>,
    pub job_status: Option<String>,
    pub job_error: Option<String>,
}

#[tauri::command]
pub async fn get_document_embedding_status(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<DocumentEmbeddingStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let doc = repo::get_document(&conn, &document_id)?
        .ok_or_else(|| "Document not found".to_string())?;

    let job = jobs_repo::get_embedding_job_for_document(&conn, &document_id)?;

    Ok(DocumentEmbeddingStatus {
        document_id: doc.id,
        embeddings_ready: doc.embeddings_ready,
        embedding_model: doc.embedding_model,
        job_status: job.as_ref().map(|j| j.status.clone()),
        job_error: job.and_then(|j| j.error),
    })
}

#[tauri::command]
pub async fn retry_document_embedding(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let doc = repo::get_document(&conn, &document_id)?
        .ok_or_else(|| "Document not found".to_string())?;

    // Queue a new embedding job with high priority
    jobs_repo::queue_embed_document_job(&conn, &document_id, &doc.project_id, 10)?;

    Ok(())
}

#[derive(serde::Serialize)]
pub struct EmbeddingMigrationStatus {
    pub documents_needing_embedding: usize,
    pub jobs_queued: usize,
}

#[tauri::command]
pub async fn get_embedding_migration_status(
    state: State<'_, AppState>,
) -> Result<EmbeddingMigrationStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count = repo::count_documents_needing_embedding(&conn)?;

    Ok(EmbeddingMigrationStatus {
        documents_needing_embedding: count,
        jobs_queued: 0,
    })
}

#[tauri::command]
pub async fn queue_embedding_migration(
    state: State<'_, AppState>,
) -> Result<EmbeddingMigrationStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let docs = repo::get_documents_needing_embedding(&conn)?;

    let mut jobs_queued = 0;
    for doc in &docs {
        // Check if there's already a pending job for this document
        if let Ok(Some(_)) = jobs_repo::get_embedding_job_for_document(&conn, &doc.id) {
            continue;
        }
        // Queue with low priority (migration jobs run after user-initiated ones)
        let _ = jobs_repo::queue_embed_document_job(&conn, &doc.id, &doc.project_id, 1);
        jobs_queued += 1;
    }

    Ok(EmbeddingMigrationStatus {
        documents_needing_embedding: docs.len(),
        jobs_queued,
    })
}
