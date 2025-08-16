# ChunkyMonkey 🐒

A powerful, intelligent RAG (Retrieval-Augmented Generation) system built in Rust with advanced ML-based retrieval and semantic understanding.

## 🚀 Key Features

### Advanced RAG Pipeline

- **Semantic Search**: Uses advanced embedding models for intelligent content retrieval
- **Multi-Factor Reranking**: Combines similarity scores with content quality metrics
- **Query Expansion**: Automatically expands queries with related concepts and synonyms
- **Content Filtering**: Filters out low-quality, irrelevant, or technical gibberish
- **Semantic Chunking**: Creates meaningful chunks that respect logical boundaries

### Intelligent Content Understanding

- **Concept Extraction**: Automatically identifies key technical concepts from queries
- **Pattern Recognition**: Recognizes question-answer patterns (how-to, definitions, etc.)
- **Quality Scoring**: Multi-dimensional content quality assessment
- **Relevance Analysis**: Detailed analysis of why search results are relevant

### Performance & Reliability

- **Higher Similarity Thresholds**: Default 0.5 threshold for better quality results
- **Duplicate Detection**: Eliminates redundant search results
- **Fallback Strategies**: Graceful degradation when high-quality results aren't available
- **Result Analysis**: Built-in tools to understand and debug search quality

## 🔧 Configuration

The system is highly configurable through `config.toml`:

```toml
[search]
# Higher similarity threshold for better quality results
base_similarity_threshold = 0.5
fallback_threshold = 0.4
max_results_per_query = 10

# Enable advanced ML-based features
enable_semantic_search = true
enable_query_expansion = true
enable_content_filtering = true
enable_reranking = true

[chunking]
# Semantic chunking for better content understanding
max_chunk_size = 1500
min_chunk_size = 200
overlap_size = 200
use_semantic_chunking = true
respect_section_boundaries = true
```

## 🎯 How It Solves RAG Problems

### Problem 1: Irrelevant Results

**Before**: Low similarity thresholds (0.3) allowed irrelevant content like protobuf version info
**Solution**: Higher thresholds (0.5) + intelligent content filtering

### Problem 2: Poor Content Quality

**Before**: Fixed-size chunks often broke logical content
**Solution**: Semantic chunking that respects section boundaries

### Problem 3: Basic Similarity Scoring

**Before**: Only cosine similarity between vectors
**Solution**: Multi-factor scoring including content quality, keyword matches, and semantic relevance

### Problem 4: No Query Understanding

**Before**: Literal query matching only
**Solution**: Query expansion with technical concepts and synonyms

### Problem 5: Inconsistent Results

**Before**: Results varied widely in quality and relevance
**Solution**: Advanced reranking and content quality filtering

## 🚀 Usage

### Interactive Mode

```bash
cargo run -- interactive
```

### Ask Questions

```
🤖 Ask me anything about your indexed content!
   Examples: 'How does authentication work?', 'What are the main features?'
   Type 'back' to return to main menu
   Type 'analyze' to analyze search results

Your question: how does auth work
```

### Analyze Results

Type `analyze` to get detailed insights about search result quality:

- Overall quality metrics
- Individual result analysis
- Content quality assessment
- Relevance indicators
- Improvement recommendations

## 🏗️ Architecture

### Core Components

1. **Semantic Search Engine**: Advanced retrieval with concept extraction
2. **Multi-Factor Reranker**: Combines similarity, quality, and relevance scores
3. **Content Quality Filter**: Filters out low-quality or irrelevant chunks
4. **Semantic Chunker**: Creates meaningful content chunks
5. **Query Expander**: Expands queries with related concepts

### Search Pipeline

1. **Query Processing**: Extract key concepts and expand queries
2. **Multi-Query Retrieval**: Search with original and expanded queries
3. **Semantic Reranking**: Apply advanced scoring algorithms
4. **Quality Filtering**: Remove low-quality results
5. **Result Analysis**: Provide insights and recommendations

## 🔍 Search Quality Metrics

The system tracks multiple quality indicators:

- **Similarity Score**: Vector similarity (0.0 - 1.0)
- **Content Quality**: Meaningful content ratio, structure, completeness
- **Relevance**: Keyword matches, concept matches, pattern recognition
- **Confidence**: Overall result quality assessment

## 🛠️ Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Configuration

Copy `config.toml.example` to `config.toml` and customize settings.

## 📊 Performance

- **Search Speed**: Sub-second response times for most queries
- **Accuracy**: Significantly improved relevance through multi-factor scoring
- **Scalability**: Efficient SQLite backend with optional Pinecone integration
- **Memory**: Optimized chunking and embedding storage

## 🤝 Contributing

Contributions are welcome! Areas for improvement:

- Additional embedding models
- More sophisticated reranking algorithms
- Enhanced content quality metrics
- Performance optimizations

## 📄 License

MIT License - see LICENSE file for details.
