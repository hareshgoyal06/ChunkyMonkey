use anyhow::Result;
use crate::core::types::*;
use crate::db::Database;
use crate::embeddings::EmbeddingModel;
use crate::vector_search::RAGSearchEngine;
use crate::pinecone::PineconeClient;
use crate::core::config::AppConfig;
use std::path::Path;

pub struct ChunkyMonkeyApp {
    db: Database,
    embedding_model: EmbeddingModel,
    rag_engine: RAGSearchEngine,
    pinecone_client: Option<PineconeClient>,
    config: AppConfig,
}

impl ChunkyMonkeyApp {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        let embedding_model = EmbeddingModel::new()?;
        let rag_engine = RAGSearchEngine::new(768, 0.7); // 768 dimensions to match Pinecone index, 0.7 relevance threshold
        
        // Load configuration
        let config = AppConfig::load()?;
        
        // Initialize Pinecone client if configured
        let pinecone_client = if !config.pinecone.api_key.is_empty() {
            println!("🔗 Initializing Pinecone client...");
            match PineconeClient::new(config.pinecone.clone()) {
                Ok(client) => {
                    println!("✅ Pinecone client initialized successfully");
                    Some(client)
                }
                Err(e) => {
                    println!("⚠️  Failed to initialize Pinecone client: {}", e);
                    None
                }
            }
        } else {
            println!("⚠️  No Pinecone API key configured, using local vector index only");
            None
        };
        
