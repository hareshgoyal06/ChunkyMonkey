use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub db_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: i64,
    pub chunk_index: usize,
    pub text: String,
    pub score: f32,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub file_path: PathBuf,
    pub file_hash: String,
    pub size: usize,
    pub chunk_count: usize,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: i64,
    pub document_id: i64,
    pub text: String,
    pub chunk_index: usize,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: i64,
    pub chunk_id: i64,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGAnswer {
    pub text: String,
    pub sources: Vec<SearchResult>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub overlap: usize,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentStatus {
    Added,
    Skipped,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 800,
            overlap: 150,
            file_patterns: vec![
                "*.rs".to_string(),
                "*.md".to_string(),
                "*.txt".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
            ],
            exclude_patterns: vec![
                "*.git*".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
        }
    }
} 