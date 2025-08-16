use crate::core::types::{DatabaseStats, SearchResult};
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        
        // Initialize database schema
        Self::init_schema(&conn)?;
        
        // Validate schema
        Self::validate_schema(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                file_path TEXT UNIQUE NOT NULL,
                file_hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                chunk_count INTEGER NOT NULL,
                metadata TEXT DEFAULT '{}',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                document_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                metadata TEXT DEFAULT '{}',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                chunk_id INTEGER NOT NULL,
                vector TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (chunk_id) REFERENCES chunks (id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_documents_file_path ON documents (file_path);
            CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks (document_id);
            CREATE INDEX IF NOT EXISTS idx_embeddings_chunk_id ON embeddings (chunk_id);
            "#,
        )?;
        
        Ok(())
    }
    
    fn validate_schema(conn: &Connection) -> Result<()> {
        // Check if required tables exist
        let tables = vec!["documents", "chunks", "embeddings"];
        for table in tables {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                params![table],
                |row| row.get(0)
            )?;
            
            if count == 0 {
                anyhow::bail!("Required table '{}' not found in database", table);
            }
        }
        
        // Check if required columns exist in chunks table
        let columns = vec!["id", "document_id", "text", "chunk_index"];
        for column in columns {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('chunks') WHERE name=?",
                params![column],
                |row| row.get(0)
            )?;
            
            if count == 0 {
                anyhow::bail!("Required column '{}' not found in chunks table", column);
            }
        }
        
        // Check if required columns exist in embeddings table
        let columns = vec!["id", "chunk_id", "vector"];
        for column in columns {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('embeddings') WHERE name=?",
                params![column],
                |row| row.get(0)
            )?;
            
            if count == 0 {
                anyhow::bail!("Required column '{}' not found in embeddings table", column);
            }
        }
        
        Ok(())
    }
    
    pub async fn add_document_with_chunks(
        &self,
        file_path: &PathBuf,
        content: &str,
        chunks: &[String],
        embeddings: &[Vec<f32>],
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        
        // Calculate file hash
        let file_hash = self.calculate_file_hash(content);
        
        // Insert or update document
        conn.execute(
            "INSERT OR REPLACE INTO documents (file_path, file_hash, size, chunk_count, metadata) VALUES (?, ?, ?, ?, ?)",
            params![
                file_path.to_string_lossy(),
                file_hash,
                content.len(),
                chunks.len(),
                "{}"
            ],
        )?;
        
        let document_id = conn.last_insert_rowid();
        
        // Delete existing chunks for this document
        conn.execute("DELETE FROM chunks WHERE document_id = ?", params![document_id])?;
        
        // Insert chunks and embeddings
        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            conn.execute(
                "INSERT INTO chunks (document_id, text, chunk_index, metadata) VALUES (?, ?, ?, ?)",
                params![document_id, chunk, i, "{}"],
            )?;
            
            let chunk_id = conn.last_insert_rowid();
            
            // Store embedding as JSON
            let embedding_json = serde_json::to_string(embedding)?;
            conn.execute(
                "INSERT INTO embeddings (chunk_id, vector) VALUES (?, ?)",
                params![chunk_id, embedding_json],
            )?;
        }
        
        Ok(())
    }
    
    pub async fn search_similar(
        &self,
        query_vec: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().await;
        
        // Get all chunks with their embeddings
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                c.id, 
                d.file_path, 
                c.text, 
                c.chunk_index, 
                c.metadata, 
                e.vector,
                d.id as doc_id
            FROM chunks c
            JOIN documents d ON c.document_id = d.id
            JOIN embeddings e ON c.id = e.chunk_id
            ORDER BY c.id
            "#,
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;
        
        let mut results = Vec::new();
        
        for row in rows {
            let (_chunk_id, file_path, text, chunk_index, _metadata, vector_json, doc_id) = row?;
            
            // Skip very short or low-quality chunks
            if text.len() < 50 {
                continue;
            }
            
            // Skip chunks that are mostly special characters or whitespace
            let meaningful_chars = text.chars().filter(|c| c.is_alphanumeric()).count();
            if meaningful_chars < 30 {
                continue;
            }
            
            // Parse embedding vector
            let chunk_embedding: Vec<f32> = serde_json::from_str(&vector_json)?;
            
            // Calculate cosine similarity
            let similarity = self.cosine_similarity(query_vec, &chunk_embedding);
            
            if similarity >= threshold {
                results.push(SearchResult {
                    document_id: doc_id,
                    chunk_index: chunk_index as usize,
                    text,
                    score: similarity,
                    file_path,
                });
            }
        }
        
        // Sort by score and limit results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }
    
    pub async fn get_document_hash(&self, file_path: &PathBuf) -> Result<Option<String>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT file_hash FROM documents WHERE file_path = ?")?;
        let mut rows = stmt.query_map(params![file_path.to_string_lossy()], |row| row.get(0))?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().await;
        
        let documents: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        let chunks: i64 = conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
        let embeddings: i64 = conn.query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))?;
        
        // Get database file size
        let db_size_mb = if let Some(path) = conn.path() {
            if let Ok(metadata) = std::fs::metadata(path) {
                metadata.len() as f64 / (1024.0 * 1024.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        Ok(DatabaseStats {
            total_documents: documents as usize,
            total_chunks: chunks as usize,
            total_embeddings: embeddings as usize,
            db_size_mb,
        })
    }
    
    pub async fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM embeddings", [])?;
        conn.execute("DELETE FROM chunks", [])?;
        conn.execute("DELETE FROM documents", [])?;
        Ok(())
    }
    
    /// Recreate the database schema (useful for fixing schema issues)
    pub async fn recreate_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        
        // Drop existing tables
        conn.execute("DROP TABLE IF EXISTS embeddings", [])?;
        conn.execute("DROP TABLE IF EXISTS chunks", [])?;
        conn.execute("DROP TABLE IF EXISTS documents", [])?;
        
        // Recreate schema
        Self::init_schema(&conn)?;
        
        Ok(())
    }
    
    fn calculate_file_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
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
} 