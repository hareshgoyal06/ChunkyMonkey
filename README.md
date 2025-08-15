# 🔍 TLDR - Too Long; Didn't Read

**Blazing-fast semantic search through any directory using vector embeddings.**

TLDR is a powerful command-line tool that helps you find and understand code, documentation, and text files instantly using AI-powered semantic search.

## 🚀 Features

- **⚡ Blazing Fast**: Built in Rust for maximum performance
- **🧠 Semantic Search**: Find content by meaning, not just keywords
- **❓ RAG Pipeline**: Ask questions and get AI-generated answers
- **🎯 Interactive Mode**: Beautiful CLI interface for exploration
- **📁 Multi-format Support**: Rust, Python, JavaScript, Markdown, JSON, YAML, and more
- **🔄 Smart Indexing**: Only re-indexes changed files
- **💾 SQLite Storage**: Lightweight, portable database

## 🎯 Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/tldr.git
cd tldr

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Basic Usage

```bash
# Index a directory
tldr index /path/to/your/project

# Search for content
tldr search "authentication function"

# Ask questions about your codebase
tldr ask "How does the API work?"

# Start interactive mode
tldr interactive

# View statistics
tldr stats
```

## 🎮 Interactive Mode

TLDR's interactive mode provides a beautiful, menu-driven interface:

```
🔍 TLDR - Too Long; Didn't Read
Blazing-fast semantic search through any directory
Powered by vector embeddings and AI

📋 Main Menu
========================================
1. 📁 Index Directory
2. 🔍 Search Content
3. ❓ Ask Questions (RAG)
4. 📊 View Statistics
5. 🧹 Clear Database
6. ❌ Exit
========================================
Enter your choice (1-6):
```

## 📁 Supported File Types

- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.jsx`, `.tsx`, `.java`, `.cpp`, `.c`
- **Markup**: `.md`, `.txt`, `.rst`
- **Config**: `.json`, `.yaml`, `.yml`, `.toml`, `.ini`
- **Documentation**: `.md`, `.txt`, `.rst`

## 🛠️ Technical Details

### Architecture

- **Language**: Rust (for performance and safety)
- **Database**: SQLite with JSON vector storage
- **Embeddings**: 384-dimensional vectors (configurable)
- **Similarity**: Cosine similarity for ranking
- **Chunking**: Configurable chunk size with overlap

### Performance

- **Indexing**: ~1000 files/second (depending on content)
- **Search**: Sub-second response times
- **Memory**: Minimal memory footprint
- **Storage**: Efficient SQLite storage with indexing

## 🎯 Use Cases

### Codebases
```bash
# Index your entire codebase
tldr index /path/to/your/project

# Find authentication code
tldr search "user login authentication"

# Understand API structure
tldr ask "How is the API structured?"
```

### Documentation
```bash
# Index documentation
tldr index /path/to/docs -p "*.md,*.txt"

# Find relevant docs
tldr search "deployment configuration"

# Get quick answers
tldr ask "How do I deploy the application?"
```

### Research
```bash
# Index research papers
tldr index /path/to/papers -p "*.pdf,*.txt"

# Find related concepts
tldr search "machine learning algorithms"

# Summarize findings
tldr ask "What are the main findings about neural networks?"
```

## 🔧 Advanced Usage

### Custom File Patterns

```bash
# Index specific file types
tldr index /path/to/project -p "*.rs,*.md,*.toml"

# Exclude certain patterns
tldr index /path/to/project -p "*.rs" --exclude "target/,*.test.rs"
```

### Search Options

```bash
# Get more results
tldr search "function definition" -l 10

# Set similarity threshold
tldr search "database connection" -t 0.5

# Combine options
tldr search "API endpoint" -l 15 -t 0.4
```

### RAG Configuration

```bash
# Use more context for better answers
tldr ask "Explain the entire authentication flow" -c 5

# Get detailed answers
tldr ask "How does error handling work?" -c 10
```

## 🚀 Performance Tips

1. **Index during off-hours** for large codebases
2. **Use specific file patterns** to reduce indexing time
3. **Exclude build artifacts** and temporary files
4. **Regular re-indexing** keeps search results fresh

## 🔧 Configuration

TLDR uses sensible defaults but can be customized:

### Environment Variables

```bash
# Custom database path
export TLDR_DB_PATH="/path/to/custom/tldr.db"

# Custom embedding model
export TLDR_MODEL="all-MiniLM-L6-v2"

# Debug mode
export TLDR_DEBUG=1
```

### Configuration File

Create `~/.config/tldr/config.toml`:

```toml
[database]
path = "/path/to/custom/tldr.db"

[embedding]
model = "all-MiniLM-L6-v2"
dimensions = 384

[indexing]
chunk_size = 800
overlap = 150
default_patterns = ["*.rs", "*.md", "*.txt"]

[search]
default_limit = 5
default_threshold = 0.3
```

## 🎯 Perfect For

- **Developers**: Understand large codebases quickly
- **Researchers**: Search through papers and documentation
- **Technical Writers**: Find relevant content instantly
- **Students**: Explore code examples and documentation
- **DevOps**: Search through configuration files

## 🔍 How It Works

1. **Indexing**: Files are chunked and converted to vector embeddings
2. **Storage**: Embeddings stored in SQLite with efficient indexing
3. **Search**: Queries are embedded and matched using cosine similarity
4. **RAG**: Retrieved chunks are used to generate contextual answers

## 🛠️ Development

### Building from Source

```bash
# Clone and build
git clone https://github.com/yourusername/tldr.git
cd tldr
cargo build

# Run tests
cargo test

# Run with debug info
cargo run -- interactive
```

### Project Structure

```
src/
├── main.rs              # CLI entry point
├── core/                # Core application logic
│   ├── app.rs          # Main application struct
│   ├── types.rs        # Data structures
│   └── config.rs       # Configuration
├── db/                  # Database operations
│   └── mod.rs          # SQLite interface
├── embeddings/          # Vector embeddings
│   └── mod.rs          # Embedding model
├── search/              # Search functionality
│   ├── mod.rs          # Search logic
│   └── indexer.rs      # File indexing
├── cli/                 # CLI interface
│   └── interactive.rs  # Interactive mode
└── ui/                  # UI components
    └── mod.rs          # TUI components
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🎉 Get Started

```bash
# Install TLDR
cargo install --git https://github.com/yourusername/tldr.git

# Index your first project
tldr index /path/to/your/project

# Start exploring
tldr interactive
```

**Experience the power of semantic search with TLDR! 🚀**
