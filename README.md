# 🐒 ChunkyMonkey - Semantic Search & RAG Made Simple & Fun! 🍌

**ChunkyMonkey** is a high-performance, Rust-based semantic search and Retrieval-Augmented Generation (RAG) system that transforms how you interact with your documents. It's designed to be both powerful and user-friendly, making advanced AI capabilities accessible to everyone.

## 🎯 What is ChunkyMonkey?

ChunkyMonkey is a **document intelligence platform** that:

- **🔍 Indexes** your documents (code, docs, text files) into searchable chunks
- **🧠 Generates** vector embeddings using state-of-the-art models
- **🔎 Performs** semantic search to find relevant content by meaning
- **🤖 Answers** questions using advanced RAG with multiple fallback strategies
- **📊 Provides** intelligent context assessment and quality scoring
- **⚡ Delivers** high-accuracy responses through sophisticated reasoning

Think of it as having an AI research assistant that can instantly search through all your documents and provide intelligent, context-aware answers.

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ChunkyMonkey.git
cd ChunkyMonkey

# Build the project
cargo build --release

# Run the CLI
./cm start
```

### First Steps

1. **📁 Index Documents**: Add your files to make them searchable
2. **🔍 Search Content**: Find relevant information using semantic search
3. **❓ Ask Questions**: Get AI-powered answers using the RAG system
4. **📊 Monitor Performance**: Check system statistics and RAG pipeline status

## 🧠 How It Works: The RAG Pipeline Deep Dive

ChunkyMonkey's RAG pipeline is a sophisticated, multi-layered system that goes far beyond simple vector similarity. Here's how it transforms your questions into high-accuracy answers:

### 🔄 **Phase 1: Intelligent Context Retrieval**

The system doesn't just find similar vectors - it employs a **multi-strategy retrieval approach**:

#### **Strategy 1: Pinecone Vector Search (Primary)**

```rust
// High-dimensional vector similarity search (768 dimensions)
if let Some(ref pinecone) = self.pinecone_client {
    let matches = pinecone.query_similar(question_vector, context_size * 2).await?;
    // Process high-quality vector matches
}
```

#### **Strategy 2: Local Vector Search (Fallback)**

```rust
// Local RAG engine with configurable relevance threshold
let results = self.rag_engine.search_relevant_chunks(
    question,
    question_vector,
    context_size
)?;
```

#### **Strategy 3: Semantic Expansion (Enhanced Coverage)**

```rust
// Expand context when initial retrieval is insufficient
if all_sources.len() < context_size / 2 {
    let expanded_context = self.semantic_expansion(
        question,
        question_vector,
        context_size - all_sources.len()
    ).await?;
}
```

### 📊 **Phase 2: Advanced Context Quality Assessment**

This is where ChunkyMonkey truly shines. The system doesn't just retrieve context - it **intelligently evaluates** it:

#### **Multi-Dimensional Scoring Algorithm**

```rust
fn score_chunk_relevance(&self, chunk_content: &str, question: &str) -> f32 {
    let mut score = 0.0;

    // 1. Exact keyword matching (50% weight)
    let exact_matches = question_words.iter()
        .filter(|word| content_words.contains(word))
        .count();
    score += (exact_matches as f32 / question_words.len() as f32) * 0.5;

    // 2. Partial word matching (30% weight)
    let partial_matches = question_words.iter()
        .filter(|word| content_words.iter().any(|cw|
            cw.contains(*word) || word.contains(cw)
        )).count();
    score += (partial_matches as f32 / question_words.len() as f32) * 0.3;

    // 3. Technical term relevance (20% weight)
    let technical_terms = ["function", "class", "method", "api", "database"];
    let tech_matches = technical_terms.iter()
        .filter(|term| question_lower.contains(*term) && content_lower.contains(*term))
        .count();
    score += (tech_matches as f32 / technical_terms.len() as f32) * 0.2;

    // 4. Content type optimization
    if content_lower.contains("def ") || content_lower.contains("fn ") {
        score += 0.1; // Function definitions are highly relevant
    }

    // 5. Content length optimization
    if content_length > 30 && content_length < 500 {
        score += 0.1; // Optimal content length
    }

    // 6. Question-specific scoring
    if question_lower.contains("how") && content_length > 100 {
        score += 0.1; // Process questions need more context
    }

    score.min(1.0)
}
```

#### **Context Quality Classification**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ContextQuality {
    Excellent,  // Score >= 0.8: Comprehensive, highly relevant
    Good,       // Score >= 0.6: Relevant with good coverage
    Acceptable, // Score >= 0.4: Some relevant information
    Poor        // Score < 0.4: Limited relevant information
}
```

