use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    total_tokens: u32,
}

pub struct OpenAIEmbeddings {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAIEmbeddings {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: "text-embedding-ada-002".to_string(),
        }
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI embedding failed: {}", error_text);
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        
        if let Some(data) = embedding_response.data.first() {
            Ok(data.embedding.clone())
        } else {
            anyhow::bail!("No embedding data received from OpenAI");
        }
    }

    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        // Process in batches to avoid rate limits
        for chunk in texts.chunks(100) {
            let batch_texts: Vec<String> = chunk.iter().map(|&s| s.to_string()).collect();
            
            let request = serde_json::json!({
                "input": batch_texts,
                "model": self.model
            });

            let response = self
                .client
                .post("https://api.openai.com/v1/embeddings")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                anyhow::bail!("OpenAI batch embedding failed: {}", error_text);
            }

            let embedding_response: EmbeddingResponse = response.json().await?;
            
            for data in embedding_response.data {
                embeddings.push(data.embedding);
            }
        }

        Ok(embeddings)
    }
} 