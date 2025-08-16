# 🐒 ChunkyMonkey - Going Bananas for Chunks! 🍌

ChunkyMonkey is a fun and powerful semantic search and RAG (Retrieval-Augmented Generation) system that helps you organize, search, and understand your documents using AI-powered chunking and vector embeddings.

## 🌟 Features

- **🐒 Project Management**: Organize your documents into projects for better organization
- **📁 Smart Indexing**: Automatically chunk and index documents with configurable patterns
- **🔍 Semantic Search**: Find relevant content using vector similarity search
- **❓ AI Q&A**: Ask questions and get intelligent answers based on your documents
- **🍌 Pinecone Integration**: Optional cloud vector database for scalable search
- **🚀 Local & Cloud**: Works with local Ollama models or cloud APIs
- **🎨 Beautiful UI**: Fun monkey-themed interface that makes document management enjoyable

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Optional: Ollama for local AI models
- Optional: Pinecone account for cloud vector storage

### Quick Commands

After installation, you can use the short `cm` alias for faster access:

```bash
cm --help          # Show help
cm start           # Start interactive mode (recommended)
cm interactive     # Start interactive mode (alternative)
cm index /path     # Index a directory
cm search "query"  # Search for content
cm ask "question"  # Ask a question
cm stats           # Show statistics
```

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/chunkymonkey.git
cd chunkymonkey

# Build the project
cargo build --release

# Run the interactive mode
./target/release/chunkymonkey start
# or
./target/release/chunkymonkey interactive

# Or use the short 'cm' alias (after installation)
cm start
# or
cm interactive
```

### First Time Setup

1. **Create Your First Project**: ChunkyMonkey will guide you through creating your first project
2. **Index Documents**: Add files and directories to your project
3. **Start Searching**: Use semantic search to find relevant content
4. **Ask Questions**: Get AI-powered answers about your documents

## 📖 Usage

### Interactive Mode (Recommended)

```bash
chunkymonkey interactive
```

The interactive mode provides a user-friendly menu system:

- 🗂️ Manage Projects
- 📁 Index Directory
- 🔍 Search Content
- ❓ Ask Questions
- 📊 View Statistics
- ⚙️ Settings

### Command Line Mode

```bash
# Index a directory
chunkymonkey index /path/to/documents --patterns "*.txt,*.md,*.py"
# or use the short alias:
cm index /path/to/documents --patterns "*.txt,*.md,*.py"

# Search for content
chunkymonkey search "your search query" --limit 10 --threshold 0.7
# or use the short alias:
cm search "your search query" --limit 10 --threshold 0.7

# Ask a question
chunkymonkey ask "What is this document about?" --context 5
# or use the short alias:
cm ask "What is this document about?" --context 5

# View statistics
chunkymonkey stats
# or use the short alias:
cm stats
```

## 🗂️ Project Management

ChunkyMonkey organizes your documents into projects:

- **Create Projects**: Give your document collections meaningful names and descriptions
- **Organize Documents**: Add files to specific projects for better organization
- **Track Progress**: Monitor document and chunk counts per project
- **Flexible Structure**: Create as many projects as you need

## 🔧 Configuration

### Shell Setup

For the easiest access to the `cm` command, you can add it to your shell profile:

```bash
# Option 1: Source the setup script
source /path/to/chunkymonkey/shell_setup.sh

# Option 2: Add to your ~/.bashrc or ~/.zshrc
echo 'export PATH="/path/to/chunkymonkey:$PATH"' >> ~/.bashrc
echo 'alias cm="/path/to/chunkymonkey/target/release/chunkymonkey"' >> ~/.bashrc

# Option 3: Use the system-wide installation (recommended)
sudo ./install_cm.sh
```

### Environment Variables

```bash
# Pinecone Configuration (Optional)
PINECONE_API_KEY=your_api_key
PINECONE_ENVIRONMENT=your_environment
PINECONE_INDEX_NAME=your_index_name

# OpenAI Configuration (Optional)
OPENAI_API_KEY=your_api_key
```

### Configuration File

Create a `config.toml` file in your project directory:

```toml
[pinecone]
api_key = "your_api_key"
environment = "your_environment"
index_name = "your_index_name"

[openai]
api_key = "your_api_key"
```

## 🏗️ Architecture

- **Core**: Rust-based application with async/await support
- **Database**: SQLite for metadata storage
- **Vector Search**: Local HNSW index + optional Pinecone integration
- **Embeddings**: Support for OpenAI and Ollama models
- **Chunking**: Intelligent text chunking with overlap
- **UI**: Terminal-based interactive interface

## 🐒 Why ChunkyMonkey?

- **Fun & Engaging**: Monkey-themed interface makes document management enjoyable
- **Powerful**: Advanced semantic search and RAG capabilities
- **Flexible**: Works locally or in the cloud
- **Fast**: Optimized Rust implementation for performance
- **Simple**: Easy-to-use interface for complex operations

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with Rust and amazing open-source libraries
- Inspired by the need for better document organization
- Named with love for our primate friends 🐒🍌

---

**🐒 Ready to go bananas for chunks? Start using ChunkyMonkey today! 🍌**
