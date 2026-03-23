pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

pub fn deserialize_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

pub async fn embed_document_chunks_background(
    conn: &rusqlite::Connection,
    document_id: &str,
    ollama_base_url: &str,
    ollama_model: &str,
) -> Result<u32, String> {
    let ollama = crate::ai::ollama::OllamaClient::new(
        ollama_base_url.to_string(),
        ollama_model.to_string(),
    );

    // Get chunks from the document
    let chunks_json: Option<String> = conn
        .query_row(
            "SELECT chunks FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let chunks_json = chunks_json.ok_or_else(|| "Document has no chunks".to_string())?;
    let chunks: Vec<crate::models::document::DocumentChunk> =
        serde_json::from_str(&chunks_json).map_err(|e| e.to_string())?;

    let mut embedded = 0u32;

    for chunk in &chunks {
        match ollama.embed(&chunk.text).await {
            Ok(embedding) => {
                let id = uuid::Uuid::new_v4().to_string();
                let embedding_bytes = serialize_embedding(&embedding);
                conn.execute(
                    "INSERT INTO document_embeddings (id, document_id, chunk_index, chunk_text, embedding, model)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        id,
                        document_id,
                        chunk.index as i64,
                        chunk.text,
                        embedding_bytes,
                        ollama_model
                    ],
                )
                .map_err(|e| e.to_string())?;
                embedded += 1;
            }
            Err(e) => {
                eprintln!("Failed to embed chunk {}: {}", chunk.index, e);
            }
        }
    }

    // Mark embeddings ready
    conn.execute(
        "UPDATE documents SET embeddings_ready = 1, embedding_model = ?1 WHERE id = ?2",
        rusqlite::params![ollama_model, document_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(embedded)
}
