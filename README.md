# Drag-n-Vector

A drag-and-drop vector search application with real-time 2D visualization using UMAP projections.

## Features

- **Drag & Drop Pipeline**: Visual pipeline with File → Chunk → Embed → Index blocks
- **File Upload**: Upload .txt files for processing
- **Vector Search**: Semantic search with cosine similarity
- **2D Visualization**: UMAP projection of embeddings with hover preview
- **Real-time Updates**: Automatic refresh of visualizations after uploads
- **Interactive Results**: Click search results to highlight in 2D map

## Architecture

- **Backend**: FastAPI with PostgreSQL + pgvector
- **Frontend**: Next.js 14 with React Flow and Plotly
- **Embeddings**: sentence-transformers (all-MiniLM-L6-v2)
- **Visualization**: UMAP for 2D projection
- **Database**: PostgreSQL with pgvector extension

## Quick Start

### 1. Start the Database

```bash
docker compose up -d
```

### 2. Setup Backend

```bash
cd backend
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
uvicorn app:app --reload --port 8000
```

### 3. Setup Frontend

```bash
cd frontend
npm install  # or pnpm install
npm run dev  # http://localhost:3000
```

## Usage

1. **Upload Document**: Click "Run Pipeline" or use the upload section to add a .txt file
2. **Search**: Enter queries in the search panel to find similar chunks
3. **Visualize**: See 2D projection of all embeddings with hover details
4. **Interact**: Click search results to highlight them in the 2D map

## API Endpoints

- `POST /collections/create` - Create a new collection
- `POST /ingest/file` - Upload and process a text file
- `POST /upsert` - Add chunks with embeddings
- `POST /query` - Search for similar chunks
- `GET /projection/{collection_id}` - Get 2D projection data

## Database Schema

- `collections` - Document collections
- `documents` - Source documents
- `chunks` - Text chunks
- `embeddings` - Vector embeddings (384d)

## Development

### Backend Structure

```
backend/
├── app.py              # FastAPI routes
├── db.py               # Database connection
├── models.sql          # Schema definition
├── util_embeddings.py  # Sentence transformers
├── util_chunk.py       # Text chunking
├── util_projection.py  # UMAP projection
└── requirements.txt    # Dependencies
```

### Frontend Structure

```
frontend/
├── src/
│   ├── app/
│   │   ├── page.tsx    # Main page
│   │   ├── layout.tsx  # Root layout
│   │   └── api/
│   │       └── config.ts # API configuration
│   └── components/
│       ├── Canvas.tsx      # Pipeline visualization
│       ├── Upload.tsx      # File upload
│       ├── QueryPanel.tsx  # Search interface
│       └── Projection.tsx  # 2D visualization
└── package.json
```

## Testing

### Quick API Test

```bash
# Create collection
curl -X POST "http://localhost:8000/collections/create" \
  -F "name=Test"

# Add sample chunks
curl -X POST "http://localhost:8000/upsert" \
  -H "Content-Type: application/json" \
  -d '{
    "collection_id": 1,
    "chunks": [
      {"text": "Machine learning is a subset of artificial intelligence."},
      {"text": "Deep learning uses neural networks with multiple layers."},
      {"text": "Natural language processing helps computers understand text."}
    ]
  }'

# Search
curl -X POST "http://localhost:8000/query" \
  -H "Content-Type: application/json" \
  -d '{
    "collection_id": 1,
    "query": "artificial intelligence",
    "top_k": 3
  }'
```

## Performance Notes

- Embeddings are normalized for better cosine similarity
- UMAP runs on-demand for collections
- IVFFlat index used for vector similarity search
- For large datasets, consider capping UMAP to 10k points

## Troubleshooting

1. **Database Connection**: Ensure PostgreSQL is running on port 5432
2. **Model Download**: First run may download sentence-transformers model (~90MB)
3. **CORS Issues**: Backend configured for localhost:3000
4. **Memory**: UMAP can be memory-intensive for large datasets

## License

MIT License
