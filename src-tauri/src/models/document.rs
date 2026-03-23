use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub project_id: String,
    pub title: Option<String>,
    pub filename: String,
    pub file_path: String,
    pub file_type: String,
    pub source_url: Option<String>,
    pub content_text: Option<String>,
    pub chunks: Option<String>,
    pub embeddings_ready: bool,
    pub embedding_model: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub uploaded_at: String,
    pub created_at: String,  // alias for uploaded_at
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub text: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub document_title: String,
    pub filename: String,
    pub chunk_text: String,
    pub content: String,  // alias for chunk_text
    pub score: f64,
    pub search_type: String,
}
