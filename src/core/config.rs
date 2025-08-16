use serde::{Deserialize, Serialize};
use crate::pinecone::PineconeConfig;
use anyhow::Result;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama: OllamaConfig,
    pub pinecone: PineconeConfig,
    pub search: SearchConfig,
    pub chunking: ChunkingConfig,
    pub rag: RAGConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub llm_model: String, // LLM model for answer generation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub base_similarity_threshold: f32,
    pub fallback_threshold: f32,
    pub max_results_per_query: usize,
    pub enable_semantic_search: bool,
    pub enable_query_expansion: bool,
    pub enable_content_filtering: bool,
    pub enable_reranking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    pub max_chunk_size: usize,
    pub min_chunk_size: usize,
    pub overlap_size: usize,
    pub use_semantic_chunking: bool,
    pub respect_section_boundaries: bool,
}

/// Configuration for the fortified RAG pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    /// Enable advanced RAG with chain-of-thought reasoning (hidden from user)
    pub enable_advanced_rag: bool,
    /// Enable context quality assessment
    pub enable_quality_assessment: bool,
    /// Enable answer validation and enhancement
    pub enable_answer_validation: bool,
    /// Enable semantic expansion for better context coverage
    pub enable_semantic_expansion: bool,
    /// Enable multiple fallback strategies
    pub enable_fallback_strategies: bool,
    /// Minimum context quality threshold for advanced RAG
    pub min_quality_threshold: f32,
    /// Maximum number of context chunks to retrieve
    pub max_context_chunks: usize,
    /// Enable confidence scoring in answers
    pub enable_confidence_scoring: bool,
    /// Enable source attribution
    pub enable_source_attribution: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig {
                base_url: String::new(),
                model: "llama3".to_string(),
                llm_model: "llama3".to_string(),
            },
            pinecone: PineconeConfig {
                api_key: String::new(),
                environment: String::new(),
                index_name: String::new(),
                host: None,
            },
            search: SearchConfig {
                base_similarity_threshold: 0.5,
                fallback_threshold: 0.4,
                max_results_per_query: 10,
                enable_semantic_search: true,
                enable_query_expansion: true,
                enable_content_filtering: true,
                enable_reranking: true,
            },
            chunking: ChunkingConfig {
                max_chunk_size: 1500,
                min_chunk_size: 200,
                overlap_size: 200,
                use_semantic_chunking: true,
                respect_section_boundaries: true,
            },
            rag: RAGConfig {
                enable_advanced_rag: true,
                enable_quality_assessment: true,
                enable_answer_validation: true,
                enable_semantic_expansion: true,
                enable_fallback_strategies: true,
                min_quality_threshold: 0.6,
                max_context_chunks: 15,
                enable_confidence_scoring: true,
                enable_source_attribution: true,
            },
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let ollama_base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_default();
        let ollama_model = std::env::var("OLLAMA_MODEL").unwrap_or_default();
        let pinecone_api_key = std::env::var("PINECONE_API_KEY").unwrap_or_default();
        let pinecone_environment = std::env::var("PINECONE_ENVIRONMENT").unwrap_or_default();
        let pinecone_index = std::env::var("PINECONE_INDEX").unwrap_or_default();
        let pinecone_host = std::env::var("PINECONE_HOST").ok();
        
        Ok(Self {
            ollama: OllamaConfig {
                base_url: ollama_base_url,
                model: ollama_model,
                llm_model: "llama3".to_string(),
            },
            pinecone: PineconeConfig {
                api_key: pinecone_api_key,
                environment: pinecone_environment,
                index_name: pinecone_index,
                host: pinecone_host,
            },
            search: SearchConfig {
                base_similarity_threshold: 0.5,
                fallback_threshold: 0.4,
                max_results_per_query: 10,
                enable_semantic_search: true,
                enable_query_expansion: true,
                enable_content_filtering: true,
                enable_reranking: true,
            },
            chunking: ChunkingConfig {
                max_chunk_size: 1500,
                min_chunk_size: 200,
                overlap_size: 200,
                use_semantic_chunking: true,
                respect_section_boundaries: true,
            },
            rag: RAGConfig {
                enable_advanced_rag: true,
                enable_quality_assessment: true,
                enable_answer_validation: true,
                enable_semantic_expansion: true,
                enable_fallback_strategies: true,
                min_quality_threshold: 0.6,
                max_context_chunks: 15,
                enable_confidence_scoring: true,
                enable_source_attribution: true,
            },
        })
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn load() -> Result<Self> {
        // Try to load from config.toml first
        if let Ok(config) = Self::from_file("config.toml") {
            return Ok(config);
        }
        
        // Fallback to environment variables
        if let Ok(config) = Self::from_env() {
            return Ok(config);
        }
        
        // Final fallback to defaults
        Ok(Self::default())
    }
} 