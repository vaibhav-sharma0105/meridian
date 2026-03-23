use crate::models::document::Document;
use rusqlite::{params, Connection};
use uuid::Uuid;

const DOC_COLS: &str = "id, project_id, title, filename, file_path, file_type, source_url,
    content_text, chunks, embedding_model, embeddings_ready, file_size_bytes, uploaded_at";

fn row_to_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<Document> {
    let embeddings_ready: i64 = row.get(10)?;
    let uploaded_at: String = row.get(12)?;
    Ok(Document {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        filename: row.get(3)?,
        file_path: row.get(4)?,
        file_type: row.get(5)?,
        source_url: row.get(6)?,
        content_text: row.get(7)?,
        chunks: row.get(8)?,
        embedding_model: row.get(9)?,
        embeddings_ready: embeddings_ready != 0,
        file_size_bytes: row.get(11)?,
        uploaded_at: uploaded_at.clone(),
        created_at: uploaded_at,
    })
}

pub fn create_document(
    conn: &Connection,
    project_id: &str,
    title: Option<&str>,
    filename: &str,
    file_path: &str,
    file_type: &str,
    source_url: Option<&str>,
    content_text: Option<&str>,
    chunks_json: Option<&str>,
    file_size_bytes: Option<i64>,
) -> Result<Document, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO documents (id, project_id, title, filename, file_path, file_type, source_url, content_text, chunks, file_size_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, project_id, title, filename, file_path, file_type, source_url, content_text, chunks_json, file_size_bytes],
    )
    .map_err(|e| e.to_string())?;

    get_document(conn, &id)?.ok_or_else(|| "Failed to retrieve created document".to_string())
}

pub fn get_documents_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Document>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM documents WHERE project_id = ?1 ORDER BY uploaded_at DESC",
            DOC_COLS
        ))
        .map_err(|e| e.to_string())?;

    let docs = stmt
        .query_map(params![project_id], row_to_document)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(docs)
}

pub fn get_document(conn: &Connection, id: &str) -> Result<Option<Document>, String> {
    let result = conn.query_row(
        &format!("SELECT {} FROM documents WHERE id = ?1", DOC_COLS),
        params![id],
        row_to_document,
    );

    match result {
        Ok(d) => Ok(Some(d)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn update_embeddings_ready(
    conn: &Connection,
    id: &str,
    ready: bool,
    model: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE documents SET embeddings_ready = ?1, embedding_model = ?2 WHERE id = ?3",
        params![ready as i64, model, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_document(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM document_embeddings WHERE document_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_url_exists(conn: &Connection, project_id: &str, url: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE project_id = ?1 AND source_url = ?2",
            params![project_id, url],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}
