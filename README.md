# 🐒 ChunkyMonkey - Going Bananas for Chunks! 🍌

A powerful semantic search and Retrieval-Augmented Generation (RAG) system that helps you organize, index, and query your documents with AI-powered intelligence.

## ✨ Features

- **🗂️ Project Management**: Organize documents into logical projects
- **🔍 Semantic Search**: Find content by meaning, not just keywords
- **❓ AI-Powered Q&A**: Get intelligent answers using RAG technology
- **🚀 Multiple Embedding Models**: Support for Ollama and local models
- **🌲 Vector Database Integration**: Optional Pinecone integration for scale
- **⚙️ Configurable Pipeline**: Customize the RAG pipeline behavior

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ChunkyMonkey.git
cd ChunkyMonkey

# Build the project
cargo build --release

# Run the interactive CLI
./cm
```

### First Steps

1. **Create a Project**: Organize your documents into logical groups
2. **Index Documents**: Add files to your project for searching
3. **Ask Questions**: Use the RAG system to get intelligent answers
4. **Monitor Performance**: Check RAG pipeline statistics

## 🤖 Fortified RAG Pipeline

ChunkyMonkey features a **fortified RAG pipeline** that provides robust, high-quality answers through multiple layers of intelligence and fallback strategies.

### 🔒 Key Features

#### **Hidden Chain of Thought Reasoning**

- Advanced reasoning happens internally using Ollama
- Users see only the final, polished answer
- No verbose "thinking" output cluttering the interface

#### **Multi-Strategy Context Retrieval**

1. **Primary Strategy**: Pinecone vector search (if available)
2. **Fallback Strategy**: Local vector search
3. **Expansion Strategy**: Semantic expansion for better coverage

#### **Context Quality Assessment**

- Automatically evaluates retrieved context relevance
- Scores context from Poor → Acceptable → Good → Excellent
- Adjusts answer generation strategy based on quality

#### **Intelligent Answer Generation**

- **Advanced RAG**: Full chain-of-thought reasoning (high-quality context)
- **Standard RAG**: Structured information extraction (acceptable context)
- **Fallback RAG**: Multiple strategies for poor context
- **Simple RAG**: Basic information extraction (minimal context)

#### **Answer Validation & Enhancement**

- Validates if answer addresses the question
- Adds confidence indicators based on context quality
- Provides source attribution when available
- Suggests improvements for better results

### ⚙️ Configuration

The RAG pipeline is highly configurable through `config.toml`:

```toml
[rag]
# Enable advanced RAG with hidden chain-of-thought reasoning
enable_advanced_rag = true

# Enable context quality assessment
enable_quality_assessment = true

# Enable answer validation and enhancement
enable_answer_validation = true

# Enable semantic expansion for better context coverage
enable_semantic_expansion = true

# Enable multiple fallback strategies
enable_fallback_strategies = true

# Minimum context quality threshold (0.0 to 1.0)
min_quality_threshold = 0.6

# Maximum context chunks to retrieve
max_context_chunks = 15

# Enable confidence scoring in answers
enable_confidence_scoring = true

# Enable source attribution
enable_source_attribution = true
```

### 📊 Monitoring

Monitor your RAG pipeline performance:

```bash
# CLI command
./cm rag-stats

# Interactive mode
# Select option 6: 🤖 RAG Pipeline Stats
```

This shows:

- Feature enablement status
- System availability (Ollama, Pinecone)
- Vector index statistics
- Configuration status

### 🛡️ Fallback Strategies

When context quality is poor, the system automatically employs:

1. **General Project Information**: Provide basic system overview
2. **Improvement Suggestions**: Guide users to better results
3. **Available Context**: Show what limited information exists
4. **Alternative Approaches**: Suggest rephrasing or additional indexing

### 🎯 Question Type Intelligence

The system automatically detects question types and tailors responses:

- **Definition/Purpose**: Comprehensive overview with key points
- **Process/How-to**: Step-by-step instructions
- **Reasoning/Why**: Underlying principles and motivations
- **General**: Relevant information with context

## 📁 Usage Examples

### Basic Search

```bash
./cm search "machine learning algorithms"
```

### Ask Questions

```bash
./cm ask "What is the purpose of this project?"
```

### Index Documents

```bash
./cm index /path/to/documents
```

### View Statistics

```bash
./cm stats
./cm rag-stats
```

## 🔧 Configuration

### Environment Variables

```bash
export OLLAMA_BASE_URL="http://localhost:11434"
export OLLAMA_MODEL="llama3"
export PINECONE_API_KEY="your-api-key"
export PINECONE_ENVIRONMENT="your-environment"
export PINECONE_INDEX="your-index-name"
```

### Configuration File

Copy `config.toml.example` to `config.toml` and customize:

```bash
cp config.toml.example config.toml
# Edit config.toml with your settings
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Query    │───▶│  RAG Pipeline   │───▶│  Polished      │
│                 │    │                 │    │   Answer       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │ Context Quality │
                       │  Assessment     │
                       └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │ Multi-Strategy  │
                       │ Answer Gen.     │
                       └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │ Validation &    │
                       │ Enhancement     │
                       └─────────────────┘
```

## 🚀 Performance Tips

1. **Use Pinecone**: For large-scale deployments, enable Pinecone integration
2. **Optimize Chunking**: Adjust chunk sizes based on your content type
3. **Quality Thresholds**: Fine-tune quality thresholds for your use case
4. **Monitor Stats**: Regularly check RAG pipeline statistics
5. **Index Strategically**: Organize documents into logical projects

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with Rust for performance and reliability
- Powered by Ollama for local AI capabilities
- Enhanced with Pinecone for scalable vector search
- Designed for simplicity and power

---

**🐒 Going Bananas for Chunks since 2024! 🍌**
