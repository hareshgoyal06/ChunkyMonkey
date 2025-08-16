use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub document_count: u32,
    pub chunk_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub id: u32,
    pub project_id: u32,
    pub document_id: u32,
    pub file_path: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: u32,
    pub document_path: String,
    pub chunk_text: String,
    pub similarity: f32,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: u32,
    pub file_path: String,
    pub file_hash: String,
    pub size: usize,
    pub chunk_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: u32,
    pub document_id: u32,
    pub text: String,
    pub chunk_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: u32,
    pub chunk_id: u32,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGAnswer {
    pub question: String,
    pub answer: String,
    pub context: String,
    pub sources: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub project_count: u32,
    pub document_count: u32,
    pub chunk_count: u32,
    pub database_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub overlap: usize,
    pub max_file_size: usize,
    pub file_patterns: Vec<String>,
} 