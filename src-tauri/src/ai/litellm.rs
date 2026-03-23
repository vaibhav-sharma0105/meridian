use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;

pub struct LiteLLMClient {
    client: Client,
    base_url: String,
    api_key: String,
    pub model: String,
}

impl LiteLLMClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<Value>,
        max_tokens: Option<u32>,
    ) -> Result<String, String> {
        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(mt) = max_tokens {
            body["max_tokens"] = json!(mt);
        }

        let resp = self
            .client
            .post(&self.chat_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    format!("Cannot reach AI server at {} — check your connection", self.base_url)
                } else {
                    format!("Network error: {}", e)
                }
            })?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;

        if !status.is_success() {
            return Err(Self::parse_error_message(status.as_u16(), &text));
        }

        let json: Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse AI response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "AI response missing content field".to_string())?
            .to_string();

        Ok(content)
    }

    pub async fn verify_connection(&self) -> Result<(bool, u64), String> {
        let start = Instant::now();
        let messages = vec![json!({"role": "user", "content": "Say OK"})];

        match self.chat_completion(messages, Some(5)).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                Ok((true, latency))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_models(&self) -> Result<Vec<Value>, String> {
        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Failed to fetch models: HTTP {}", resp.status()));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse models response: {}", e))?;

        let models = json["data"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        Ok(models)
    }

    fn parse_error_message(status: u16, body: &str) -> String {
        match status {
            401 => "Invalid API key — check your credentials in Settings".to_string(),
            403 => "Access forbidden — your API key may not have permission for this model".to_string(),
            404 => "Model not found — verify the model ID in Settings".to_string(),
            429 => "Rate limited — too many requests, try again in a moment".to_string(),
            500..=599 => "AI server error — the provider is experiencing issues".to_string(),
            _ => {
                // Try to extract error message from body
                if let Ok(json) = serde_json::from_str::<Value>(body) {
                    if let Some(msg) = json["error"]["message"].as_str() {
                        return msg.to_string();
                    }
                }
                format!("AI request failed with status {}", status)
            }
        }
    }
}
