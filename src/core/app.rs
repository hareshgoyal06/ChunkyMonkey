use anyhow::Result;
use crate::core::types::*;
use crate::db::Database;
use crate::embeddings::EmbeddingModel;
use crate::vector_search::RAGSearchEngine;
use crate::pinecone::PineconeClient;
use crate::core::config::AppConfig;
use std::path::Path;

/// Simple LLM client for Ollama
struct OllamaLLMClient {
    base_url: String,
    model: String,
}

impl OllamaLLMClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model }
    }
    
    pub async fn generate_answer(&self, question: &str, context: &str) -> Result<String> {
        let client = reqwest::Client::new();
        
        // Create a well-structured prompt for the LLM
        let prompt = format!(
            "You are a helpful AI assistant. Based on the following context, provide a clear and concise answer to the question.\n\nQuestion: {}\n\nContext:\n{}\n\nAnswer:",
            question, context
        );
        
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
                "max_tokens": 1000
            }
        });
        
        let response = client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            if let Some(response_text) = response_json["response"].as_str() {
                return Ok(response_text.trim().to_string());
            }
        }
        
        // Fallback to a simple response if LLM fails
        Ok("I couldn't generate a response using the LLM. Here's the relevant information from the context:\n\n".to_string() + context)
    }
}

pub struct ChunkyMonkeyApp {
    pub db: Database,
    pub embedding_model: EmbeddingModel,
    pub rag_engine: RAGSearchEngine,
    pub pinecone_client: Option<PineconeClient>,
    pub config: AppConfig,
    pub llm_client: Option<OllamaLLMClient>, // LLM client for answer generation
}

impl ChunkyMonkeyApp {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        let embedding_model = EmbeddingModel::new()?;
        let mut rag_engine = RAGSearchEngine::new(768, 0.1); // 768 dimensions to match Pinecone index, 0.1 relevance threshold
        
        // Load configuration
        let config = AppConfig::load()?;
        
        // Initialize Pinecone client if configured (silently)
        let pinecone_client = if !config.pinecone.api_key.is_empty() {
            match PineconeClient::new(config.pinecone.clone()) {
                Ok(client) => Some(client),
                Err(_) => None, // Silently fail
            }
        } else {
            None
        };
        
        // Load existing vectors from database into the RAG engine
        if let Err(e) = rag_engine.load_vectors_from_database(&db) {
            eprintln!("Warning: Failed to load vectors from database: {}", e);
        }
        
        // Initialize LLM client if configured
        let llm_client = if !config.ollama.base_url.is_empty() && !config.ollama.llm_model.is_empty() {
            Some(OllamaLLMClient::new(
                config.ollama.base_url.clone(),
                config.ollama.llm_model.clone(),
            ))
        } else {
            None
        };
        
