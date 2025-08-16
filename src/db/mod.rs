use anyhow::Result;
use rusqlite::{Connection, params};
use crate::core::types::*;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("chunkymonkey.db")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Get a reference to the database connection
    pub fn get_connection(&self) -> &Connection {
        &self.conn
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

    pub fn add_document(&mut self, file_path: &str, file_hash: &str, size: usize) -> Result<u32> {
        let document_id = self.conn.execute(
            "INSERT INTO documents (file_path, file_hash, size, chunk_count) VALUES (?, ?, ?, 0)",
            params![file_path, file_hash, size]
        )? as u32;
        
        Ok(document_id)
    }

    pub fn get_document(&self, document_id: u32) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_hash, size, chunk_count FROM documents WHERE id = ?"
        )?;
        
        let mut rows = stmt.query_map([document_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_hash: row.get(2)?,
                size: row.get(3)?,
                chunk_count: row.get(4)?,
            })
        })?;
        
        Ok(rows.next().transpose()?)
    }

    pub fn get_documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_hash, size, chunk_count FROM documents ORDER BY id DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_hash: row.get(2)?,
                size: row.get(3)?,
                chunk_count: row.get(4)?,
            })
        })?;
        
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }
        Ok(documents)
    }

    pub fn get_document_hash(&self, file_path: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_hash FROM documents WHERE file_path = ?"
        )?;
        
        let mut rows = stmt.query_map([file_path], |row| {
            Ok(row.get(0)?)
        })?;
        
        Ok(rows.next().transpose()?)
    }

    pub fn update_document_chunk_count(&mut self, document_id: u32, chunk_count: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE documents SET chunk_count = ? WHERE id = ?",
            params![chunk_count, document_id]
        )?;
        Ok(())
    }

    pub fn add_chunk(&mut self, document_id: u32, text: &str, chunk_index: usize) -> Result<u32> {
        let chunk_id = self.conn.execute(
            "INSERT INTO chunks (document_id, text, chunk_index) VALUES (?, ?, ?)",
            params![document_id, text, chunk_index]
        )? as u32;
        
        Ok(chunk_id)
    }

    pub fn get_chunk(&self, chunk_id: u32) -> Result<Option<Chunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, text, chunk_index FROM chunks WHERE id = ?"
        )?;
        
        let mut rows = stmt.query_map([chunk_id], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                text: row.get(2)?,
                chunk_index: row.get(3)?,
            })
        })?;
        
        Ok(rows.next().transpose()?)
    }

    pub fn get_chunks_by_document(&self, document_id: u32) -> Result<Vec<Chunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, text, chunk_index FROM chunks WHERE document_id = ? ORDER BY chunk_index"
        )?;
        
        let rows = stmt.query_map([document_id], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                text: row.get(2)?,
                chunk_index: row.get(3)?,
            })
        })?;
        
        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row?);
        }
        Ok(chunks)
    }

    pub fn add_embedding(&mut self, chunk_id: u32, vector: &[f32]) -> Result<u32> {
        let vector_json = serde_json::to_string(vector)?;
        let embedding_id = self.conn.execute(
            "INSERT INTO embeddings (chunk_id, vector) VALUES (?, ?)",
            params![chunk_id, vector_json]
        )? as u32;
        
        Ok(embedding_id)
    }

    pub fn get_embedding(&self, chunk_id: u32) -> Result<Option<Embedding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chunk_id, vector FROM embeddings WHERE chunk_id = ?"
        )?;
        
        let mut rows = stmt.query_map([chunk_id], |row| {
            let vector_json: String = row.get(2)?;
            let vector: Vec<f32> = serde_json::from_str(&vector_json).unwrap_or_default();
            Ok(Embedding {
                id: row.get(0)?,
                chunk_id: row.get(1)?,
                vector,
            })
        })?;
        
        Ok(rows.next().transpose()?)
    }

    pub fn get_all_embeddings(&self) -> Result<Vec<Embedding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chunk_id, vector FROM embeddings ORDER BY id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let vector_json: String = row.get(2)?;
            let vector: Vec<f32> = serde_json::from_str(&vector_json).unwrap_or_default();
            Ok(Embedding {
                id: row.get(0)?,
                chunk_id: row.get(1)?,
                vector,
            })
        })?;
        
        let mut embeddings = Vec::new();
        for row in rows {
            embeddings.push(row?);
        }
        Ok(embeddings)
    }

    pub fn add_document_with_chunks(&mut self, file_path: &str, file_hash: &str, size: usize, chunks: &[Chunk], embeddings: &[Vec<f32>]) -> Result<(u32, Vec<u32>)> {
        let tx = self.conn.transaction()?;
        
        // Add document
        let document_id = tx.execute(
            "INSERT INTO documents (file_path, file_hash, size, chunk_count) VALUES (?, ?, ?, ?)",
            params![file_path, file_hash, size, chunks.len()]
        )? as u32;
        
        let mut chunk_ids = Vec::new();
        
        // Add chunks and embeddings
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            let chunk_id = tx.execute(
                "INSERT INTO chunks (document_id, text, chunk_index) VALUES (?, ?, ?)",
                params![document_id, chunk.text, chunk.chunk_index]
            )? as u32;
            
            chunk_ids.push(chunk_id);
            
            // Add embedding
            let vector_json = serde_json::to_string(embedding)?;
            tx.execute(
                "INSERT INTO embeddings (chunk_id, vector) VALUES (?, ?)",
                params![chunk_id, vector_json]
            )?;
        }
        
        tx.commit()?;
        Ok((document_id, chunk_ids))
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
        
        // Calculate database size
        let db_size: u64 = std::fs::metadata("chunkymonkey.db")?.len();
        let database_size_mb = db_size as f64 / (1024.0 * 1024.0);
        
        Ok(DatabaseStats {
            document_count,
            chunk_count,
            database_size_mb,
        })
    }

    pub fn clear_all(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM embeddings;
             DELETE FROM chunks;
             DELETE FROM documents;"
        )?;
        Ok(())
    }
} 