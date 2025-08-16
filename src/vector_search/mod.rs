use anyhow::Result;
use std::collections::HashMap;
use crate::embeddings::cosine_similarity;

pub struct VectorIndex {
    vectors: HashMap<u32, Vec<f32>>,
    metadata: HashMap<u32, (String, String)>, // chunk_id -> (document_path, chunk_text)
    dimension: usize,
}

impl VectorIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            metadata: HashMap::new(),
            dimension,
        }
    }

    pub fn add_vector(&mut self, chunk_id: u32, vector: &[f32], document_path: &str, chunk_text: &str) -> Result<()> {
        if vector.len() != self.dimension {
            anyhow::bail!("Vector dimension mismatch: expected {}, got {}", self.dimension, vector.len());
        }
        
        // Store vector and metadata
        self.vectors.insert(chunk_id, vector.to_vec());
        self.metadata.insert(chunk_id, (document_path.to_string(), chunk_text.to_string()));
        
        Ok(())
    }

    pub fn search_similar(&self, query_vector: &[f32], k: usize) -> Result<Vec<(u32, f32, String, String)>> {
        if query_vector.len() != self.dimension {
            anyhow::bail!("Query vector dimension mismatch: expected {}, got {}", self.dimension, query_vector.len());
        }
        
        let mut results = Vec::new();
        
        // Calculate similarity with all vectors
        for (chunk_id, vector) in &self.vectors {
            if let Some((document_path, chunk_text)) = self.metadata.get(chunk_id) {
                let similarity = cosine_similarity(query_vector, vector);
                results.push((*chunk_id, similarity, document_path.clone(), chunk_text.clone()));
            }
        }
        
        // Sort by similarity (highest first) and take top k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        
        Ok(results)
    }

    pub fn get_chunk_info(&self, chunk_id: u32) -> Option<&(String, String)> {
        self.metadata.get(&chunk_id)
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    pub fn clear(&mut self) {
        self.vectors.clear();
        self.metadata.clear();
    }
}

// Enhanced RAG search with relevance scoring
pub struct RAGSearchEngine {
    vector_index: VectorIndex,
    relevance_threshold: f32,
}

impl RAGSearchEngine {
    pub fn new(dimension: usize, relevance_threshold: f32) -> Self {
        Self {
            vector_index: VectorIndex::new(dimension),
            relevance_threshold,
        }
    }

    pub fn add_chunk(&mut self, chunk_id: u32, vector: &[f32], document_path: &str, chunk_text: &str) -> Result<()> {
        self.vector_index.add_vector(chunk_id, vector, document_path, chunk_text)
    }

    pub fn search_relevant_chunks(&self, _query: &str, query_vector: &[f32], k: usize) -> Result<Vec<(u32, f32, String, String)>> {
        // Get initial vector search results
        let mut results = self.vector_index.search_similar(query_vector, k * 2)?;
        
        // Filter by relevance threshold
        results.retain(|(_, similarity, _, _)| *similarity >= self.relevance_threshold);
        
        // Take top k results
        results.truncate(k);
        
        Ok(results)
    }

    pub fn get_context_for_question(&self, question: &str, question_vector: &[f32], context_size: usize) -> Result<String> {
        let relevant_chunks = self.search_relevant_chunks(question, question_vector, context_size)?;
        
        let mut context = String::new();
        for (i, (_, similarity, document_path, chunk_text)) in relevant_chunks.iter().enumerate() {
            context.push_str(&format!("--- Chunk {} (Similarity: {:.3}) ---\n", i + 1, similarity));
            context.push_str(&format!("Source: {}\n", document_path));
            context.push_str(&format!("Content: {}\n\n", chunk_text));
        }
        
        Ok(context)
    }

    pub fn set_relevance_threshold(&mut self, threshold: f32) {
        self.relevance_threshold = threshold.max(0.0).min(1.0);
    }

    pub fn get_relevance_threshold(&self) -> f32 {
        self.relevance_threshold
    }

    pub fn clear(&mut self) {
        self.vector_index.clear();
    }
} 