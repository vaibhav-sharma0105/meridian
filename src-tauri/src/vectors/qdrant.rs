use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder, DeletePointsBuilder,
    PointId, Filter, Condition, value::Kind, Value as QdrantValue,
};
use qdrant_client::Qdrant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_QDRANT_URL: &str = "http://localhost:6334";
const DEFAULT_VECTOR_DIMENSION: u64 = 384; // MiniLM-L6-v2 dimension

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPayload {
    pub document_id: String,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: VectorPayload,
}

pub struct QdrantClient {
    client: Arc<RwLock<Option<Qdrant>>>,
    url: String,
}

impl QdrantClient {
    pub fn new(url: Option<&str>) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            url: url.unwrap_or(DEFAULT_QDRANT_URL).to_string(),
        }
    }

    async fn get_client(&self) -> Result<Qdrant, String> {
        // Check if we have a cached client
        {
            let read_lock = self.client.read().await;
            if let Some(client) = read_lock.as_ref() {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = Qdrant::from_url(&self.url)
            .build()
            .map_err(|e| format!("Failed to create Qdrant client: {}", e))?;

        // Cache it
        {
            let mut write_lock = self.client.write().await;
            *write_lock = Some(client.clone());
        }

        Ok(client)
    }

    pub async fn is_available(&self) -> bool {
        match self.get_client().await {
            Ok(client) => client.health_check().await.is_ok(),
            Err(_) => false,
        }
    }

    pub async fn create_collection(&self, name: &str) -> Result<(), String> {
        self.create_collection_with_dimension(name, DEFAULT_VECTOR_DIMENSION).await
    }

    pub async fn create_collection_with_dimension(&self, name: &str, dimension: u64) -> Result<(), String> {
        let client = self.get_client().await?;

        // Check if collection exists
        let collections = client
            .list_collections()
            .await
            .map_err(|e| format!("Failed to list collections: {}", e))?;

        if collections.collections.iter().any(|c| c.name == name) {
            return Ok(()); // Already exists
        }

        // Create collection with vector config
        client
            .create_collection(
                CreateCollectionBuilder::new(name)
                    .vectors_config(VectorParamsBuilder::new(dimension, Distance::Cosine)),
            )
            .await
            .map_err(|e| format!("Failed to create collection '{}': {}", name, e))?;

        Ok(())
    }

    pub async fn delete_collection(&self, name: &str) -> Result<(), String> {
        let client = self.get_client().await?;

        client
            .delete_collection(name)
            .await
            .map_err(|e| format!("Failed to delete collection '{}': {}", name, e))?;

        Ok(())
    }

    pub async fn list_collections(&self) -> Result<Vec<String>, String> {
        let client = self.get_client().await?;

        let collections = client
            .list_collections()
            .await
            .map_err(|e| format!("Failed to list collections: {}", e))?;

        Ok(collections.collections.into_iter().map(|c| c.name).collect())
    }

    pub async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>, VectorPayload)>,
    ) -> Result<(), String> {
        if vectors.is_empty() {
            return Ok(());
        }

        // Determine dimension from first vector
        let dimension = vectors.first().map(|(_, v, _)| v.len() as u64).unwrap_or(DEFAULT_VECTOR_DIMENSION);

        let client = self.get_client().await?;

        // Ensure collection exists with correct dimension
        self.create_collection_with_dimension(collection, dimension).await?;

        // Convert to PointStruct
        let points: Vec<PointStruct> = vectors
            .into_iter()
            .map(|(id, vector, payload)| {
                let mut payload_map: HashMap<String, QdrantValue> = HashMap::new();
                payload_map.insert(
                    "document_id".to_string(),
                    QdrantValue {
                        kind: Some(Kind::StringValue(payload.document_id)),
                    },
                );
                payload_map.insert(
                    "chunk_index".to_string(),
                    QdrantValue {
                        kind: Some(Kind::IntegerValue(payload.chunk_index as i64)),
                    },
                );
                payload_map.insert(
                    "chunk_text".to_string(),
                    QdrantValue {
                        kind: Some(Kind::StringValue(payload.chunk_text)),
                    },
                );
                if let Some(project_id) = payload.project_id {
                    payload_map.insert(
                        "project_id".to_string(),
                        QdrantValue {
                            kind: Some(Kind::StringValue(project_id)),
                        },
                    );
                }

                PointStruct::new(id, vector, payload_map)
            })
            .collect();

        client
            .upsert_points(UpsertPointsBuilder::new(collection, points))
            .await
            .map_err(|e| format!("Failed to insert vectors: {}", e))?;

        Ok(())
    }

    pub async fn search(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        limit: u64,
        filter_project_id: Option<&str>,
    ) -> Result<Vec<SearchResult>, String> {
        let client = self.get_client().await?;

        // Build filter if project_id specified
        let filter = filter_project_id.map(|project_id| {
            Filter::must([Condition::matches("project_id", project_id.to_string())])
        });

        let mut search_builder = SearchPointsBuilder::new(collection, query_vector, limit)
            .with_payload(true);

        if let Some(f) = filter {
            search_builder = search_builder.filter(f);
        }

        let results = client
            .search_points(search_builder)
            .await
            .map_err(|e| format!("Failed to search: {}", e))?;

        let search_results: Vec<SearchResult> = results
            .result
            .into_iter()
            .filter_map(|point| {
                let payload = point.payload;
                let document_id = payload
                    .get("document_id")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| match k {
                        Kind::StringValue(s) => Some(s.clone()),
                        _ => None,
                    })?;
                let chunk_index = payload
                    .get("chunk_index")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| match k {
                        Kind::IntegerValue(i) => Some(*i as i32),
                        _ => None,
                    })
                    .unwrap_or(0);
                let chunk_text = payload
                    .get("chunk_text")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| match k {
                        Kind::StringValue(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let project_id = payload
                    .get("project_id")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| match k {
                        Kind::StringValue(s) => Some(s.clone()),
                        _ => None,
                    });

                Some(SearchResult {
                    id: match point.id? {
                        PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(s)) } => s,
                        PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(n)) } => n.to_string(),
                        _ => return None,
                    },
                    score: point.score,
                    payload: VectorPayload {
                        document_id,
                        chunk_index,
                        chunk_text,
                        project_id,
                    },
                })
            })
            .collect();

        Ok(search_results)
    }

    pub async fn delete_by_document_id(
        &self,
        collection: &str,
        document_id: &str,
    ) -> Result<(), String> {
        let client = self.get_client().await?;

        let filter = Filter::must([Condition::matches("document_id", document_id.to_string())]);

        client
            .delete_points(
                DeletePointsBuilder::new(collection).points(filter),
            )
            .await
            .map_err(|e| format!("Failed to delete vectors: {}", e))?;

        Ok(())
    }

    pub async fn count_vectors(&self, collection: &str) -> Result<u64, String> {
        let client = self.get_client().await?;

        let info = client
            .collection_info(collection)
            .await
            .map_err(|e| format!("Failed to get collection info: {}", e))?;

        Ok(info.result.map(|r| r.points_count.unwrap_or(0)).unwrap_or(0))
    }
}