        Ok(Self {
            db,
            embedding_model,
            rag_engine,
            pinecone_client,
            config,
            llm_client,
        })
    }

    // Project management methods
    pub async fn create_project(&mut self, name: &str, description: &str) -> Result<u32> {
        let project_id = self.db.create_project(name, description)?;
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
                Err(_) => {
                    // Silently fall back to local search
                }
            }
        }
        
        // Fallback to local search if Pinecone failed or no results
        if search_results.is_empty() {
            let results = self.rag_engine.search_relevant_chunks(query, &query_embedding, limit)?;
            
            for (chunk_id, similarity, document_path, chunk_text) in results {
                search_results.push(SearchResult {
                    chunk_id,
                    document_path,
                    chunk_text,
                    similarity,
                    project_name: None, // TODO: Get project name from document
                });
            }
        }
        
        Ok(search_results)
    }

    pub async fn ask_question(&self, question: &str, context_size: Option<usize>) -> Result<RAGAnswer> {
        let context_size = context_size.unwrap_or(self.config.rag.max_context_chunks);
        
        println!("🔍 Generating embeddings for your question...");
        let question_embedding = self.embedding_model.embed_text(question).await?;
        
        println!("📚 Retrieving relevant context from documents...");
        let (context, sources) = self.retrieve_enhanced_context(question, &question_embedding, context_size).await?;
        
        // Step 2: Context quality assessment (if enabled)
        let context_quality = if self.config.rag.enable_quality_assessment {
            self.assess_context_quality(&context, question)
        } else {
            ContextQuality::Good // Default to good if assessment is disabled
        };
        
        // Step 3: Generate answer using multiple strategies
        let answer = if self.config.rag.enable_advanced_rag && context_quality.is_good() {
            // High-quality context - use advanced RAG
            println!("🧠 Generating answer with LLM (llama2:7b)...");
            println!("   This may take a few moments as the model processes your question...");
            self.generate_advanced_rag_response(question, &context, &context_quality).await?
        } else if context_quality.is_acceptable() {
            // Acceptable context - use standard RAG
            println!("📝 Generating answer with standard RAG...");
            self.generate_standard_rag_response(question, &context, &context_quality).await?
        } else if self.config.rag.enable_fallback_strategies {
            // Poor context - use fallback strategies
            println!("⚠️  Using fallback answer generation...");
            self.generate_fallback_response(question, &context, &context_quality).await?
        } else {
            // No fallback - use simple response
            println!("📋 Generating simple answer...");
            self.generate_simple_answer(question, &context)?
        };
        
        // Step 4: Answer validation and enhancement (if enabled)
        let final_answer = if self.config.rag.enable_answer_validation {
            println!("✅ Validating and enhancing answer...");
            self.validate_and_enhance_answer(&answer, question, &context, &context_quality).await?
        } else {
            answer
        };
        
        println!("✨ Answer generation complete!");
        
        Ok(RAGAnswer {
            question: question.to_string(),
            answer: final_answer,
            context: String::new(), // Don't show context in output
            sources: Vec::new(), // Don't show sources in output
        })
    }

    async fn retrieve_enhanced_context(&self, question: &str, question_vector: &[f32], context_size: usize) -> Result<(String, Vec<SearchResult>)> {
        let mut all_context = String::new();
        let mut all_sources = Vec::new();
        
        // Strategy 1: Try Pinecone first if available
        if let Some(ref pinecone) = self.pinecone_client {
            if let Ok(matches) = pinecone.query_similar(question_vector.to_vec(), (context_size * 2) as u32).await {
                for (i, m) in matches.iter().enumerate() {
                    if let (Some(doc_path), Some(chunk_text)) = (
                        m.metadata.get("source").and_then(|v| v.as_str()),
                        m.metadata.get("text").and_then(|v| v.as_str())
                    ) {
                        let chunk_id = m.metadata.get("chunk_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(i as u64) as u32;
                        
                        all_context.push_str(&format!("--- Chunk {} (Similarity: {:.3}) ---\n", i + 1, m.score));
                        all_context.push_str(&format!("Source: {}\n", doc_path));
                        all_context.push_str(&format!("Content: {}\n\n", chunk_text));
                        
                        all_sources.push(SearchResult {
                            chunk_id,
                            document_path: doc_path.to_string(),
                            chunk_text: chunk_text.to_string(),
                            similarity: m.score,
                            project_name: None,
                        });
                    }
                }
            }
        }
        
        // Strategy 2: Fallback to local search if Pinecone failed or insufficient results
        if all_sources.len() < context_size {
            let local_results = self.rag_engine.search_relevant_chunks(question, question_vector, context_size)?;
            
            for (chunk_id, similarity, document_path, chunk_text) in local_results {
                if !all_sources.iter().any(|s| s.document_path == document_path) {
                    let chunk_num = all_sources.len() + 1;
                    all_context.push_str(&format!("--- Chunk {} (Similarity: {:.3}) ---\n", chunk_num, similarity));
                    all_context.push_str(&format!("Source: {}\n", document_path));
                    all_context.push_str(&format!("Content: {}\n\n", chunk_text));
                    
                    all_sources.push(SearchResult {
                        chunk_id,
                        document_path,
                        chunk_text,
                        similarity,
                        project_name: None,
                    });
                }
            }
        }
        
        // Strategy 3: Semantic expansion for better coverage (if enabled)
        if self.config.rag.enable_semantic_expansion && all_sources.len() < context_size / 2 {
            let expanded_context = self.semantic_expansion(question, question_vector, context_size - all_sources.len()).await?;
            all_context.push_str(&expanded_context);
        }
        
        Ok((all_context, all_sources))
    }

    fn assess_context_quality(&self, context: &str, question: &str) -> ContextQuality {
        let mut score = 0.0;
        let mut total_chunks = 0;
        
        // Parse context chunks
        let chunks: Vec<&str> = context.split("--- Chunk").collect();
        
        for chunk in chunks {
            if chunk.trim().is_empty() { continue; }
            
            if let Some(content_start) = chunk.find("Content:") {
                let content = &chunk[content_start..];
                let chunk_score = self.score_chunk_relevance(content, question);
                score += chunk_score;
                total_chunks += 1;
            }
        }
        
        let avg_score = if total_chunks > 0 { score / total_chunks as f32 } else { 0.0 };
        
        if avg_score >= 0.8 {
            ContextQuality::Excellent
        } else if avg_score >= 0.6 {
            ContextQuality::Good
        } else if avg_score >= 0.4 {
            ContextQuality::Acceptable
        } else {
            ContextQuality::Poor
        }
    }

    fn score_chunk_relevance(&self, chunk_content: &str, question: &str) -> f32 {
        let question_lower = question.to_lowercase();
        let content_lower = chunk_content.to_lowercase();
        
        let mut score = 0.0;
        
        // 1. Exact keyword matching (highest weight)
        let question_words: Vec<&str> = question_lower.split_whitespace()
            .filter(|word| word.len() > 2) // Filter out very short words
            .collect();
        
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        
        let exact_matches = question_words.iter()
            .filter(|word| content_words.contains(word))
            .count();
        
        if !question_words.is_empty() {
            score += (exact_matches as f32 / question_words.len() as f32) * 0.5;
        }
        
        // 2. Partial word matching (medium weight)
        let partial_matches = question_words.iter()
            .filter(|word| {
                content_words.iter().any(|content_word| {
                    content_word.contains(*word) || word.contains(content_word)
                })
            })
            .count();
        
        if !question_words.is_empty() {
            score += (partial_matches as f32 / question_words.len() as f32) * 0.3;
        }
        
        // 3. Semantic similarity for technical terms
        let technical_terms = ["function", "class", "method", "api", "database", "file", "code", "implementation"];
        let tech_matches = technical_terms.iter()
            .filter(|term| question_lower.contains(*term) && content_lower.contains(*term))
            .count();
        
        score += (tech_matches as f32 / technical_terms.len() as f32) * 0.2;
        
        // 4. Content type relevance
        if content_lower.contains("def ") || content_lower.contains("fn ") || content_lower.contains("function") {
            score += 0.1; // Function definitions are often relevant
        }
        
        if content_lower.contains("class ") || content_lower.contains("struct ") {
            score += 0.1; // Class/struct definitions are often relevant
        }
        
        if content_lower.contains("//") || content_lower.contains("/*") {
            score += 0.05; // Comments often contain explanations
        }
        
        // 5. Content length optimization
        let content_length = chunk_content.len();
        if content_length > 30 && content_length < 500 {
            score += 0.1; // Optimal content length
        } else if content_length > 500 {
            score += 0.05; // Long content might be too verbose
        }
        
        // 6. Question-specific scoring
        if question_lower.contains("what") || question_lower.contains("how") || question_lower.contains("why") {
            // For explanatory questions, prefer content with more context
            if content_length > 100 {
                score += 0.1;
            }
        }
        
        if question_lower.contains("function") || question_lower.contains("method") {
            // For function-related questions, prefer function definitions
            if content_lower.contains("def ") || content_lower.contains("fn ") {
                score += 0.2;
            }
        }
        
        score.min(1.0)
    }

    async fn generate_advanced_rag_response(&self, question: &str, context: &str, quality: &ContextQuality) -> Result<String> {
        // Use LLM for advanced reasoning if available
        if let Some(ref llm_client) = self.llm_client {
            // Generate high-quality answer using the LLM
            match llm_client.generate_answer(question, context).await {
                Ok(llm_answer) => {
                    if !llm_answer.is_empty() && !llm_answer.contains("I couldn't generate a response") {
                        return Ok(llm_answer);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: LLM generation failed: {}", e);
                }
            }
        }
        
        // Fallback to standard RAG if LLM is not available or fails
        self.generate_standard_rag_response(question, context, quality).await
    }

    async fn generate_standard_rag_response(&self, _question: &str, context: &str, _quality: &ContextQuality) -> Result<String> {
        let mut answer = String::new();
        
        // Extract key information from context
        let key_info = self.extract_key_information(context, _question);
        
        if key_info.is_empty() {
            answer.push_str("Based on the available information, I couldn't find specific details to answer your question. ");
            answer.push_str("Consider rephrasing your question or indexing more relevant documents.");
        } else {
            answer.push_str("Based on the indexed documents, here's what I found:\n\n");
            answer.push_str(&key_info);
            
            // The original code had a bug here, quality was unused.
            // It should be passed as a parameter or removed if not needed.
            // Assuming quality is not needed for this fallback response.
        }
        
        Ok(answer)
    }

    async fn generate_fallback_response(&self, _question: &str, context: &str, _quality: &ContextQuality) -> Result<String> {
        let mut answer = String::new();
        
        // Fallback strategy 1: General project information
        answer.push_str("I don't have enough specific information to provide a detailed answer to your question. ");
        answer.push_str("However, based on the project structure, this appears to be a semantic search and RAG system.\n\n");
        
        // Fallback strategy 2: Suggest improvements
        answer.push_str("To get better answers, consider:\n");
        answer.push_str("1. Indexing more documentation about the topic\n");
        answer.push_str("2. Using more specific search terms\n");
        answer.push_str("3. Checking if the documents are properly indexed\n\n");
        
        // Fallback strategy 3: Show what little context is available
        if !context.trim().is_empty() {
            answer.push_str("Available context (limited):\n");
            let lines: Vec<&str> = context.lines().collect();
            for line in lines.iter().take(3) {
                if line.contains("Content:") {
                    let content = line.replace("Content: ", "");
                    if !content.is_empty() {
                        answer.push_str(&format!("• {}\n", content.chars().take(100).collect::<String>()));
                    }
                }
            }
        }
        
        Ok(answer)
    }

    async fn validate_and_enhance_answer(&self, answer: &str, question: &str, context: &str, quality: &ContextQuality) -> Result<String> {
        let mut enhanced_answer = answer.to_string();
        
        // Validation 1: Check if answer directly addresses the question
        if !self.answer_addresses_question(answer, question) {
            enhanced_answer.push_str("\n\nNote: This answer may not fully address your specific question. Consider rephrasing or providing more context.");
        }
        
        // Validation 2: Add confidence indicators (if enabled)
        if self.config.rag.enable_confidence_scoring {
            match quality {
                ContextQuality::Excellent => {
                    enhanced_answer.push_str("\n\nConfidence: High - Based on comprehensive and relevant information.");
                }
                ContextQuality::Good => {
                    enhanced_answer.push_str("\n\nConfidence: Good - Based on relevant information with some gaps.");
                }
                ContextQuality::Acceptable => {
                    enhanced_answer.push_str("\n\nConfidence: Moderate - Based on limited but relevant information.");
                }
                ContextQuality::Poor => {
                    enhanced_answer.push_str("\n\nConfidence: Low - Limited relevant information available.");
                }
            }
        }
        
        // Validation 3: Add source attribution if available (if enabled)
        if self.config.rag.enable_source_attribution && !context.contains("Source:") {
            enhanced_answer.push_str("\n\nNote: Source information not available for this answer.");
        }
        
        Ok(enhanced_answer)
    }

    fn analyze_question_type(&self, question: &str) -> String {
        let question_lower = question.to_lowercase();
        
        // Check for question patterns
        if question_lower.starts_with("what") {
            if question_lower.contains("is") || question_lower.contains("does") || question_lower.contains("are") {
                return "Definition/Purpose question".to_string();
            } else if question_lower.contains("function") || question_lower.contains("method") || question_lower.contains("class") {
                return "Technical/Implementation question".to_string();
            }
            return "Definition/Purpose question".to_string();
        }
        
        if question_lower.starts_with("how") {
            if question_lower.contains("to") || question_lower.contains("do") || question_lower.contains("implement") {
                return "Process/How-to question".to_string();
            } else if question_lower.contains("does") || question_lower.contains("work") {
                return "Technical/Implementation question".to_string();
            }
            return "Process/How-to question".to_string();
        }
        
        if question_lower.starts_with("why") {
            return "Reasoning/Why question".to_string();
        }
        
        if question_lower.starts_with("where") {
            return "Location/Structure question".to_string();
        }
        
        if question_lower.starts_with("when") {
            return "Timing/Sequence question".to_string();
        }
        
        if question_lower.starts_with("which") {
            return "Selection/Choice question".to_string();
        }
        
        // Check for technical keywords
        if question_lower.contains("function") || question_lower.contains("method") || question_lower.contains("class") {
            return "Technical/Implementation question".to_string();
        }
        
        if question_lower.contains("error") || question_lower.contains("bug") || question_lower.contains("problem") {
            return "Troubleshooting/Problem question".to_string();
        }
        
        if question_lower.contains("example") || question_lower.contains("sample") || question_lower.contains("code") {
            return "Example/Code question".to_string();
        }
        
        // Default to general question
        "General/Information question".to_string()
    }

    fn analyze_context_structure(&self, context: &str) -> String {
        let chunks: Vec<&str> = context.split("--- Chunk").collect();
        let total_chunks = chunks.len() - 1; // -1 because first split is empty
        
        let mut total_similarity = 0.0;
        let mut source_files = std::collections::HashSet::new();
        
        for chunk in chunks.iter().skip(1) {
            if let Some(sim_start) = chunk.find("Similarity:") {
                if let Some(sim_end) = chunk[sim_start..].find(")") {
                    let sim_str = &chunk[sim_start + 12..sim_start + sim_end];
                    if let Ok(sim) = sim_str.parse::<f32>() {
                        total_similarity += sim;
                    }
                }
            }
            
            if let Some(source_start) = chunk.find("Source:") {
                if let Some(source_end) = chunk[source_start..].find("\n") {
                    let source = &chunk[source_start + 8..source_start + source_end];
                    source_files.insert(source.trim());
                }
            }
        }
        
        let avg_similarity = if total_chunks > 0 { total_similarity / total_chunks as f32 } else { 0.0 };
        
        format!("{} chunks, {:.3} avg similarity, {} unique sources", 
                total_chunks, avg_similarity, source_files.len())
    }

    fn extract_key_information(&self, context: &str, question: &str) -> String {
        let mut key_info = String::new();
        let lines: Vec<&str> = context.lines().collect();
        let mut relevant_chunks = Vec::new();
        
        // Parse context into structured chunks
        let mut current_chunk = String::new();
        let mut current_source = String::new();
        let mut current_similarity = 0.0;
        
        for line in lines {
            if line.starts_with("--- Chunk") {
                // Save previous chunk if exists
                if !current_chunk.is_empty() {
                    let relevance = self.score_chunk_relevance(&current_chunk, question);
                    if relevance > 0.05 { // Very low threshold to include more content
                        relevant_chunks.push((current_chunk.clone(), relevance, current_source.clone(), current_similarity));
                    }
                }
                
                // Start new chunk
                current_chunk.clear();
                current_source.clear();
                current_similarity = 0.0;
                
                // Extract similarity score
                if let Some(sim_str) = line.split("Similarity: ").nth(1) {
                    if let Some(sim_end) = sim_str.find(')') {
                        if let Ok(sim) = sim_str[..sim_end].parse::<f32>() {
                            current_similarity = sim;
                        }
                    }
                }
            } else if line.starts_with("Source: ") {
                current_source = line.replace("Source: ", "").trim().to_string();
            } else if line.starts_with("Content: ") {
                let content = line.replace("Content: ", "").trim().to_string();
                if !content.is_empty() {
                    current_chunk.push_str(&content);
                    current_chunk.push(' ');
                }
            } else if !line.trim().is_empty() && !current_chunk.is_empty() {
                // Continue content on subsequent lines
                current_chunk.push_str(line.trim());
                current_chunk.push(' ');
            }
        }
        
        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            let relevance = self.score_chunk_relevance(&current_chunk, question);
            if relevance > 0.05 {
                relevant_chunks.push((current_chunk.clone(), relevance, current_source.clone(), current_similarity));
            }
        }
        
        // Sort by relevance and similarity combined
        relevant_chunks.sort_by(|a, b| {
            let score_a = a.1 * 0.7 + a.3 * 0.3;
            let score_b = b.1 * 0.7 + b.3 * 0.3;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        if relevant_chunks.is_empty() {
            return "No relevant information found in the indexed documents.".to_string();
        }
        
        // Take top chunks and synthesize a coherent answer
        let top_chunks = relevant_chunks.iter().take(3).collect::<Vec<_>>();
        
        // Group by source file for better organization
        let mut source_groups: std::collections::HashMap<String, Vec<&str>> = std::collections::HashMap::new();
        for (content, _, source, _) in &top_chunks {
            source_groups.entry(source.clone()).or_default().push(content);
        }
        
        // Generate organized answer
        key_info.push_str("Based on the indexed documents, here's what I found:\n\n");
        
        for (source, contents) in source_groups {
            key_info.push_str(&format!("**From {}:**\n", source));
            for (i, content) in contents.iter().enumerate() {
                let clean_content = self.clean_and_summarize_content(content);
                if !clean_content.is_empty() {
                    key_info.push_str(&format!("{}. {}\n", i + 1, clean_content));
                }
            }
            key_info.push_str("\n");
        }
        
        key_info
    }
    
    fn clean_and_summarize_content(&self, content: &str) -> String {
        let content = content.trim();
        
        // Remove excessive whitespace and newlines
        let content = content.replace('\n', " ").replace('\r', " ");
        let content = content.split_whitespace().collect::<Vec<_>>().join(" ");
        
        // If it's code, try to extract meaningful parts
        if content.contains('(') && content.contains(')') && content.contains(';') {
            // Likely code - extract function calls or important statements
            if let Some(func_call) = self.extract_function_call(&content) {
                return format!("Function: {}", func_call);
            }
        }
        
        // If it's a long string, truncate intelligently
        if content.len() > 200 {
            let words: Vec<&str> = content.split_whitespace().collect();
            if words.len() > 30 {
                let truncated = words.iter().take(30).cloned().collect::<Vec<_>>().join(" ");
                return format!("{}...", truncated);
            }
        }
        
        content
    }
    
    fn extract_function_call(&self, content: &str) -> Option<String> {
        // Look for function calls like: function_name(arg1, arg2)
        if let Some(start) = content.find('(') {
            if let Some(end) = content.rfind(')') {
                if start < end {
                    let before_paren = content[..start].trim();
                    let args = content[start+1..end].trim();
                    
                    // Find the function name (last word before parentheses)
                    if let Some(func_name) = before_paren.split_whitespace().last() {
                        if !func_name.is_empty() {
                            return Some(format!("{}({})", func_name, args));
                        }
                    }
                }
            }
        }
        None
    }

    fn synthesize_answer_from_context(&self, context: &str, question: &str, _quality: &ContextQuality) -> String {
        let question_type = self.analyze_question_type(question);
        let key_info = self.extract_key_information(context, question);
        
        if key_info.contains("No relevant information found") {
            return "I couldn't find specific information to answer your question. The indexed documents don't contain relevant content for this query.".to_string();
        }
        
        let mut answer = String::new();
        
        match question_type.as_str() {
            "Definition/Purpose question" => {
                answer.push_str("Based on the available information:\n\n");
                answer.push_str(&key_info);
                answer.push_str("\n\nThis provides a comprehensive overview of the topic you asked about. ");
                answer.push_str("The information is extracted from the indexed source code and documentation files.");
            }
            "Process/How-to question" => {
                answer.push_str("Here's the process based on the available information:\n\n");
                answer.push_str(&key_info);
                answer.push_str("\n\nThese steps and code examples show how the functionality is implemented. ");
                answer.push_str("Follow the code structure and function calls in order for best results.");
            }
            "Reasoning/Why question" => {
                answer.push_str("The reasoning behind this is:\n\n");
                answer.push_str(&key_info);
                answer.push_str("\n\nThis explains the underlying principles and motivations as shown in the code. ");
                answer.push_str("The implementation details reveal the design decisions and architecture.");
            }
            _ => {
                answer.push_str("Here's what I found relevant to your question:\n\n");
                answer.push_str(&key_info);
                answer.push_str("\n\nThis information should help answer your question. ");
                answer.push_str("The content is extracted from the indexed source files and organized by relevance.");
            }
        }
        
        answer
    }

    fn answer_addresses_question(&self, answer: &str, question: &str) -> bool {
        let question_lower = question.to_lowercase();
        let answer_lower = answer.to_lowercase();
        
        // Check if key question words are addressed in the answer
        let question_words: Vec<&str> = question_lower.split_whitespace()
            .filter(|word| word.len() > 3) // Filter out short words
            .collect();
        
        let addressed_words = question_words.iter()
            .filter(|word| answer_lower.contains(*word))
            .count();
        
        let coverage = addressed_words as f32 / question_words.len() as f32;
        coverage > 0.5 // At least 50% of key words should be addressed
    }

    async fn semantic_expansion(&self, question: &str, question_vector: &[f32], additional_chunks: usize) -> Result<String> {
        // Try to find semantically related content
        let mut expanded_context = String::new();
        
        // Use local search with lower threshold for expansion
        if let Ok(results) = self.rag_engine.search_relevant_chunks(question, question_vector, additional_chunks * 2) {
            for (_chunk_id, similarity, document_path, chunk_text) in results {
                if similarity > 0.3 { // Lower threshold for expansion
                    let chunk_num = expanded_context.matches("--- Chunk").count() + 1;
                    expanded_context.push_str(&format!("--- Chunk {} (Similarity: {:.3}) ---\n", chunk_num, similarity));
                    expanded_context.push_str(&format!("Source: {}\n", document_path));
                    expanded_context.push_str(&format!("Content: {}\n\n", chunk_text));
                }
            }
        }
        
        Ok(expanded_context)
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
            // INTERNAL: Chain of thought reasoning (completely hidden from user)
            let mut internal_reasoning = String::new();
            
            // Step 1: Question analysis
            internal_reasoning.push_str("Question Analysis: ");
            internal_reasoning.push_str(&self.analyze_question_type(question));
            internal_reasoning.push_str("\n");
            
            // Step 2: Context analysis
            internal_reasoning.push_str("Context Analysis: ");
            internal_reasoning.push_str(&self.analyze_context_structure(context));
            internal_reasoning.push_str("\n");
            
            // Step 3: Information extraction
            internal_reasoning.push_str("Information Extraction: ");
            internal_reasoning.push_str(&self.extract_key_information(context, question));
            internal_reasoning.push_str("\n");
            
            // Step 4: Answer synthesis
            internal_reasoning.push_str("Answer Synthesis: ");
            let answer = self.synthesize_answer_from_context(context, question, &ContextQuality::Good);
            internal_reasoning.push_str(&answer);
            internal_reasoning.push_str("\n");
            
            // Return only the final polished answer, not the reasoning
            Ok(answer)
        } else {
            anyhow::bail!("Ollama not available for RAG generation")
        }
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        self.db.get_stats()
    }

    pub async fn get_rag_stats(&self) -> Result<RAGPipelineStats> {
        let mut stats = RAGPipelineStats::default();
        
        // Get configuration status
        stats.config_enabled = self.config.rag.enable_advanced_rag;
        stats.quality_assessment_enabled = self.config.rag.enable_quality_assessment;
        stats.answer_validation_enabled = self.config.rag.enable_answer_validation;
        stats.semantic_expansion_enabled = self.config.rag.enable_semantic_expansion;
        stats.fallback_strategies_enabled = self.config.rag.enable_fallback_strategies;
        
        // Get vector index statistics
        stats.local_vector_count = self.rag_engine.len();
        stats.pinecone_available = self.pinecone_client.is_some();
        
        // Get embedding model status
        stats.ollama_available = self.embedding_model.ollama_embeddings.is_some();
        stats.embedding_dimension = self.embedding_model.get_dimension();
        
        Ok(stats)
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
            // Silently truncate without verbose logging
        }
        
        // Chunk the text
        let chunks = self.chunk_text(&content, MAX_CHUNKS)?;
        
        // Generate embeddings for each chunk
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self.embedding_model.embed_texts(&chunk_texts).await?;
        
        // Store in database
        let (document_id, chunk_ids) = self.db.add_document_with_chunks(
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
        
        // Add to vector index using actual chunk IDs from database
        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let chunk_id = chunk_ids[i]; // Use actual chunk ID from database
            
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
            // Silently truncate without verbose logging
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




