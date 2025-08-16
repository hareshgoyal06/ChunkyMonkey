use anyhow::Result;
use rusqlite::{Connection, params};
use crate::core::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

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
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL,
                document_count INTEGER DEFAULT 0,
                chunk_count INTEGER DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                file_path TEXT UNIQUE NOT NULL,
                file_hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                chunk_count INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS project_documents (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                document_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                added_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects (id),
                FOREIGN KEY (document_id) REFERENCES documents (id)
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

    pub fn create_project(&mut self, name: &str, description: &str) -> Result<u32> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let created_at = timestamp.to_string();
        
        let project_id = self.conn.execute(
            "INSERT INTO projects (name, description, created_at) VALUES (?, ?, ?)",
            params![name, description, created_at]
        )? as u32;
        
        Ok(project_id)
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, document_count, chunk_count FROM projects ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                document_count: row.get(4)?,
                chunk_count: row.get(5)?,
            })
        })?;
        
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        
        Ok(projects)
    }

    pub fn get_project(&self, project_id: u32) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, document_count, chunk_count FROM projects WHERE id = ?"
        )?;
        
        let mut rows = stmt.query_map([project_id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                document_count: row.get(4)?,
                chunk_count: row.get(5)?,
                created_at: row.get(3)?,
            })
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn add_document_to_project(&mut self, project_id: u32, document_id: u32, file_path: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let added_at = timestamp.to_string();
        
        self.conn.execute(
            "INSERT INTO project_documents (project_id, document_id, file_path, added_at) VALUES (?, ?, ?, ?)",
            params![project_id, document_id, file_path, added_at]
        )?;
        
        // Update project document count
        self.conn.execute(
            "UPDATE projects SET document_count = document_count + 1 WHERE id = ?",
            params![project_id]
        )?;
        
        Ok(())
    }

    pub fn get_project_documents(&self, project_id: u32) -> Result<Vec<ProjectDocument>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, document_id, file_path, added_at FROM project_documents WHERE project_id = ? ORDER BY added_at DESC"
        )?;
        
        let rows = stmt.query_map([project_id], |row| {
            Ok(ProjectDocument {
                id: row.get(0)?,
                project_id: row.get(1)?,
                document_id: row.get(2)?,
                file_path: row.get(3)?,
                added_at: row.get(4)?,
            })
        })?;
        
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }
        
        Ok(documents)
    }

    pub fn add_document_with_chunks(
        &mut self,
        file_path: &str,
        file_hash: &str,
        size: usize,
        chunks: &[Chunk],
        embeddings: &[Vec<f32>],
    ) -> Result<(u32, Vec<u32>)> {
        let tx = self.conn.transaction()?;
        
        // Insert document
        let document_id = tx.execute(
            "INSERT INTO documents (file_path, file_hash, size, chunk_count) VALUES (?, ?, ?, ?)",
            params![file_path, file_hash, size, chunks.len()]
        )? as u32;
        
        let mut chunk_ids = Vec::new();
        
        // Insert chunks
        for (i, chunk) in chunks.iter().enumerate() {
            tx.execute(
                "INSERT INTO chunks (document_id, text, chunk_index) VALUES (?, ?, ?)",
                params![document_id, chunk.text, i]
            )?;
            
            // Get the ID of the chunk we just inserted
            let chunk_id = tx.last_insert_rowid() as u32;
            chunk_ids.push(chunk_id);
            
            // Insert embedding
            let vector_json = serde_json::to_string(&embeddings[i])?;
            tx.execute(
                "INSERT INTO embeddings (chunk_id, vector) VALUES (?, ?)",
                params![chunk_id, vector_json]
            )?;
        }
        
        tx.commit()?;
        Ok((document_id, chunk_ids))
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
                    project_name: None, // TODO: Get project name from document
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
        let project_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM projects",
            [],
            |row| row.get(0)
        )?;
        
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
            project_count,
            document_count,
            chunk_count,
            database_size_mb: db_size_mb,
        })
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM embeddings;
             DELETE FROM chunks;
             DELETE FROM project_documents;
             DELETE FROM documents;
             DELETE FROM projects;"
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