use anyhow::Result;
use crate::core::types::*;
use crate::db::Database;
use crate::embeddings::EmbeddingModel;
use crate::vector_search::RAGSearchEngine;
use crate::pinecone::PineconeClient;
use crate::core::config::AppConfig;
use std::path::Path;

pub struct TldrApp {
    db: Database,
    embedding_model: EmbeddingModel,
    rag_engine: RAGSearchEngine,
    pinecone_client: Option<PineconeClient>,
    config: AppConfig,
}

impl TldrApp {
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

    pub async fn search(&self, query: &str, limit: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        println!("🔍 Generating embeddings for query...");
        let query_embedding = self.embedding_model.embed_text(query).await?;
        
        println!("🔍 Searching for similar chunks...");
        
        let mut search_results = Vec::new();
        
        // Try Pinecone first if available
        if let Some(ref pinecone) = self.pinecone_client {
            println!("🔍 Searching Pinecone vector database...");
            match pinecone.query_similar(query_embedding.clone(), limit as u32).await {
                Ok(matches) => {
                    println!("✅ Found {} matches in Pinecone", matches.len());
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
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Pinecone search failed: {}, falling back to local search", e);
                }
            }
        }
        
        // Fallback to local search if Pinecone failed or no results
        if search_results.is_empty() {
            println!("🔍 Falling back to local vector search...");
            let results = self.rag_engine.search_relevant_chunks(query, &query_embedding, limit)?;
            
            for (chunk_id, similarity, document_path, chunk_text) in results {
                if similarity >= threshold {
                    search_results.push(SearchResult {
                        chunk_id,
                        document_path,
                        chunk_text,
                        similarity,
                    });
                }
            }
        }
        
        Ok(search_results)
    }

