use reqwest::Client;
use serde_json::{json, Value};

pub struct OllamaClient {
    client: Client,
    pub base_url: String,
    pub model: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/embeddings", self.base_url);
        let body = json!({
            "model": self.model,
            "prompt": text,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama connection failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned error: HTTP {}", resp.status()));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let embedding = json["embedding"]
            .as_array()
            .ok_or_else(|| "Ollama response missing embedding field".to_string())?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    pub async fn check_status(&self) -> Result<(bool, Vec<String>), String> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let json: Value = resp.json().await.map_err(|e| e.to_string())?;
                let models = json["models"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok((true, models))
            }
            _ => Ok((false, vec![])),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let (_, models) = self.check_status().await?;
        Ok(models)
    }
}
