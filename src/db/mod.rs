use anyhow::Result;
use rusqlite::{Connection, params};
use crate::core::types::*;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("tldr.db")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                file_path TEXT UNIQUE NOT NULL,
                file_hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                chunk_count INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                document_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents (id)
            );
            
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                chunk_id INTEGER NOT NULL,
                vector TEXT NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES chunks (id)
            );"
        )?;
        Ok(())
    }

    pub fn add_document_with_chunks(
        &mut self,
        file_path: &str,
        file_hash: &str,
        size: usize,
        chunks: &[Chunk],
        embeddings: &[Vec<f32>],
    ) -> Result<u32> {
        let tx = self.conn.transaction()?;
        
        // Insert document
        let document_id = tx.execute(
            "INSERT INTO documents (file_path, file_hash, size, chunk_count) VALUES (?, ?, ?, ?)",
            params![file_path, file_hash, size, chunks.len()]
        )? as u32;
        
        // Insert chunks
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = tx.execute(
                "INSERT INTO chunks (document_id, text, chunk_index) VALUES (?, ?, ?)",
                params![document_id, chunk.text, i]
            )? as u32;
            
            // Insert embedding
            let vector_json = serde_json::to_string(&embeddings[i])?;
            tx.execute(
                "INSERT INTO embeddings (chunk_id, vector) VALUES (?, ?)",
                params![chunk_id, vector_json]
            )?;
        }
        
        tx.commit()?;
        Ok(document_id)
    }

    pub fn search_similar(&self, query_embedding: &[f32], limit: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.text, d.file_path, e.vector
             FROM chunks c
             JOIN documents d ON c.document_id = d.id
             JOIN embeddings e ON c.id = e.chunk_id
             ORDER BY c.id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let chunk_id: u32 = row.get(0)?;
            let text: String = row.get(1)?;
            let file_path: String = row.get(2)?;
            let vector_json: String = row.get(3)?;
            
            let vector: Vec<f32> = serde_json::from_str(&vector_json)
                .unwrap_or_default();
            
            Ok((chunk_id, text, file_path, vector))
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            let (chunk_id, text, file_path, vector) = row?;
            let similarity = cosine_similarity(query_embedding, &vector);
            
            if similarity >= threshold {
                results.push(SearchResult {
                    chunk_id,
                    document_path: file_path,
                    chunk_text: text,
                    similarity,
                });
            }
        }
        
        // Sort by similarity and limit
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    pub fn get_document_hash(&self, file_path: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_hash FROM documents WHERE file_path = ?"
        )?;
        
        let mut rows = stmt.query_map([file_path], |row| {
            row.get(0)
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let document_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents",
            [],
            |row| row.get(0)
        )?;
        
        let chunk_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM chunks",
            [],
            |row| row.get(0)
        )?;
        
        let db_size_mb = self.get_database_size_mb()?;
        
        Ok(DatabaseStats {
            document_count,
            chunk_count,
            database_size_mb: db_size_mb,
        })
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM embeddings;
             DELETE FROM chunks;
             DELETE FROM documents;"
        )?;
        Ok(())
    }

    fn get_database_size_mb(&self) -> Result<f64> {
        if let Ok(metadata) = std::fs::metadata("tldr.db") {
            Ok(metadata.len() as f64 / (1024.0 * 1024.0))
        } else {
            Ok(0.0)
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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