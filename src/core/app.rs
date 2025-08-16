use crate::db::Database;
use crate::embeddings::EmbeddingService;
use crate::pinecone::{PineconeClient, PineconeConfig, Vector};
use crate::core::types::*;
use crate::core::config::OllamaConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

const MAX_TEXT_SIZE: usize = 1 * 1024 * 1024; // 1MB

pub struct TldrApp {
    db: Database,
    embeddings: EmbeddingService,
    pinecone: PineconeClient,
}

impl TldrApp {
    pub async fn new(
        db_path: &str,
        ollama_config: OllamaConfig,
        pinecone_config: PineconeConfig,
    ) -> Result<Self> {
        let db = Database::new(db_path.into()).await?;
        let embeddings = EmbeddingService::new_with_ollama(ollama_config)?;
        let pinecone = if pinecone_config.api_key.is_empty() {
            PineconeClient::new_dummy()?
        } else {
            PineconeClient::new(pinecone_config)?
        };

        Ok(Self {
            db,
            embeddings,
            pinecone,
        })
    }

    pub async fn add_document(&mut self, file_path: &Path, content: &str) -> Result<DocumentStatus> {
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
        
        // Generate file hash
        let file_hash = self.generate_file_hash(content);
        
        // Check if document already exists
        if let Some(existing_hash) = self.db.get_document_hash(&file_path.to_path_buf()).await? {
            if existing_hash == file_hash {
                return Ok(DocumentStatus::Skipped);
            }
        }

        // Chunk the text
        let chunks = self.chunk_text(content)?;
        
        // Generate embeddings for chunks
        let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
        let embeddings = self.embeddings.embed_batch(chunk_texts).await?;

        // Store document and chunks in local SQLite
        let chunk_strings: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        self.db.add_document_with_chunks(&file_path.to_path_buf(), content, &chunk_strings, &embeddings).await?;

        // Store vectors in Pinecone (if available)
        if !self.pinecone.config.api_key.is_empty() {
            let mut vectors = Vec::new();
            for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
                let vector_id = format!("{}_{}", file_hash, i);
                let metadata = HashMap::from([
                    ("file_hash".to_string(), serde_json::Value::String(file_hash.clone())),
                    ("chunk_index".to_string(), serde_json::Value::Number(serde_json::Number::from(i))),
                    ("file_path".to_string(), serde_json::Value::String(file_path.to_string_lossy().to_string())),
                    ("text_preview".to_string(), serde_json::Value::String(chunk.text.chars().take(100).collect())),
                ]);

                vectors.push(Vector {
                    id: vector_id,
                    values: embedding.clone(),
                    metadata,
                });
            }

            self.pinecone.upsert_vectors(vectors).await?;
        }