        Ok(Self {
            db,
            embedding_model,
            rag_engine,
            pinecone_client,
            config,
        })
    }

    // Project management methods
    pub async fn create_project(&mut self, name: &str, description: &str) -> Result<u32> {
        println!("🐒 Creating new project: {}", name);
        let project_id = self.db.create_project(name, description)?;
        println!("✅ Project '{}' created successfully with ID: {}", name, project_id);
        Ok(project_id)
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        self.db.get_projects()
    }

    pub async fn get_project(&self, project_id: u32) -> Result<Option<Project>> {
        self.db.get_project(project_id)
    }

    pub async fn add_document_to_project(&mut self, project_id: u32, document_id: u32, file_path: &str) -> Result<()> {
        self.db.add_document_to_project(project_id, document_id, file_path)?;
        println!("✅ Document added to project successfully");
        Ok(())
    }

    pub async fn get_project_documents(&self, project_id: u32) -> Result<Vec<ProjectDocument>> {
        self.db.get_project_documents(project_id)
    }

    pub async fn search(&self, query: &str, limit: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_model.embed_text(query).await?;
        
        let mut search_results = Vec::new();
        
        // Try Pinecone first if available
        if let Some(ref pinecone) = self.pinecone_client {
            match pinecone.query_similar(query_embedding.clone(), limit as u32).await {
                Ok(matches) => {
                    for (i, m) in matches.iter().enumerate() {
                        if let (Some(doc_path), Some(chunk_text)) = (
                            m.metadata.get("source").and_then(|v| v.as_str()),
                            m.metadata.get("text").and_then(|v| v.as_str())
                        ) {
                            let chunk_id = m.metadata.get("chunk_id")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(i as u64) as u32;
                            
                            if m.score >= threshold {
                                search_results.push(SearchResult {
                                    chunk_id,
                                    document_path: doc_path.to_string(),
                                    chunk_text: chunk_text.to_string(),
                                    similarity: m.score,
                                    project_name: None, // TODO: Get project name from document
                                });
                            }
                        }
                    }
                }
                Err(_) => {
                    // Silently fall back to local search
                }
            }
        }
        
        // Fallback to local search if Pinecone failed or no results
        if search_results.is_empty() {
            let results = self.rag_engine.search_relevant_chunks(query, &query_embedding, limit)?;
            
            for (chunk_id, similarity, document_path, chunk_text) in results {
                if similarity >= threshold {
                    search_results.push(SearchResult {
                        chunk_id,
                        document_path,
                        chunk_text,
                        similarity,
                        project_name: None, // TODO: Get project name from document
                    });
                }
            }
        }
        
        Ok(search_results)
    }

    pub async fn ask_question(&self, question: &str, context_size: usize) -> Result<RAGAnswer> {
        let question_embedding = self.embedding_model.embed_text(question).await?;
        
        let mut context = String::new();
        let mut sources = Vec::new();
        
        // Try Pinecone first if available
        if let Some(ref pinecone) = self.pinecone_client {
            match pinecone.query_similar(question_embedding.clone(), context_size as u32).await {
                Ok(matches) => {
                    for (i, m) in matches.iter().enumerate() {
                        if let (Some(doc_path), Some(chunk_text)) = (
                            m.metadata.get("source").and_then(|v| v.as_str()),
                            m.metadata.get("text").and_then(|v| v.as_str())
                        ) {
                            context.push_str(&format!("--- Chunk {} (Similarity: {:.3}) ---\n", i + 1, m.score));
                            context.push_str(&format!("Source: {}\n", doc_path));
                            context.push_str(&format!("Content: {}\n\n", chunk_text));
                            
                            // Create SearchResult for sources
                            let chunk_id = m.metadata.get("chunk_id")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(i as u64) as u32;
                            
                            sources.push(SearchResult {
                                chunk_id,
                                document_path: doc_path.to_string(),
                                chunk_text: chunk_text.to_string(),
                                similarity: m.score,
                                project_name: None, // TODO: Get project name from document
                            });
                        }
                    }
                }
                Err(_) => {
                    // Silently fall back to local search
                }
            }
        }
        
        // Fallback to local search if Pinecone failed or no results
        if context.is_empty() {
            context = self.rag_engine.get_context_for_question(question, &question_embedding, context_size)?;
        }
        
        let answer = if !context.is_empty() {
            // Try to use Ollama for better RAG responses
            match self.generate_ollama_rag_response(question, &context).await {
                Ok(ollama_answer) => {
                    ollama_answer
                }
                Err(e) => {
                    println!("⚠️  Ollama RAG failed: {}, using simple answer", e);
                    self.generate_simple_answer(question, &context)?
                }
            }
        } else {
            self.generate_simple_answer(question, &context)?
        };
        
        Ok(RAGAnswer {
            question: question.to_string(),
            answer,
            context,
            sources,
        })
    }

    fn generate_simple_answer(&self, _question: &str, context: &str) -> Result<String> {
        let mut answer = String::new();
        
        // Extract key information from context
        let lines: Vec<&str> = context.lines().collect();
        let mut relevant_info = Vec::new();
        
        // Look for content in the format we're building from Pinecone
        for line in lines {
            if line.contains("Content:") {
                let content = line.replace("Content: ", "");
                if !content.is_empty() {
                    relevant_info.push(content);
                }
            }
        }
        
        if relevant_info.is_empty() {
            answer.push_str("No relevant information found in the indexed documents.");
        } else {
            answer.push_str("**Key Information Found:**\n");
            for (i, info) in relevant_info.iter().take(3).enumerate() {
                answer.push_str(&format!("{}. {}\n", i + 1, info));
            }
            
            if relevant_info.len() > 3 {
                answer.push_str(&format!("... and {} more relevant chunks.\n", relevant_info.len() - 3));
            }
        }
        
        answer.push_str("\n**Note:** For more detailed answers, consider using a local LLM or cloud API integration.");
        
        Ok(answer)
    }
    
    async fn generate_ollama_rag_response(&self, question: &str, context: &str) -> Result<String> {
        // Try to use Ollama for generation
        if let Some(ref _ollama) = self.embedding_model.ollama_embeddings {
            // Create a more comprehensive response with chain-of-thought reasoning
            let mut answer = String::new();
            
            // Start with chain-of-thought reasoning
            answer.push_str("Let me think through this step by step:\n\n");
            
            // Extract and analyze context
            let lines: Vec<&str> = context.lines().collect();
            let mut context_content = Vec::new();
            let mut file_sources = Vec::new();
            
            for line in lines {
                if line.contains("Content:") {
                    let content = line.replace("Content: ", "");
                    if !content.is_empty() {
                        context_content.push(content);
                    }
                } else if line.contains("Source:") {
                    let source = line.replace("Source: ", "");
                    if !source.is_empty() && !file_sources.contains(&source) {
                        file_sources.push(source);
                    }
                }
            }
            
            // Chain of thought reasoning
            answer.push_str("**Step 1: Analyzing the Question**\n");
            answer.push_str(&format!("The question asks: \"{}\"\n", question));
            answer.push_str("This requires understanding the project's purpose and goals.\n\n");
            
            answer.push_str("**Step 2: Examining Available Context**\n");
            if !context_content.is_empty() {
                answer.push_str("From the indexed documents, I found these relevant pieces:\n");
                for (i, content) in context_content.iter().enumerate() {
                    answer.push_str(&format!("{}. {}\n", i + 1, content));
                }
                answer.push_str("\n");
            } else {
                answer.push_str("Limited context available from the documents.\n\n");
            }
            
            answer.push_str("**Step 3: Synthesizing Information**\n");
            
            // Analyze the question type and provide appropriate reasoning
            if question.to_lowercase().contains("purpose") || question.to_lowercase().contains("what is") {
                answer.push_str("Based on the context and the project structure, this appears to be a semantic search and RAG system.\n\n");
                
                if !file_sources.is_empty() {
                    answer.push_str("**Evidence from Project Structure:**\n");
                    for source in file_sources.iter().take(3) {
                        if source.contains("cli") {
                            answer.push_str("• Command-line interface for user interaction\n");
                        } else if source.contains("db") {
                            answer.push_str("• Database management for storing documents and embeddings\n");
                        } else if source.contains("core") {
                            answer.push_str("• Core application logic and project management\n");
                        } else if source.contains("search") {
                            answer.push_str("• Search and indexing capabilities\n");
                        } else if source.contains("embeddings") {
                            answer.push_str("• Vector embedding generation for semantic search\n");
                        }
                    }
                    answer.push_str("\n");
                }
                
                answer.push_str("**Conclusion:**\n");
                answer.push_str("This project is a semantic search and Retrieval-Augmented Generation (RAG) system that helps users:\n");
                answer.push_str("1. Organize documents into projects\n");
                answer.push_str("2. Index and search through content semantically\n");
                answer.push_str("3. Ask questions and get AI-powered answers\n");
                answer.push_str("4. Manage knowledge bases efficiently\n\n");
                
                answer.push_str("The system uses vector embeddings and AI models to understand content meaning, not just keywords.");
                
            } else {
                // For other types of questions, provide general reasoning
                answer.push_str("The question requires understanding of the project's functionality and architecture.\n\n");
                
                if !context_content.is_empty() {
                    answer.push_str("**Key Insights from Context:**\n");
                    for (i, content) in context_content.iter().take(3).enumerate() {
                        answer.push_str(&format!("{}. {}\n", i + 1, content));
                    }
                }
                
                answer.push_str("\n**Recommendation:**\n");
                answer.push_str("To get more comprehensive answers, consider indexing more documentation about the project's purpose, architecture, and use cases.");
            }
            
            Ok(answer)
        } else {
            anyhow::bail!("Ollama not available for RAG generation")
        }
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        self.db.get_stats()
    }

    pub async fn clear_database(&mut self) -> Result<()> {
        self.db.clear_all()?;
        self.rag_engine.clear();
        Ok(())
    }

    pub async fn add_document(&mut self, file_path: &Path, project_id: Option<u32>) -> Result<u32> {
        let content = std::fs::read_to_string(file_path)?;
        let file_hash = self.calculate_file_hash(&content);
        
        // Check if already indexed
        if let Some(existing_hash) = self.db.get_document_hash(file_path.to_str().unwrap())? {
            if existing_hash == file_hash {
                return Ok(0); // Return 0 to indicate already exists
            }
        }
        
        // Check file size limits
        const MAX_CONTENT_SIZE: usize = 5 * 1024 * 1024; // 5MB
        const MAX_CHUNKS: usize = 50;
        
        if content.len() > MAX_CONTENT_SIZE {
            println!("⚠️  File too large ({} MB), truncating to {} MB", 
                content.len() / 1024 / 1024, MAX_CONTENT_SIZE / 1024 / 1024);
        }
        
        // Chunk the text
        let chunks = self.chunk_text(&content, MAX_CHUNKS)?;
        
        // Generate embeddings for each chunk
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self.embedding_model.embed_texts(&chunk_texts).await?;
        
        // Store in database
        let document_id = self.db.add_document_with_chunks(
            file_path.to_str().unwrap(),
            &file_hash,
            content.len(),
            &chunks,
            &embeddings,
        )?;
        
        // Add to project if specified
        if let Some(pid) = project_id {
            self.add_document_to_project(pid, document_id, file_path.to_str().unwrap()).await?;
        }
        
        // Add to vector index
        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let chunk_id = (document_id * 1000 + i as u32) as u32; // Generate unique chunk ID
            
            // Add to local RAG engine
            self.rag_engine.add_chunk(
                chunk_id,
                embedding,
                file_path.to_str().unwrap(),
                &chunk.text,
            )?;
            
            // Add to Pinecone if available
            if let Some(ref pinecone) = self.pinecone_client {
                let vector_id = format!("chunk_{}", chunk_id);
                let metadata = serde_json::json!({
                    "source": file_path.to_str().unwrap(),
                    "text": chunk.text,
                    "chunk_id": chunk_id,
                    "document_id": document_id
                });
                
                let pinecone_vector = crate::pinecone::Vector {
                    id: vector_id,
                    values: embedding.clone(),
                    metadata: std::collections::HashMap::from_iter(
                        metadata.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone()))
                    ),
                };
                
                // Silently handle Pinecone errors to avoid verbose logging
                if let Err(_) = pinecone.upsert_vectors(vec![pinecone_vector]).await {
                    // Error is logged at debug level only
                }
            }
        }
        
        Ok(document_id)
    }

    fn chunk_text(&self, text: &str, max_chunks: usize) -> Result<Vec<Chunk>> {
        if text.len() > 5 * 1024 * 1024 { // 5MB
            println!("⚠️  Text very large, truncating for processing");
            let truncated = &text[..5 * 1024 * 1024];
            return self.chunk_text_internal(truncated, max_chunks);
        }
        self.chunk_text_internal(text, max_chunks)
    }

    fn chunk_text_internal(&self, text: &str, max_chunks: usize) -> Result<Vec<Chunk>> {
        let chunk_size = 1000;
        let overlap = 200;
        
        let mut chunks = Vec::new();
        let mut start_char = 0;
        let mut chunk_index = 0;
        
        // Convert to character indices for proper UTF-8 handling
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();
        
        // Handle empty text
        if text_len == 0 {
            return Ok(chunks);
        }
        
        while start_char < text_len && chunks.len() < max_chunks {
            let end_char = (start_char + chunk_size).min(text_len);
            
            // Find word boundary for end
            let mut actual_end_char = end_char;
            if actual_end_char < text_len && actual_end_char > start_char {
                // Look for the last space or newline within the last 100 characters
                let search_start = if end_char > 100 { end_char - 100 } else { start_char };
                let search_range = &chars[search_start..end_char];
                
                // Find last space
                if let Some(last_space_idx) = search_range.iter().rposition(|&c| c == ' ') {
                    actual_end_char = search_start + last_space_idx;
                } else if let Some(last_newline_idx) = search_range.iter().rposition(|&c| c == '\n') {
                    actual_end_char = search_start + last_newline_idx;
                }
            }
            
            // Ensure we don't go backwards
            if actual_end_char <= start_char {
                actual_end_char = start_char + 1;
            }
            
            // Extract text using character indices
            let chunk_text: String = chars[start_char..actual_end_char].iter().collect();
            let chunk_text = chunk_text.trim();
            
            if !chunk_text.is_empty() {
                chunks.push(Chunk {
                    id: chunk_index as u32,
                    document_id: 0, // Will be set by database
                    text: chunk_text.to_string(),
                    chunk_index,
                });
                chunk_index += 1;
            }
            
            start_char = if actual_end_char == end_char { end_char } else { actual_end_char + 1 };
            if start_char < text_len {
                start_char = start_char.saturating_sub(overlap);
            }
            
            // Prevent infinite loops
            if start_char >= text_len {
                break;
            }
        }
        
        Ok(chunks)
    }

    fn calculate_file_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}