impl Clone for QdrantClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
        }
    }
}

pub fn get_collection_name(project_id: Option<&str>) -> String {
    match project_id {
        Some(id) => format!("project_{}", id),
        None => "global".to_string(),
    }
}

pub fn get_collection_name_with_dimension(project_id: Option<&str>, dimension: usize) -> String {
    let base = get_collection_name(project_id);
    format!("{}_{}", base, dimension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_name() {
        assert_eq!(get_collection_name(None), "global");
        assert_eq!(get_collection_name(Some("123")), "project_123");
    }

    #[test]
    fn test_vector_payload_serialization() {
        let payload = VectorPayload {
            document_id: "doc-123".to_string(),
            chunk_index: 0,
            chunk_text: "Hello world".to_string(),
            project_id: Some("proj-1".to_string()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("doc-123"));
        assert!(json.contains("Hello world"));

        let deserialized: VectorPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.document_id, "doc-123");
        assert_eq!(deserialized.chunk_index, 0);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            id: "point-1".to_string(),
            score: 0.95,
            payload: VectorPayload {
                document_id: "doc-123".to_string(),
                chunk_index: 0,
                chunk_text: "Test".to_string(),
                project_id: None,
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("0.95"));

        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.score, 0.95);
    }

    #[test]
    fn test_qdrant_client_creation() {
        let client = QdrantClient::new(None);
        assert_eq!(client.url, "http://localhost:6334");

        let client_custom = QdrantClient::new(Some("http://custom:1234"));
        assert_eq!(client_custom.url, "http://custom:1234");
    }

    #[tokio::test]
    async fn test_is_available_returns_false_when_not_running() {
        // This test assumes Qdrant is not running on a random port
        let client = QdrantClient::new(Some("http://localhost:19999"));
        assert!(!client.is_available().await);
    }
}