        Ok(DocumentStatus::Added)
    }

    pub async fn search_similar(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embeddings.embed_text(query).await?;
        
        // Use a higher threshold for better quality results
        let base_threshold = 0.5; // Increased from 0.3
        
        // Try with base threshold first
        let mut results = self.db.search_similar(&query_embedding, limit * 2, base_threshold).await?;
        
        // If we don't have enough results, try with a slightly lower threshold
        if results.len() < limit && base_threshold > 0.4 {
            let lower_threshold = base_threshold - 0.1;
            let additional_results = self.db.search_similar(&query_embedding, limit, lower_threshold).await?;
            results.extend(additional_results);
        }
        
        // Remove duplicates based on file_path and chunk_index
        results.sort_by(|a, b| {
            (a.file_path.clone(), a.chunk_index).cmp(&(b.file_path.clone(), b.chunk_index))
        });
        results.dedup_by(|a, b| {
            a.file_path == b.file_path && a.chunk_index == b.chunk_index
        });
        
        // Sort by score and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    pub async fn ask_question(&self, question: &str, context_chunks: usize) -> Result<RAGAnswer> {
        // Step 1: Use advanced semantic search instead of basic similarity search
        let search_results = self.semantic_search(question, context_chunks * 2).await?;
        
        if search_results.is_empty() {
            anyhow::bail!("No relevant content found to answer the question");
        }
        
        // Step 2: Advanced reranking with multiple factors
        let reranked_results = self.rerank_results(question, search_results, context_chunks).await?;
        
        // Step 3: Content quality filtering
        let filtered_results = self.filter_by_content_quality(reranked_results).await?;
        
        if filtered_results.is_empty() {
            anyhow::bail!("No high-quality content found to answer the question");
        }
        
        // Step 4: Generate contextual answer
        let context: String = filtered_results
            .iter()
            .map(|r| r.text.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let answer = format!("Based on the relevant content:\n\n{}", context);
        let sources: Vec<SearchResult> = filtered_results.clone();
        
        // Calculate confidence based on result quality
        let confidence = self.calculate_confidence(&filtered_results);
        
        Ok(RAGAnswer {
            text: answer,
            sources,
            confidence,
        })
    }

    /// Expand the original question with related concepts and synonyms
    async fn expand_query(&self, question: &str) -> Result<Vec<String>> {
        let mut expanded = vec![question.to_string()];
        
        // Add question variations
        if question.to_lowercase().contains("how") {
            expanded.push(question.replace("how", "what").to_string());
            expanded.push(question.replace("how", "explain").to_string());
        }
        
        if question.to_lowercase().contains("what") {
            expanded.push(question.replace("what", "how").to_string());
            expanded.push(question.replace("what", "describe").to_string());
        }
        
        // Add authentication-specific expansions
        if question.to_lowercase().contains("auth") || question.to_lowercase().contains("authentication") {
            expanded.extend_from_slice(&[
                "login system".to_string(),
                "user verification".to_string(),
                "security protocols".to_string(),
                "access control".to_string(),
                "identity management".to_string(),
            ]);
        }
        
        // Add feature-specific expansions
        if question.to_lowercase().contains("feature") {
            expanded.extend_from_slice(&[
                "capabilities".to_string(),
                "functionality".to_string(),
                "what can it do".to_string(),
                "main functions".to_string(),
            ]);
        }
        
        Ok(expanded)
    }

    /// Advanced reranking using multiple factors beyond just similarity
    async fn rerank_results(
        &self,
        question: &str,
        mut results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Calculate additional scoring factors
        for result in &mut results {
            let mut score = result.score;
            
            // Boost score for exact keyword matches
            let question_lower = question.to_lowercase();
            let text_lower = result.text.to_lowercase();
            
            if question_lower.split_whitespace().any(|word| text_lower.contains(word)) {
                score += 0.1;
            }
            
            // Boost for longer, more informative chunks
            let length_boost = (result.text.len() as f32 / 1000.0).min(0.2);
            score += length_boost;
            
            // Penalize very short chunks that might be incomplete
            if result.text.len() < 100 {
                score -= 0.15;
            }
            
            // Boost for chunks that seem to contain complete thoughts
            if result.text.contains('.') && result.text.contains(' ') {
                score += 0.05;
            }
            
            // Penalize chunks that are mostly code or technical gibberish
            if self.is_low_quality_content(&result.text) {
                score -= 0.3;
            }
            
            result.score = score.max(0.0).min(1.0);
        }
        
        // Sort by new score and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    /// Filter results by content quality
    async fn filter_by_content_quality(&self, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        let mut filtered = Vec::new();
        
        for result in results {
            // Skip very low quality content
            if self.is_low_quality_content(&result.text) {
                continue;
            }
            
            // Skip chunks that are too short to be useful
            if result.text.len() < 50 {
                continue;
            }
            
            // Skip chunks that are mostly punctuation or whitespace
            let meaningful_chars = result.text.chars().filter(|c| c.is_alphanumeric()).count();
            if meaningful_chars < 30 {
                continue;
            }
            
            filtered.push(result);
        }
        
        Ok(filtered)
    }

    /// Check if content is low quality (code, gibberish, etc.)
    fn is_low_quality_content(&self, text: &str) -> bool {
        let text = text.trim();
        
        // Skip if too short
        if text.len() < 20 {
            return true;
        }
        
        // Skip if mostly special characters or code
        let special_char_ratio = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count() as f32 / text.len() as f32;
        if special_char_ratio > 0.7 {
            return true;
        }
        
        // Skip if it looks like version info or technical metadata
        let lower = text.to_lowercase();
        if lower.contains("version") && lower.contains("crate") {
            return true;
        }
        
        if lower.contains("protobuf") && lower.contains("symbol") {
            return true;
        }
        
        // Skip if it's mostly CLI argument definitions
        if text.contains("#[arg") && text.contains("value_name") {
            return true;
        }
        
        // Skip if it's mostly empty or whitespace
        if text.chars().filter(|c| c.is_whitespace()).count() > text.len() / 2 {
            return true;
        }
        
        false
    }

    /// Calculate confidence score based on result quality
    fn calculate_confidence(&self, results: &[SearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }
        
        let avg_score: f32 = results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32;
        let score_consistency = 1.0 - (results.iter().map(|r| (r.score - avg_score).abs()).sum::<f32>() / results.len() as f32);
        
        let length_quality: f32 = results.iter()
            .map(|r| (r.text.len() as f32 / 1000.0).min(1.0))
            .sum::<f32>() / results.len() as f32;
        
        (avg_score * 0.5 + score_consistency * 0.3 + length_quality * 0.2).max(0.0).min(1.0)
    }

    /// Advanced semantic search with better understanding of query intent
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Extract key concepts from the query
        let key_concepts = self.extract_key_concepts(query);
        
        // Generate multiple search queries based on key concepts
        let mut all_results = Vec::new();
        
        // Search with original query
        let original_results = self.search_similar(query, limit).await?;
        all_results.extend(original_results);
        
        // Search with key concepts
        for concept in &key_concepts {
            let concept_results = self.search_similar(concept, limit / 2).await?;
            all_results.extend(concept_results);
        }
        
        // Remove duplicates and rerank
        all_results.sort_by(|a, b| {
            (a.file_path.clone(), a.chunk_index).cmp(&(b.file_path.clone(), b.chunk_index))
        });
        all_results.dedup_by(|a, b| {
            a.file_path == b.file_path && a.chunk_index == b.chunk_index
        });
        
        // Apply semantic reranking
        let reranked = self.semantic_rerank(query, all_results, limit).await?;
        
        Ok(reranked)
    }

    /// Extract key concepts from a query for better search
    fn extract_key_concepts(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut concepts = Vec::new();
        
        // Extract technical terms
        let technical_terms = [
            "authentication", "auth", "login", "security", "encryption",
            "database", "api", "endpoint", "config", "setup",
            "feature", "functionality", "capability", "system", "architecture"
        ];
        
        for term in &technical_terms {
            if query_lower.contains(term) {
                concepts.push(term.to_string());
            }
        }
        
        // Extract action words
        let action_words = [
            "how", "what", "where", "when", "why", "explain", "describe",
            "work", "function", "operate", "implement", "configure"
        ];
        
        for word in &action_words {
            if query_lower.contains(word) {
                concepts.push(word.to_string());
            }
        }
        
        // If no specific concepts found, use the main words
        if concepts.is_empty() {
            let words: Vec<&str> = query_lower.split_whitespace()
                .filter(|w| w.len() > 2)
                .collect();
            concepts.extend(words.iter().take(3).map(|s| s.to_string()));
        }
        
        concepts
    }

    /// Semantic reranking based on content understanding
    async fn semantic_rerank(
        &self,
        query: &str,
        mut results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        
        for result in &mut results {
            let mut score = result.score;
            let text_lower = result.text.to_lowercase();
            
            // Boost for exact concept matches
            let key_concepts = self.extract_key_concepts(query);
            for concept in &key_concepts {
                if text_lower.contains(&concept.to_lowercase()) {
                    score += 0.15;
                }
            }
            
            // Boost for question-answer patterns
            if query_lower.contains("how") && text_lower.contains("by") {
                score += 0.1;
            }
            
            if query_lower.contains("what") && text_lower.contains("is") {
                score += 0.1;
            }
            
            // Boost for explanatory content
            if text_lower.contains("example") || text_lower.contains("explanation") {
                score += 0.1;
            }
            
            // Penalize overly technical or code-heavy content for general questions
            let code_ratio = text_lower.chars()
                .filter(|c| *c == '{' || *c == '}' || *c == '[' || *c == ']' || *c == '(' || *c == ')')
                .count() as f32 / text_lower.len() as f32;
            
            if code_ratio > 0.3 && !query_lower.contains("code") && !query_lower.contains("implementation") {
                score -= 0.2;
            }
            
            // Normalize score
            result.score = score.max(0.0).min(1.0);
        }
        
        // Sort by new score and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        self.db.get_stats().await
    }

    pub async fn clear_database(&mut self) -> Result<()> {
        self.db.clear_all().await?;
        Ok(())
    }
    
    /// Recreate the database schema (useful for fixing schema issues)
    pub async fn recreate_schema(&self) -> Result<()> {
        self.db.recreate_schema().await?;
        Ok(())
    }

    fn chunk_text(&self, text: &str) -> Result<Vec<Chunk>> {
        if text.len() > MAX_TEXT_SIZE {
            anyhow::bail!("Text too large for chunking: {} bytes (max: {} bytes)", text.len(), MAX_TEXT_SIZE);
        }
        self.chunk_text_internal(text)
    }

    fn chunk_text_internal(&self, text: &str) -> Result<Vec<Chunk>> {
        // Use semantic chunking instead of fixed-size chunks
        let mut chunks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut current_chunk = String::new();
        let mut chunk_index = 0;
        
        for line in lines {
            let line = line.trim();
            
            // Skip empty lines and very short lines
            if line.is_empty() || line.len() < 10 {
                continue;
            }
            
            // Check if this line starts a new logical section
            let starts_new_section = self.starts_new_section(line);
            
            // If we have a substantial chunk and this starts a new section, save the current chunk
            if current_chunk.len() > 200 && starts_new_section {
                if !current_chunk.trim().is_empty() {
                    chunks.push(Chunk {
                        id: 0,
                        document_id: 0,
                        text: current_chunk.trim().to_string(),
                        chunk_index,
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                    });
                    chunk_index += 1;
                }
                current_chunk.clear();
            }
            
            // Add line to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(line);
            
            // If chunk gets too long, split it
            if current_chunk.len() > 1500 {
                if !current_chunk.trim().is_empty() {
                    chunks.push(Chunk {
                        id: 0,
                        document_id: 0,
                        text: current_chunk.trim().to_string(),
                        chunk_index,
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                    });
                    chunk_index += 1;
                }
                current_chunk.clear();
            }
        }
        
        // Add the last chunk if it's not empty
        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                id: 0,
                document_id: 0,
                text: current_chunk.trim().to_string(),
                chunk_index,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            });
        }
        
        // If we still have no chunks, fall back to the old method
        if chunks.is_empty() {
            return self.fallback_chunking(text);
        }
        
        Ok(chunks)
    }

    /// Fallback to the original chunking method if semantic chunking fails
    fn fallback_chunking(&self, text: &str) -> Result<Vec<Chunk>> {
        let chunk_size = 1000;
        let overlap = 200;
        
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;
        
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let mut actual_end = end;
            
            if end < chars.len() {
                for i in (start..end).rev() {
                    if chars[i].is_whitespace() || chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
                        actual_end = i + 1;
                        break;
                    }
                }
            }
            
            let chunk_text: String = chars[start..actual_end].iter().collect();
            chunks.push(Chunk {
                id: 0,
                document_id: 0,
                text: chunk_text,
                chunk_index,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            });
            
            chunk_index += 1;
            start = if actual_end < chars.len() { actual_end.saturating_sub(overlap) } else { actual_end };
        }
        
        Ok(chunks)
    }

    /// Check if a line starts a new logical section
    fn starts_new_section(&self, line: &str) -> bool {
        let line = line.trim();
        
        // Check for common section markers
        if line.starts_with('#') || line.starts_with("##") || line.starts_with("###") {
            return true;
        }
        
        // Check for function/struct definitions
        if line.contains("fn ") || line.contains("struct ") || line.contains("impl ") || line.contains("pub ") {
            return true;
        }
        
        // Check for comment blocks
        if line.starts_with("///") || line.starts_with("//!") || line.starts_with("/*") {
            return true;
        }
        
        // Check for configuration sections
        if line.contains(":") && line.ends_with("{") {
            return true;
        }
        
        false
    }

    fn generate_file_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Analyze search results to understand why they were returned
    pub async fn analyze_search_results(&self, question: &str, results: &[SearchResult]) -> Result<String> {
        if results.is_empty() {
            return Ok("No search results to analyze.".to_string());
        }
        
        let mut analysis = String::new();
        analysis.push_str("🔍 Search Results Analysis:\n\n");
        
        // Analyze overall result quality
        let avg_score: f32 = results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32;
        let score_variance: f32 = results.iter()
            .map(|r| (r.score - avg_score).powi(2))
            .sum::<f32>() / results.len() as f32;
        let score_std = score_variance.sqrt();
        
        analysis.push_str(&format!("📊 Overall Quality Metrics:\n"));
        analysis.push_str(&format!("   • Average similarity score: {:.3}\n", avg_score));
        analysis.push_str(&format!("   • Score standard deviation: {:.3}\n", score_std));
        analysis.push_str(&format!("   • Number of results: {}\n\n", results.len()));
        
        // Analyze individual results
        analysis.push_str("📋 Individual Result Analysis:\n");
        for (i, result) in results.iter().enumerate() {
            analysis.push_str(&format!("\n{}. {} (Score: {:.3})\n", i + 1, result.file_path, result.score));
            
            // Analyze content quality
            let content_quality = self.analyze_content_quality(&result.text);
            analysis.push_str(&format!("   • Content quality: {}\n", content_quality));
            
            // Show relevance indicators
            let relevance = self.analyze_relevance_to_question(question, &result.text);
            analysis.push_str(&format!("   • Relevance indicators: {}\n", relevance));
            
            // Show text preview
            let preview = if result.text.len() > 100 {
                format!("{}...", &result.text[..100])
            } else {
                result.text.clone()
            };
            analysis.push_str(&format!("   • Preview: {}\n", preview));
        }
        
        // Provide recommendations
        analysis.push_str("\n💡 Recommendations:\n");
        if avg_score < 0.6 {
            analysis.push_str("   • Consider lowering similarity threshold for more results\n");
        }
        if score_std > 0.2 {
            analysis.push_str("   • High score variance suggests inconsistent result quality\n");
        }
        if results.iter().any(|r| r.text.len() < 100) {
            analysis.push_str("   • Some results are very short - consider increasing min chunk size\n");
        }
        
        Ok(analysis)
    }

    /// Analyze the quality of a text chunk
    fn analyze_content_quality(&self, text: &str) -> String {
        let meaningful_chars = text.chars().filter(|c| c.is_alphanumeric()).count();
        let meaningful_ratio = meaningful_chars as f32 / text.len() as f32;
        
        let has_complete_sentences = text.contains('.') && text.contains(' ');
        let has_structure = text.contains('\n') || text.contains('•') || text.contains('-');
        
        let mut quality_score = 0;
        let mut quality_desc = Vec::new();
        
        if meaningful_ratio > 0.7 {
            quality_score += 1;
            quality_desc.push("high meaningful content");
        }
        if has_complete_sentences {
            quality_score += 1;
            quality_desc.push("complete sentences");
        }
        if has_structure {
            quality_score += 1;
            quality_desc.push("good structure");
        }
        if text.len() > 200 {
            quality_score += 1;
            quality_desc.push("substantial length");
        }
        
        match quality_score {
            0..=1 => format!("Low ({})", quality_desc.join(", ")),
            2..=3 => format!("Medium ({})", quality_desc.join(", ")),
            _ => format!("High ({})", quality_desc.join(", ")),
        }
    }

    /// Analyze how relevant a text chunk is to the question
    fn analyze_relevance_to_question(&self, question: &str, text: &str) -> String {
        let question_lower = question.to_lowercase();
        let text_lower = text.to_lowercase();
        
        let mut relevance_indicators = Vec::new();
        
        // Check for exact keyword matches
        let question_words: Vec<&str> = question_lower.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        let mut keyword_matches = 0;
        for word in &question_words {
            if text_lower.contains(word) {
                keyword_matches += 1;
            }
        }
        
        if keyword_matches > 0 {
            relevance_indicators.push(format!("{} keyword matches", keyword_matches));
        }
        
        // Check for semantic concepts
        let key_concepts = self.extract_key_concepts(question);
        let mut concept_matches = 0;
        for concept in &key_concepts {
            if text_lower.contains(&concept.to_lowercase()) {
                concept_matches += 1;
            }
        }
        
        if concept_matches > 0 {
            relevance_indicators.push(format!("{} concept matches", concept_matches));
        }
        
        // Check for question-answer patterns
        if question_lower.contains("how") && text_lower.contains("by") {
            relevance_indicators.push("how-to pattern".to_string());
        }
        
        if question_lower.contains("what") && text_lower.contains("is") {
            relevance_indicators.push("definition pattern".to_string());
        }
        
        if relevance_indicators.is_empty() {
            "Limited relevance indicators".to_string()
        } else {
            relevance_indicators.join(", ")
        }
    }
}
