use anyhow::Result;
use std::collections::HashMap;
use crate::core::config::AppConfig;
mod ollama;

pub struct EmbeddingModel {
    dimension: usize,
    pub ollama_embeddings: Option<ollama::OllamaEmbeddings>,
}

impl EmbeddingModel {
    pub fn new() -> Result<Self> {
        // Try to load config to get the correct dimension
        let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
        
        // For now, use 768 dimensions to match Pinecone index
        // In the future, this should be configurable based on the model
        let dimension = 768;
        
        // Try to initialize Ollama embeddings (silently)
        let ollama_embeddings = match ollama::OllamaEmbeddings::new_with_config(config.ollama) {
            Ok(emb) => Some(emb),
            Err(_) => None, // Silently fail
        };
        
        Ok(Self {
            dimension,
            ollama_embeddings,
        })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // Try Ollama first if available
        if let Some(ref ollama) = self.ollama_embeddings {
            match ollama.embed_text(text).await {
                Ok(embedding) => {
                    // Ensure the embedding has the correct dimension
                    if embedding.len() == self.dimension {
                        return Ok(embedding);
                    } else {
                        // Silently fall back to simple embedding
                    }
                }
                Err(_) => {
                    // Silently fall back to simple embedding
                }
            }
        }
        
        // Fallback to simple embedding generation
        let embedding = self.generate_simple_embedding(text);
        Ok(embedding)
    }

    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Try Ollama first if available
        if let Some(ref ollama) = self.ollama_embeddings {
            let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            match ollama.embed_batch(text_refs).await {
                Ok(embeddings) => {
                    // Check if all embeddings have correct dimensions
                    let all_correct = embeddings.iter().all(|emb| emb.len() == self.dimension);
                    if all_correct {
                        return Ok(embeddings);
                    } else {
                        // Silently fall back to simple embeddings
                    }
                }
                Err(_) => {
                    // Silently fall back to simple embeddings
                }
            }
        }
        
        // Fallback to simple embedding generation
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.generate_simple_embedding(text));
        }
        Ok(embeddings)
    }

    fn generate_simple_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.dimension];
        
        // Character frequency analysis
        let mut char_counts: HashMap<char, usize> = HashMap::new();
        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }
        
        // Word-based features
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Generate embedding based on text characteristics
        for (i, ch) in text.chars().take(self.dimension / 2).enumerate() {
            if i < embedding.len() / 2 {
                let char_freq = *char_counts.get(&ch).unwrap_or(&0) as f32;
                embedding[i] = (ch as u32 as f32 * char_freq) / (text.len() as f32);
            }
        }
        
        // Word-based features in second half
        for (i, word) in words.iter().take(self.dimension / 2).enumerate() {
            let idx = self.dimension / 2 + i;
            if idx < embedding.len() {
                let word_hash = self.hash_string(word);
                embedding[idx] = (word_hash as f32) / (u64::MAX as f32);
            }
        }
        
        // Fill remaining dimensions with additional features
        for i in (self.dimension / 2 + words.len().min(self.dimension / 2))..self.dimension {
            let feature_value = (i as f32 * text.len() as f32) / (self.dimension as f32);
            embedding[i] = (feature_value.sin() + 1.0) / 2.0; // Normalize to [0,1]
        }
        
        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }

    fn hash_string(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }
}

// Vector similarity functions
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
} 