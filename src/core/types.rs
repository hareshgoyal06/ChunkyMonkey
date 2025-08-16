use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: u32,
    pub document_path: String,
    pub chunk_text: String,
    pub similarity: f32,
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

/// Represents the quality of context retrieved for RAG questions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextQuality {
    /// Excellent context with high relevance and comprehensive coverage
    Excellent,
    /// Good context with relevant information and good coverage
    Good,
    /// Acceptable context with some relevant information but limited coverage
    Acceptable,
    /// Poor context with limited relevant information
    Poor,
}

impl ContextQuality {
    /// Check if the context quality is good or excellent
    pub fn is_good(&self) -> bool {
        matches!(self, ContextQuality::Good | ContextQuality::Excellent)
    }
    
    /// Check if the context quality is acceptable or better
    pub fn is_acceptable(&self) -> bool {
        matches!(self, ContextQuality::Acceptable | ContextQuality::Good | ContextQuality::Excellent)
    }
}

/// Statistics for the fortified RAG pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGPipelineStats {
    /// Whether advanced RAG is enabled
    pub config_enabled: bool,
    /// Whether quality assessment is enabled
    pub quality_assessment_enabled: bool,
    /// Whether answer validation is enabled
    pub answer_validation_enabled: bool,
    /// Whether semantic expansion is enabled
    pub semantic_expansion_enabled: bool,
    /// Whether fallback strategies are enabled
    pub fallback_strategies_enabled: bool,
    /// Number of vectors in local index
    pub local_vector_count: usize,
    /// Whether Pinecone is available
    pub pinecone_available: bool,
    /// Whether Ollama is available
    pub ollama_available: bool,
    /// Embedding dimension
    pub embedding_dimension: usize,
}

impl Default for RAGPipelineStats {
    fn default() -> Self {
        Self {
            config_enabled: false,
            quality_assessment_enabled: false,
            answer_validation_enabled: false,
            semantic_expansion_enabled: false,
            fallback_strategies_enabled: false,
            local_vector_count: 0,
            pinecone_available: false,
            ollama_available: false,
            embedding_dimension: 768,
        }
    }
} 