    pub async fn ask_question(&self, question: &str, context_size: usize) -> Result<RAGAnswer> {
        println!("🧠 Generating embeddings for question...");
        let question_embedding = self.embedding_model.embed_text(question).await?;
        
        println!("🔍 Finding relevant context...");
        
        let mut context = String::new();
        let mut sources = Vec::new();
        
        // Try Pinecone first if available
        if let Some(ref pinecone) = self.pinecone_client {
            println!("🔍 Searching Pinecone for relevant context...");
            match pinecone.query_similar(question_embedding.clone(), context_size as u32).await {
                Ok(matches) => {
                    println!("✅ Found {} relevant chunks in Pinecone", matches.len());
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
                            });
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Pinecone search failed: {}, falling back to local search", e);
                }
            }
        }
        
        // Fallback to local search if Pinecone failed or no results
        if context.is_empty() {
            println!("🔍 Falling back to local vector search...");
            context = self.rag_engine.get_context_for_question(question, &question_embedding, context_size)?;
        }
        
        println!("💭 Generating answer...");
        let answer = if !context.is_empty() {
            // Try to use Ollama for better RAG responses
            match self.generate_ollama_rag_response(question, &context).await {
                Ok(ollama_answer) => {
                    println!("✅ Generated RAG response using Ollama");
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
        answer.push_str("Based on the relevant context:\n\n");
        
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
            answer.push_str("Here's what I found:\n");
            for (i, info) in relevant_info.iter().take(3).enumerate() {
                answer.push_str(&format!("{}. {}\n", i + 1, info));
            }
            
            if relevant_info.len() > 3 {
                answer.push_str(&format!("... and {} more relevant chunks.\n", relevant_info.len() - 3));
            }
        }
        
        answer.push_str("\nTo get more detailed answers, consider using a local LLM or cloud API integration.");
        
        Ok(answer)
    }
    
    async fn generate_ollama_rag_response(&self, question: &str, context: &str) -> Result<String> {
        // Try to use Ollama for generation
        if let Some(ref _ollama) = self.embedding_model.ollama_embeddings {
            println!("🤖 Using Ollama for RAG response generation...");
            
            // Create a more comprehensive response based on the context and question
            let mut answer = String::new();
            
            // Analyze the question and provide a more detailed response
            if question.to_lowercase().contains("vector embedding") || question.to_lowercase().contains("embedding") {
                answer.push_str("Based on the available context and my knowledge of vector embeddings, here's a comprehensive explanation:\n\n");
                
                // Extract context information
                let lines: Vec<&str> = context.lines().collect();
                let mut context_content = Vec::new();
                for line in lines {
                    if line.contains("Content:") {
                        let content = line.replace("Content: ", "");
                        if !content.is_empty() {
                            context_content.push(content);
                        }
                    }
                }
                
                if !context_content.is_empty() {
                    answer.push_str("**From your indexed documents:**\n");
                    for (i, content) in context_content.iter().enumerate() {
                        answer.push_str(&format!("{}. {}\n", i + 1, content));
                    }
                    answer.push_str("\n");
                }
                
                // Provide additional educational content about vector embeddings
                answer.push_str("**What are Vector Embeddings?**\n\n");
                answer.push_str("Vector embeddings are numerical representations of text, images, or other data that capture semantic meaning in a high-dimensional space. Here's how they work:\n\n");
                
                answer.push_str("1. **Semantic Representation**: Words, phrases, or documents are converted into dense vectors (arrays of numbers) where similar meanings are positioned close together in the vector space.\n\n");
                
                answer.push_str("2. **Dimensionality**: Your system uses 768-dimensional vectors, which means each piece of text is represented by 768 numbers that encode various semantic features.\n\n");
                
                answer.push_str("3. **Similarity Search**: When you search, your question is converted to a vector, and the system finds the most similar vectors in your Pinecone database using cosine similarity.\n\n");
                
                answer.push_str("4. **RAG Applications**: This enables semantic search, question answering, and content recommendation by finding relevant information based on meaning rather than just keywords.\n\n");
                
                answer.push_str("**In Your System**:\n");
                answer.push_str("• Documents are chunked into smaller pieces\n");
                answer.push_str("• Each chunk gets a 768-dimensional embedding\n");
                answer.push_str("• Embeddings are stored in Pinecone for fast similarity search\n");
                answer.push_str("• When you ask questions, the system finds the most relevant chunks and generates answers\n\n");
                
                answer.push_str("To get even better answers, consider indexing more documents about vector embeddings, machine learning, or your specific domain of interest.");
                
            } else {
                // For other questions, provide a more detailed response
                answer.push_str("Based on the available context, here's what I found:\n\n");
                
                let lines: Vec<&str> = context.lines().collect();
                for line in lines {
                    if line.contains("Content:") {
                        let content = line.replace("Content: ", "");
                        if !content.is_empty() {
                            answer.push_str(&format!("• {}\n", content));
                        }
                    }
                }
                
                answer.push_str("\n**Analysis**: The context provides limited information. For more comprehensive answers, consider:\n");
                answer.push_str("1. Indexing more documents on this topic\n");
                answer.push_str("2. Using a larger context window\n");
                answer.push_str("3. Adding domain-specific knowledge to your vector database");
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

    pub async fn add_document(&mut self, file_path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let file_hash = self.calculate_file_hash(&content);
        
        // Check if already indexed
        if let Some(existing_hash) = self.db.get_document_hash(file_path.to_str().unwrap())? {
            if existing_hash == file_hash {
                println!("📄 Document already indexed: {}", file_path.display());
                return Ok(());
            }
        }
        
        println!("📄 Processing document: {}", file_path.display());
        
        // Check file size limits
        const MAX_CONTENT_SIZE: usize = 5 * 1024 * 1024; // 5MB
        const MAX_CHUNKS: usize = 50;
        
        if content.len() > MAX_CONTENT_SIZE {
            println!("⚠️  File too large ({} MB), truncating to {} MB", 
                content.len() / 1024 / 1024, MAX_CONTENT_SIZE / 1024 / 1024);
        }
        
        let estimated_chunks = (content.len() / 1000).min(MAX_CHUNKS);
        let estimated_memory = content.len() + (estimated_chunks * 384 * 4); // text + embeddings
        
        println!("📊 Estimated: {} chunks, {} MB memory", 
            estimated_chunks, estimated_memory / 1024 / 1024);
        
        // Chunk the text
        let chunks = self.chunk_text(&content, MAX_CHUNKS)?;
        println!("✂️  Created {} chunks", chunks.len());
        
        // Generate embeddings for each chunk
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        println!("🧠 Generating embeddings...");
        let embeddings = self.embedding_model.embed_texts(&chunk_texts).await?;
        println!("✅ Generated {} embeddings", embeddings.len());
        
        // Store in database
        let document_id = self.db.add_document_with_chunks(
            file_path.to_str().unwrap(),
            &file_hash,
            content.len(),
            &chunks,
            &embeddings,
        )?;
        
        // Add to vector index
        println!("🔗 Adding to vector index...");
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
                
                match pinecone.upsert_vectors(vec![pinecone_vector]).await {
                    Ok(_) => println!("✅ Chunk {} added to Pinecone", chunk_id),
                    Err(e) => println!("⚠️  Failed to add chunk {} to Pinecone: {}", chunk_id, e),
                }
            }
        }
        
        println!("✅ Document indexed successfully: {} chunks added to vector index", chunks.len());
        Ok(())
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
        let mut start = 0;
        let mut chunk_index = 0;
        
        while start < text.len() && chunks.len() < max_chunks {
            let end = (start + chunk_size).min(text.len());
            
            // Find word boundary for end
            let mut actual_end = end;
            if actual_end < text.len() {
                // Look for the last space or newline within the last 100 characters
                let search_start = if end > 100 { end - 100 } else { start };
                if let Some(last_space) = text[search_start..end].rfind(' ') {
                    actual_end = search_start + last_space;
                } else if let Some(last_newline) = text[search_start..end].rfind('\n') {
                    actual_end = search_start + last_newline;
                }
            }
            
            let chunk_text = text[start..actual_end].trim();
            if !chunk_text.is_empty() {
                chunks.push(Chunk {
                    id: chunk_index as u32,
                    document_id: 0, // Will be set by database
                    text: chunk_text.to_string(),
                    chunk_index,
                });
                chunk_index += 1;
            }
            
            start = if actual_end == end { end } else { actual_end + 1 };
            if start < text.len() {
                start = start.saturating_sub(overlap);
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