### 🧠 **Phase 3: Multi-Strategy Answer Generation**

Based on context quality, the system automatically selects the optimal generation strategy:

#### **Strategy A: Advanced RAG (High-Quality Context)**

```rust
async fn generate_advanced_rag_response(&self, question: &str, context: &str, quality: &ContextQuality) -> Result<String> {
    if let Some(ref llm_client) = self.llm_client {
        // Use LLM for sophisticated reasoning
        let prompt = format!(
            "You are a helpful AI assistant. Based on the following context, provide a clear and concise answer to the question.\n\nQuestion: {}\n\nContext:\n{}\n\nAnswer:",
            question, context
        );

        match llm_client.generate_answer(question, context).await {
            Ok(llm_answer) => {
                if !llm_answer.is_empty() && !llm_answer.contains("I couldn't generate a response") {
                    return Ok(llm_answer);
                }
            }
            Err(e) => eprintln!("Warning: LLM generation failed: {}", e),
        }
    }

    // Fallback to standard RAG if LLM fails
    self.generate_standard_rag_response(question, context, quality).await
}
```

#### **Strategy B: Standard RAG (Acceptable Context)**

```rust
async fn generate_standard_rag_response(&self, _question: &str, context: &str, _quality: &ContextQuality) -> Result<String> {
    // Extract and synthesize key information
    let key_info = self.extract_key_information(context, _question);

    if key_info.is_empty() {
        return Ok("Based on the available information, I couldn't find specific details to answer your question. Consider rephrasing or indexing more relevant documents.".to_string());
    }

    Ok(format!("Based on the indexed documents, here's what I found:\n\n{}", key_info))
}
```

#### **Strategy C: Fallback RAG (Poor Context)**

```rust
async fn generate_fallback_response(&self, _question: &str, context: &str, _quality: &ContextQuality) -> Result<String> {
    let mut answer = String::new();

    // Provide system overview and improvement suggestions
    answer.push_str("I don't have enough specific information to provide a detailed answer. ");
    answer.push_str("However, this appears to be a semantic search and RAG system.\n\n");

    answer.push_str("To get better answers, consider:\n");
    answer.push_str("1. Indexing more documentation about the topic\n");
    answer.push_str("2. Using more specific search terms\n");
    answer.push_str("3. Checking if the documents are properly indexed\n\n");

    // Show available context (even if limited)
    if !context.trim().is_empty() {
        answer.push_str("Available context (limited):\n");
        // Process and display what little context exists
    }

    Ok(answer)
}
```

### ✅ **Phase 4: Answer Validation & Enhancement**

The final phase ensures answer quality and provides user confidence:

#### **Question Coverage Validation**

```rust
fn answer_addresses_question(&self, answer: &str, question: &str) -> bool {
    let question_words: Vec<&str> = question_lower.split_whitespace()
        .filter(|word| word.len() > 3) // Filter out short words
        .collect();

    let addressed_words = question_words.iter()
        .filter(|word| answer_lower.contains(*word))
        .count();

    let coverage = addressed_words as f32 / question_words.len() as f32;
    coverage > 0.5 // At least 50% of key words should be addressed
}
```

#### **Confidence Scoring & Attribution**

```rust
async fn validate_and_enhance_answer(&self, answer: &str, question: &str, context: &str, quality: &ContextQuality) -> Result<String> {
    let mut enhanced_answer = answer.to_string();

    // Add confidence indicators based on context quality
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

    // Add source attribution when available
    if self.config.rag.enable_source_attribution && !context.contains("Source:") {
        enhanced_answer.push_str("\n\nNote: Source information not available for this answer.");
    }

    Ok(enhanced_answer)
}
```

## ⚙️ **Advanced Configuration & Tuning**

### **RAG Pipeline Configuration**

