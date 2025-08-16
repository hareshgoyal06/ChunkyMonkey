use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use crate::core::config::OllamaConfig;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct OllamaEmbeddings {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaEmbeddings {
    pub fn new() -> Result<Self> {
        let base_url = env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2:13b".to_string());
        
        Ok(Self {
            client: Client::new(),
            base_url,
            model,
        })
    }

    pub fn new_with_config(config: OllamaConfig) -> Result<Self> {
        let base_url = if config.base_url.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            config.base_url
        };
        
        let model = if config.model.is_empty() {
            "llama2:13b".to_string()
        } else {
            config.model
        };
        
        Ok(Self {
            client: Client::new(),
            base_url,
            model,
        })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self.client
            .post(&format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let embedding_response: EmbeddingResponse = response.json().await?;
            Ok(embedding_response.embedding)
        } else {
            anyhow::bail!("Ollama API request failed: {}", response.status())
        }
    }

    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        for text in texts {
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }
} 