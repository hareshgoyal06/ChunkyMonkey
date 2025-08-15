use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PineconeConfig {
    pub api_key: String,
    pub environment: String,
    pub index_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertRequest {
    pub vectors: Vec<Vector>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub vector: Vec<f32>,
    pub top_k: Option<u32>,
    pub include_metadata: Option<bool>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub matches: Vec<Match>,
    pub namespace: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub read_units: u32,
}

pub struct PineconeClient {
    client: reqwest::Client,
    pub config: PineconeConfig,
    base_url: String,
}

impl PineconeClient {
    pub fn new(config: PineconeConfig) -> Result<Self> {
        let client = reqwest::Client::new();
        let base_url = format!(
            "https://{}-{}.svc.{}.pinecone.io",
            config.index_name, "0", config.environment
        );

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    pub fn new_dummy() -> Result<Self> {
        // Create a dummy client for local-only operation
        let config = PineconeConfig {
            api_key: String::new(),
            environment: String::new(),
            index_name: String::new(),
        };
        
        Ok(Self {
            client: reqwest::Client::new(),
            config,
            base_url: String::new(),
        })
    }

    pub async fn upsert_vectors(&self, vectors: Vec<Vector>) -> Result<()> {
        let request = UpsertRequest {
            vectors,
            namespace: None,
        };

        let response = self
            .client
            .post(&format!("{}/vectors/upsert", self.base_url))
            .header("Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Pinecone upsert failed: {}", error_text);
        }

        Ok(())
    }

    pub async fn query_similar(
        &self,
        vector: Vec<f32>,
        top_k: u32,
    ) -> Result<Vec<Match>> {
        let request = QueryRequest {
            vector,
            top_k: Some(top_k),
            include_metadata: Some(true),
            namespace: None,
        };

        let response = self
            .client
            .post(&format!("{}/query", self.base_url))
            .header("Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Pinecone query failed: {}", error_text);
        }

        let query_response: QueryResponse = response.json().await?;
        Ok(query_response.matches)
    }

    pub async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let request = serde_json::json!({
            "ids": ids,
            "namespace": null
        });

        let response = self
            .client
            .post(&format!("{}/vectors/delete", self.base_url))
            .header("Api-Key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Pinecone delete failed: {}", error_text);
        }

        Ok(())
    }
} 