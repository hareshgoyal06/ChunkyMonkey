# 🔍 TLDR - Too Long; Didn't Read

**Blazing-fast semantic search through any directory using real vector embeddings and Pinecone.**

TLDR is a powerful command-line tool that helps you find and understand code, documentation, and text files instantly using AI-powered semantic search with **real vector embeddings** stored in **Pinecone**.

## ✨ Features

- 🧠 **Real Semantic Understanding** - Uses OpenAI embeddings for accurate semantic search
- 🚀 **Production Vector Database** - Pinecone integration for scalable vector storage
- 📁 **Smart File Indexing** - Automatically chunks and indexes your documents
- 🔍 **Semantic Search** - Find content by meaning, not just keywords
- ❓ **RAG (Retrieval-Augmented Generation)** - Ask questions about your content
- 💻 **Interactive CLI** - Beautiful, intuitive command-line interface
- 🎯 **Smart Chunking** - Unicode-aware text processing with word boundaries
- 📊 **Real-time Statistics** - Monitor your indexed content

## 🚀 Quick Start

### 1. **Prerequisites**

You'll need:

- **OpenAI API Key** - For generating embeddings
- **Pinecone Account** - For vector storage
- **Rust** - For building the tool

### 2. **Setup Pinecone**

1. Create a [Pinecone account](https://www.pinecone.io/)
2. Create a new index:
   - **Dimensions**: 1536 (for OpenAI text-embedding-ada-002)
   - **Metric**: Cosine
   - **Environment**: Choose your preferred region
3. Note your:
   - API Key
   - Environment
   - Index Name

### 3. **Install TLDR**

```bash
# Clone and build
git clone <your-repo>
cd ChunkyMonkey
cargo build --release

# Create symlink for easy access
sudo ln -s $(pwd)/target/release/tldr /usr/local/bin/tldr
```

### 4. **Configure API Keys**

**Option A: Environment Variables**

```bash
export OPENAI_API_KEY="your-openai-api-key"
export PINECONE_API_KEY="your-pinecone-api-key"
export PINECONE_ENVIRONMENT="your-environment"
export PINECONE_INDEX="your-index-name"
```

**Option B: Configuration File**

```bash
mkdir -p ~/.config/tldr
cp config.toml.example ~/.config/tldr/config.toml
# Edit ~/.config/tldr/config.toml with your API keys
```

### 5. **Start Using TLDR**

```bash
# Index a directory
tldr index /path/to/your/project

# Search for content
tldr search "authentication function"

# Ask questions
tldr ask "How does the API work?"

# Interactive mode
tldr interactive
```

## 🎯 Usage Examples

### **Indexing Content**

```bash
# Index all text files in a directory
tldr index /path/to/docs

# Index specific file types
tldr index /path/to/code --patterns "*.py,*.js,*.md"

# Index with custom patterns
tldr index /path/to/project --patterns "*.rs,*.toml,*.md"
```

### **Semantic Search**

```bash
# Find authentication-related code
tldr search "user login authentication"

# Search with custom limits
tldr search "database connection" --limit 10 --threshold 0.5

# Find specific functionality
tldr search "error handling middleware"
```

### **RAG Questions**

```bash
# Ask about specific features
tldr ask "How does the caching system work?"

# Get implementation details
tldr ask "What are the main API endpoints?"

# Understand architecture
tldr ask "How is the data structured?"
```

## 🏗️ Architecture

### **Components**

- **OpenAI Embeddings** - Generates 1536-dimensional semantic vectors
- **Pinecone Vector Database** - Stores and searches vectors at scale
- **SQLite Local Storage** - Stores document metadata and chunk text
- **Smart Chunking** - Unicode-aware text processing with overlap
- **CLI Interface** - Interactive, user-friendly command-line experience

### **Data Flow**

1. **Indexing**: File → Chunking → OpenAI Embeddings → Pinecone Storage
2. **Search**: Query → OpenAI Embedding → Pinecone Similarity Search → Results
3. **RAG**: Question → Search → Context Retrieval → Answer Generation

### **Vector Storage**

- **Embedding Model**: OpenAI text-embedding-ada-002
- **Vector Dimensions**: 1536
- **Similarity Metric**: Cosine similarity
- **Storage**: Pinecone managed vector database

## ⚙️ Configuration

### **Environment Variables**

```bash
OPENAI_API_KEY=sk-...
PINECONE_API_KEY=your-pinecone-key
PINECONE_ENVIRONMENT=us-west1-gcp
PINECONE_INDEX=your-index
```

### **Configuration File**

```toml
[openai]
api_key = "your-openai-api-key"

[pinecone]
api_key = "your-pinecone-api-key"
environment = "us-west1-gcp"
index_name = "your-index"

[database]
path = "tldr.db"
```

## 🔧 Development

### **Building**

```bash
cargo build --release
```

### **Testing**

```bash
cargo test
```

### **Running**

```bash
cargo run -- index /path/to/test
cargo run -- search "test query"
```

## 📊 Performance

- **Indexing Speed**: ~100-500 files/minute (depending on file sizes)
- **Search Speed**: <100ms for most queries
- **Memory Usage**: Optimized with file size and chunk limits
- **Scalability**: Pinecone handles millions of vectors

## 🎨 Interactive Mode

The interactive mode provides a guided experience:

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║  ████████╗██╗     ██████╗ ██████╗                          ║
║  ╚══██╔══╝██║     ██╔══██╗██╔══██╗                         ║
║     ██║   ██║     ██║  ██║██║  ██║                         ║
║     ██║   ██║     ██║  ██║██║  ██║                         ║
║     ██║   ███████╗██████╔╝██████╔╝                         ║
║     ╚═╝   ╚══════╝╚═════╝ ╚═════╝                          ║
║                                                              ║
║  Too Long; Didn't Read - Semantic Search Made Simple        ║
║  Blazing-fast search through any directory                   ║
╚══════════════════════════════════════════════════════════════╝

📊 Current Status:
   📄 Documents: 0
   📝 Chunks: 0
   💾 Database: 0.00 MB

🚀 Actions:
   1. 📁 Index Directory     - Add files to search
   2. 🔍 Search Content      - Find relevant content
   3. ❓ Ask Questions       - Get AI-powered answers
   4. 📊 View Statistics     - See database info
   5. 🧹 Clear Database      - Remove all data
   6. ⚙️  Settings           - Configure TLDR
   7. ❌ Exit                - Close TLDR

💡 Tip: Type 'q', 'quit', or 'exit' to leave
```

## 🚨 Limitations

- **File Size**: Maximum 5MB per file
- **Chunk Count**: Maximum 100 chunks per file
- **Memory**: Estimated 10MB limit per file
- **API Costs**: OpenAI and Pinecone usage costs apply
- **Rate Limits**: Respect OpenAI and Pinecone rate limits

## 🔮 Future Enhancements

- **Local Embeddings** - ONNX models for offline use
- **Advanced RAG** - GPT integration for better answers
- **Vector Visualization** - 2D/3D projections of embeddings
- **Batch Processing** - Parallel indexing for large datasets
- **Web Interface** - Optional web UI for visualization

## 📝 License

MIT License - see LICENSE file for details.

## 🤝 Contributing

Contributions welcome! Please read CONTRIBUTING.md for guidelines.

---

**TLDR** - Because life's too short to read everything! 🚀
