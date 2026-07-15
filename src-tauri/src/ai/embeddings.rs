use ort::session::Session;
use ort::value::{DynTensorValueType, Tensor};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tokenizers::Tokenizer;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}

static BUNDLED_EMBEDDER: OnceLock<Result<BundledEmbedder, String>> = OnceLock::new();

pub struct BundledEmbedder {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

impl BundledEmbedder {
    pub fn new(model_dir: PathBuf) -> Result<Self, String> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            return Err(format!("ONNX model not found at {:?}", model_path));
        }
        if !tokenizer_path.exists() {
            return Err(format!("Tokenizer not found at {:?}", tokenizer_path));
        }

        let session = Session::builder()
            .map_err(|e| format!("Failed to create ONNX session builder: {}", e))?
            .with_intra_threads(4)
            .map_err(|e| format!("Failed to set thread count: {}", e))?
            .commit_from_file(&model_path)
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
        })
    }

    pub fn get_or_init(model_dir: PathBuf) -> Result<&'static BundledEmbedder, String> {
        let result = BUNDLED_EMBEDDER.get_or_init(|| BundledEmbedder::new(model_dir));
        match result {
            Ok(embedder) => Ok(embedder),
            Err(e) => Err(e.clone()),
        }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> = encoding
            .get_type_ids()
            .iter()
            .map(|&t| t as i64)
            .collect();

        let seq_len = input_ids.len();

        let input_ids_tensor = Tensor::from_array(([1usize, seq_len], input_ids.into_boxed_slice()))
            .map_err(|e| format!("Failed to create input_ids tensor: {}", e))?;
        let attention_mask_tensor = Tensor::from_array(([1usize, seq_len], attention_mask.clone().into_boxed_slice()))
            .map_err(|e| format!("Failed to create attention_mask tensor: {}", e))?;
        let token_type_ids_tensor = Tensor::from_array(([1usize, seq_len], token_type_ids.into_boxed_slice()))
            .map_err(|e| format!("Failed to create token_type_ids tensor: {}", e))?;

        let mut session = self
            .session
            .lock()
            .map_err(|e| format!("Failed to lock session: {}", e))?;

        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor,
                "token_type_ids" => token_type_ids_tensor
            ])
            .map_err(|e| format!("ONNX inference failed: {}", e))?;

        let output_value = outputs
            .get("last_hidden_state")
            .or_else(|| outputs.get("token_embeddings"))
            .ok_or_else(|| "Output tensor not found".to_string())?;

        let output_tensor = output_value
            .view()
            .downcast::<DynTensorValueType>()
            .map_err(|e| format!("Failed to downcast output: {}", e))?;

        let (shape, data) = output_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract output tensor: {}", e))?;

        let shape_vec: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
        if shape_vec.len() != 3 {
            return Err(format!("Unexpected output shape: {:?}", shape_vec));
        }

        let attention_weights: Vec<f32> = attention_mask
            .iter()
            .map(|&x| x as f32)
            .collect();

        let hidden_dim = shape_vec[2];
        let num_tokens = shape_vec[1];
        let mut weighted_sum = vec![0.0f32; hidden_dim];
        let mut total_weight = 0.0f32;

        for token_idx in 0..num_tokens {
            let weight = attention_weights[token_idx];
            total_weight += weight;
            for dim_idx in 0..hidden_dim {
                let idx = token_idx * hidden_dim + dim_idx;
                weighted_sum[dim_idx] += data[idx] * weight;
            }
        }

        let mean_pooled: Vec<f32> = if total_weight > 0.0 {
            weighted_sum.iter().map(|&x| x / total_weight).collect()
        } else {
            weighted_sum
        };

        let embedding = normalize_l2(mean_pooled);
        Ok(embedding)
    }

}

impl EmbeddingProvider for BundledEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        BundledEmbedder::embed(self, text)
    }

    fn dimensions(&self) -> usize {
        384
    }

    fn provider_name(&self) -> &str {
        "bundled"
    }
}

pub struct OllamaEmbedder {
    base_url: String,
    model: String,
}

impl OllamaEmbedder {
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model }
    }
}

impl EmbeddingProvider for OllamaEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let client = crate::ai::ollama::OllamaClient::new(
            self.base_url.clone(),
            self.model.clone(),
        );
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(client.embed(text))
        })
    }

    fn dimensions(&self) -> usize {
        768 // nomic-embed-text default
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

pub fn get_embedding_provider(
    provider_type: &str,
    model_dir: Option<PathBuf>,
    ollama_base_url: Option<&str>,
    ollama_model: Option<&str>,
) -> Result<Box<dyn EmbeddingProvider>, String> {
    match provider_type {
        "bundled" => {
            let dir = model_dir.ok_or("Model directory required for bundled provider")?;
            let embedder = BundledEmbedder::get_or_init(dir)?;
            Ok(Box::new(BundledEmbedderWrapper(embedder)))
        }
        "ollama" => {
            let url = ollama_base_url.unwrap_or("http://localhost:11434");
            let model = ollama_model.unwrap_or("nomic-embed-text");
            Ok(Box::new(OllamaEmbedder::new(url.to_string(), model.to_string())))
        }
        _ => Err(format!("Unknown embedding provider: {}", provider_type)),
    }
}

struct BundledEmbedderWrapper(&'static BundledEmbedder);

impl EmbeddingProvider for BundledEmbedderWrapper {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        self.0.embed(text)
    }

    fn dimensions(&self) -> usize {
        self.0.dimensions()
    }

    fn provider_name(&self) -> &str {
        self.0.provider_name()
    }
}

unsafe impl Send for BundledEmbedderWrapper {}
unsafe impl Sync for BundledEmbedderWrapper {}

fn normalize_l2(vec: Vec<f32>) -> Vec<f32> {
    let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        vec.iter().map(|x| x / magnitude).collect()
    } else {
        vec
    }
}

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
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
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

    conn.execute(
        "UPDATE documents SET embeddings_ready = 1, embedding_model = ?1 WHERE id = ?2",
        rusqlite::params![ollama_model, document_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(embedded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_normalize_l2() {
        let vec = vec![3.0, 4.0];
        let normalized = normalize_l2(vec);
        let magnitude: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_serialize_deserialize_embedding() {
        let embedding = vec![1.0f32, 2.0, 3.0, 4.0];
        let bytes = serialize_embedding(&embedding);
        let recovered = deserialize_embedding(&bytes);
        assert_eq!(embedding, recovered);
    }
}
