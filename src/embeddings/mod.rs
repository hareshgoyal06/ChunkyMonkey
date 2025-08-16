pub mod openai;
pub mod ollama;

use anyhow::Result;
use crate::embeddings::openai::OpenAIEmbeddings;
use crate::embeddings::ollama::OllamaEmbeddings;
use crate::core::config::OllamaConfig;

pub enum EmbeddingProvider {
    OpenAI(OpenAIEmbeddings),
    Ollama(OllamaEmbeddings),
}

pub struct EmbeddingService {
    provider: EmbeddingProvider,
}

impl EmbeddingService {
    pub fn new(openai_api_key: Option<String>) -> Result<Self> {
        let provider = if let Some(api_key) = openai_api_key {
            if api_key.is_empty() {
                // Use Ollama if no OpenAI API key provided
                EmbeddingProvider::Ollama(OllamaEmbeddings::new()?)
            } else {
                // Use OpenAI if API key is provided
                EmbeddingProvider::OpenAI(OpenAIEmbeddings::new(api_key))
            }
        } else {
            // Default to Ollama
            EmbeddingProvider::Ollama(OllamaEmbeddings::new()?)
        };
        
        Ok(Self { provider })
    }

    pub fn new_with_ollama(ollama_config: OllamaConfig) -> Result<Self> {
        let provider = EmbeddingProvider::Ollama(OllamaEmbeddings::new_with_config(ollama_config)?);
        Ok(Self { provider })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        match &self.provider {
            EmbeddingProvider::OpenAI(openai) => openai.embed_text(text).await,
            EmbeddingProvider::Ollama(ollama) => ollama.embed_text(text).await,
        }
    }

    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        match &self.provider {
            EmbeddingProvider::OpenAI(openai) => openai.embed_batch(texts).await,
            EmbeddingProvider::Ollama(ollama) => ollama.embed_batch(texts).await,
        }
    }
} 