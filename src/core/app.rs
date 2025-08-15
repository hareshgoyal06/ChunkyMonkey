use crate::core::types::{DatabaseStats, RAGAnswer, SearchResult};
use crate::db::Database;
use crate::embeddings::EmbeddingModel;
use std::path::PathBuf;

pub struct TldrApp {
    db: Database,
    embedding_model: EmbeddingModel,
}

impl TldrApp {
    pub async fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        let db = Database::new(db_path).await?;
        let embedding_model = EmbeddingModel::new().await?;
        
        Ok(Self {
            db,
            embedding_model,
        })
    }
    
    pub async fn search(&self, query: &str, limit: usize, threshold: f32) -> anyhow::Result<Vec<SearchResult>> {
        // Get query embedding
        let query_embedding = self.embedding_model.embed_text(query).await?;
        
        // Search database
        let results = self.db.search_similar(&query_embedding, limit, threshold).await?;
        
        Ok(results)
    }
    
    pub async fn ask_question(&self, question: &str, context_chunks: usize) -> anyhow::Result<RAGAnswer> {
        // Get question embedding
        let question_embedding = self.embedding_model.embed_text(question).await?;
        
        // Find relevant chunks
        let relevant_chunks = self.db.search_similar(&question_embedding, context_chunks, 0.3).await?;
        
        if relevant_chunks.is_empty() {
            return Ok(RAGAnswer {
                text: "No relevant information found.".to_string(),
                sources: vec![],
                confidence: 0.0,
            });
        }
        
        // Generate answer (simple concatenation for now, could be enhanced with LLM)
        let context: String = relevant_chunks
            .iter()
            .map(|r| r.text.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let answer_text = format!(
            "Based on the retrieved information:\n\n{}",
            context
        );
        
        let confidence = relevant_chunks.iter().map(|r| r.similarity).sum::<f32>() / relevant_chunks.len() as f32;
        
        Ok(RAGAnswer {
            text: answer_text,
            sources: relevant_chunks,
            confidence,
        })
    }
    
    pub async fn get_stats(&self) -> anyhow::Result<DatabaseStats> {
        self.db.get_stats().await
    }
    
    pub async fn clear_database(&mut self) -> anyhow::Result<()> {
        self.db.clear_all().await
    }
    
    pub async fn add_document(&mut self, file_path: PathBuf, content: String) -> anyhow::Result<()> {
        // Check if document already exists and hasn't changed
        if let Some(existing_hash) = self.db.get_document_hash(&file_path).await? {
            let new_hash = self.calculate_file_hash(&content);
            if existing_hash == new_hash {
                return Ok(()); // Document unchanged
            }
        }
        
        // Chunk the content
        let chunks = self.chunk_text(&content, 800, 150);
        
        // Create embeddings for chunks
        let embeddings = self.embedding_model.embed_texts(&chunks).await?;
        
        // Store in database
        self.db.add_document_with_chunks(&file_path, &content, &chunks, &embeddings).await?;
        
        Ok(())
    }
    
    fn chunk_text(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        if text.is_empty() {
            return vec![];
        }
        
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            let end = (start + chunk_size).min(text.len());
            
            // Try to break at word boundary
            let mut actual_end = end;
            if end < text.len() {
                if let Some(last_space) = text[start..end].rfind(' ') {
                    if last_space > chunk_size / 2 {
                        actual_end = start + last_space;
                    }
                }
            }
            
            let chunk = text[start..actual_end].trim();
            if !chunk.is_empty() {
                chunks.push(chunk.to_string());
            }
            
            start = actual_end.saturating_sub(overlap);
            if start >= text.len() {
                break;
            }
        }
        
        chunks
    }
    
    fn calculate_file_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
} 