```toml
[rag]
# Enable advanced RAG with LLM reasoning
enable_advanced_rag = true

# Enable context quality assessment
enable_quality_assessment = true

# Enable answer validation and enhancement
enable_answer_validation = true

# Enable semantic expansion for better coverage
enable_semantic_expansion = true

# Enable multiple fallback strategies
enable_fallback_strategies = true

# Enable confidence scoring in answers
enable_confidence_scoring = true

# Enable source attribution
enable_source_attribution = true

# Maximum context chunks to retrieve
max_context_chunks = 15

# Relevance threshold for local search
relevance_threshold = 0.1
```

### **Embedding Model Configuration**

```toml
[ollama]
base_url = "http://localhost:11434"
embedding_model = "nomic-embed-text"
llm_model = "llama2:7b"

[pinecone]
api_key = "your-api-key"
environment = "your-environment"
index_name = "your-index-name"
```

## 📊 **Performance Monitoring & Analytics**

### **RAG Pipeline Statistics**

```bash
./cm rag-stats
```

Shows:

- **Feature Status**: Which RAG features are enabled
- **System Availability**: Ollama, Pinecone, local vector status
- **Vector Metrics**: Local vector count, embedding dimensions
- **Quality Metrics**: Context assessment performance

### **Database Statistics**

```bash
./cm stats
```

Shows:

- **Document Count**: Total indexed documents
- **Chunk Count**: Total text chunks
- **Database Size**: Storage usage
- **Index Performance**: Search and retrieval metrics

## 🏗️ **System Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Query    │───▶│  Vector Search  │───▶│ Context Quality │
│                 │    │  (Multi-Strategy)│   │  Assessment     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Multi-Strategy  │    │ Quality-Based   │
                       │ Context Retrieval│   │ Strategy Selection│
                       └─────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Answer Generation│   │ Validation &    │
                       │ (LLM/Standard/  │   │ Enhancement     │
                       │  Fallback)      │   │                 │
                       └─────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                       ┌─────────────────┐
                       │  Polished,      │
                       │ High-Confidence │
                       │     Answer      │
                       └─────────────────┘
```

## 🚀 **Performance Optimization Tips**

### **1. Vector Search Optimization**

- **Use Pinecone** for large-scale deployments (>10k documents)
- **Optimize chunk sizes** based on content type (code: 500-1000 chars, docs: 1000-2000 chars)
- **Tune relevance thresholds** for your use case (0.1 for broad, 0.7 for precise)

### **2. Context Quality Tuning**

- **Index diverse content** to improve context coverage
- **Use descriptive filenames** for better source attribution
- **Monitor quality scores** to identify indexing gaps

### **3. LLM Integration**

- **Local Ollama models** for privacy and speed
- **Cloud models** for higher accuracy and reasoning
- **Model selection** based on your content domain

## 🔍 **Use Cases & Applications**

### **Code Documentation & Search**

- **API Documentation**: Find relevant functions and examples
- **Code Reviews**: Understand implementation details
- **Bug Investigation**: Trace issues through codebase

### **Knowledge Management**

- **Research Papers**: Semantic search through academic content
- **Technical Documentation**: Find relevant procedures and concepts
- **Company Knowledge**: Search through internal documents

### **Content Creation**

- **Writing Assistance**: Find relevant information for articles
- **Research Support**: Gather context for reports
- **Learning Aid**: Understand complex topics through examples

## 🤝 **Contributing**

We welcome contributions! ChunkyMonkey is built with Rust for performance and reliability. Areas for contribution:

- **RAG Pipeline Enhancements**: New context assessment algorithms
- **Vector Search Optimization**: Improved similarity metrics
- **UI/UX Improvements**: Better user experience
- **Performance Tuning**: Optimization and benchmarking
- **Documentation**: Examples and tutorials

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Rust Community**: For the amazing language and ecosystem
- **Ollama Team**: For local AI capabilities
- **Pinecone**: For scalable vector search infrastructure
- **Open Source Community**: For inspiration and collaboration

---

**🐒 Going Bananas for Chunks since 2024! 🍌**

_ChunkyMonkey: Where semantic search meets intelligent reasoning, delivering high-accuracy answers through advanced RAG technology._
