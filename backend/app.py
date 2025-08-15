from fastapi import FastAPI, UploadFile, File, Form, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import json
import io

from db import get_conn, init_db
from util_embeddings import embed_texts
from util_chunk import simple_chunk
from util_projection import project_umap, project_umap_3d

app = FastAPI(title="Vector Database Demo API")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize database on startup
@app.on_event("startup")
async def startup_event():
    init_db()

# Pydantic models
class UpsertRequest(BaseModel):
    collection_id: int
    chunks: List[Dict[str, Any]]

class QueryRequest(BaseModel):
    collection_id: int
    query: str
    top_k: int = 5

class QueryResponse(BaseModel):
    chunk_id: int
    text: str
    metadata: Optional[Dict[str, Any]]
    score: float

class RAGRequest(BaseModel):
    collection_id: int
    query: str
    top_k: int = 5
    include_context: bool = True

class RAGResponse(BaseModel):
    query: str
    context: List[str]
    answer: str
    sources: List[Dict[str, Any]]

# Debug endpoint to check database contents
@app.get("/debug/{collection_id}")
async def debug_collection(collection_id: int):
    """Debug endpoint to check what's in the database"""
    try:
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                # Check collections
                cur.execute("SELECT id, name FROM collections WHERE id = %s", (collection_id,))
                collections = cur.fetchall()
                
                # Check documents
                cur.execute("SELECT id, source_uri FROM documents WHERE collection_id = %s", (collection_id,))
                documents = cur.fetchall()
                
                # Check chunks
                cur.execute("SELECT id, text FROM chunks WHERE collection_id = %s LIMIT 5", (collection_id,))
                chunks = cur.fetchall()
                
                # Check embeddings
                cur.execute("""
                    SELECT e.id, e.chunk_id 
                    FROM embeddings e 
                    JOIN chunks c ON c.id = e.chunk_id 
                    WHERE c.collection_id = %s 
                    LIMIT 5
                """, (collection_id,))
                embeddings = cur.fetchall()
                
                return {
                    "collections": collections,
                    "documents": documents,
                    "chunks_count": len(chunks),
                    "sample_chunks": chunks,
                    "embeddings_count": len(embeddings),
                    "sample_embeddings": embeddings
                }
        finally:
            conn.close()
    except Exception as e:
        return {"error": str(e)}

# Routes
@app.post("/collections/create")
async def create_collection(name: str = Form(...)):
    """Create a new collection"""
    conn = get_conn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "INSERT INTO collections(name) VALUES (%s) RETURNING id, name",
                (name,)
            )
            result = cur.fetchone()
            return {"id": result[0], "name": result[1]}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
    finally:
        conn.close()

@app.post("/ingest/file")
async def ingest_file(
    collection_id: int = Form(...),
    file: UploadFile = File(...),
    chunk_size: int = Form(800),
    overlap: int = Form(150)
):
    """Ingest a text file and create embeddings"""
    if not file.filename.endswith('.txt'):
        raise HTTPException(status_code=400, detail="Only .txt files are supported")
    
    try:
        # Read file content
        content = await file.read()
        text = content.decode('utf-8')
        
        # Chunk the text
        chunks = simple_chunk(text, chunk_size, overlap)
        if not chunks:
            raise HTTPException(status_code=400, detail="No valid chunks created from file")
        
        # Create embeddings
        vectors = embed_texts(chunks)
        
        # Store in database
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                # Insert document
                cur.execute(
                    "INSERT INTO documents(collection_id, source_uri, metadata) VALUES (%s, %s, %s) RETURNING id",
                    (collection_id, file.filename, json.dumps({}))
                )
                document_id = cur.fetchone()[0]
                
                # Insert chunks and embeddings
                for i, (chunk, vector) in enumerate(zip(chunks, vectors)):
                    # Insert chunk
                    cur.execute(
                        "INSERT INTO chunks(collection_id, document_id, text, metadata) VALUES (%s, %s, %s, %s) RETURNING id",
                        (collection_id, document_id, chunk, json.dumps({}))
                    )
                    chunk_id = cur.fetchone()[0]
                    
                    # Insert embedding
                    cur.execute(
                        "INSERT INTO embeddings(chunk_id, vector) VALUES (%s, %s)",
                        (chunk_id, vector)
                    )
                
                return {"ok": True, "chunks": len(chunks)}
        finally:
            conn.close()
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/upsert")
async def upsert_chunks(request: UpsertRequest):
    """Upsert chunks with embeddings"""
    if not request.chunks:
        raise HTTPException(status_code=400, detail="No chunks provided")
    
    try:
        # Extract texts and create embeddings
        texts = [chunk["text"] for chunk in request.chunks]
        vectors = embed_texts(texts)
        
        # Store in database
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                # Insert document (placeholder)
                cur.execute(
                    "INSERT INTO documents(collection_id, source_uri, metadata) VALUES (%s, %s, %s) RETURNING id",
                    (request.collection_id, "upsert", json.dumps({}))
                )
                document_id = cur.fetchone()[0]
                
                # Insert chunks and embeddings
                for i, (chunk_data, vector) in enumerate(zip(request.chunks, vectors)):
                    metadata = chunk_data.get("metadata", {})
                    
                    # Insert chunk
                    cur.execute(
                        "INSERT INTO chunks(collection_id, document_id, text, metadata) VALUES (%s, %s, %s, %s) RETURNING id",
                        (request.collection_id, document_id, chunk_data["text"], json.dumps(metadata))
                    )
                    chunk_id = cur.fetchone()[0]
                    
                    # Insert embedding
                    cur.execute(
                        "INSERT INTO embeddings(chunk_id, vector) VALUES (%s, %s)",
                        (chunk_id, vector)
                    )
                
                return {"ok": True, "count": len(request.chunks)}
        finally:
            conn.close()
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/query")
async def query_chunks(request: QueryRequest) -> List[QueryResponse]:
    """Query chunks using vector similarity"""
    try:
        # Embed the query
        query_vectors = embed_texts([request.query])
        query_vector = query_vectors[0]
        
        # Search in database
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                cur.execute("""
                    SELECT c.id, c.text, c.metadata, 1 - (e.vector <=> %s::vector) as score
                    FROM embeddings e
                    JOIN chunks c ON c.id = e.chunk_id
                    WHERE c.collection_id = %s
                    ORDER BY e.vector <=> %s::vector
                    LIMIT %s
                """, (query_vector, request.collection_id, query_vector, request.top_k))
                
                results = []
                for row in cur.fetchall():
                    chunk_id, text, metadata, score = row
                    results.append(QueryResponse(
                        chunk_id=chunk_id,
                        text=text,
                        metadata=metadata if metadata else None,
                        score=float(score)
                    ))
                
                return results
        finally:
            conn.close()
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/rag")
async def rag_query(request: RAGRequest) -> RAGResponse:
    """RAG (Retrieval-Augmented Generation) query"""
    try:
        # First, retrieve relevant chunks
        query_vectors = embed_texts([request.query])
        query_vector = query_vectors[0]
        
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                cur.execute("""
                    SELECT c.id, c.text, c.metadata, 1 - (e.vector <=> %s::vector) as score
                    FROM embeddings e
                    JOIN chunks c ON c.id = e.chunk_id
                    WHERE c.collection_id = %s
                    ORDER BY e.vector <=> %s::vector
                    LIMIT %s
                """, (query_vector, request.collection_id, query_vector, request.top_k))
                
                results = cur.fetchall()
                
                if not results:
                    return RAGResponse(
                        query=request.query,
                        context=[],
                        answer="No relevant information found in the knowledge base.",
                        sources=[]
                    )
                
                # Extract context and sources
                context_chunks = [row[1] for row in results]
                sources = [
                    {
                        "chunk_id": row[0],
                        "text": row[1][:200] + "..." if len(row[1]) > 200 else row[1],
                        "score": float(row[3])
                    }
                    for row in results
                ]
                
                # Simple answer generation (in a real RAG system, you'd use an LLM here)
                if request.include_context:
                    context_text = "\n\n".join(context_chunks)
                    answer = f"Based on the retrieved information:\n\n{context_text}"
                else:
                    answer = f"Found {len(results)} relevant chunks with scores ranging from {min([r[3] for r in results]):.2f} to {max([r[3] for r in results]):.2f}"
                
                return RAGResponse(
                    query=request.query,
                    context=context_chunks,
                    answer=answer,
                    sources=sources
                )
        finally:
            conn.close()
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/projection/{collection_id}")
async def get_projection(collection_id: int):
    """Get 3D projection of all vectors in a collection"""
    try:
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                # Get all vectors and texts for the collection
                cur.execute("""
                    SELECT c.id, c.text, e.vector
                    FROM embeddings e
                    JOIN chunks c ON c.id = e.chunk_id
                    WHERE c.collection_id = %s
                """, (collection_id,))
                
                rows = cur.fetchall()
                print(f"DEBUG: Found {len(rows)} rows for collection {collection_id}")
                
                if not rows:
                    print("DEBUG: No rows found, returning empty points")
                    return {"points": []}
                
                # Extract data
                chunk_ids = [row[0] for row in rows]
                texts = [row[1] for row in rows]
                vectors = [row[2] for row in rows]
                
                print(f"DEBUG: Processing {len(vectors)} vectors for 3D UMAP")
                
                # Project to 3D
                try:
                    points_3d = project_umap_3d(vectors)
                    print(f"DEBUG: 3D UMAP returned {len(points_3d)} points")
                except Exception as e:
                    print(f"DEBUG: 3D UMAP error: {e}")
                    raise e
                
                # Format response
                points = []
                for i, (chunk_id, text, coords) in enumerate(zip(chunk_ids, texts, points_3d)):
                    points.append({
                        "id": chunk_id,
                        "x": coords[0],
                        "y": coords[1],
                        "z": coords[2],
                        "text": text[:100] + "..." if len(text) > 100 else text
                    })
                
                print(f"DEBUG: Returning {len(points)} 3D points")
                return {"points": points}
        finally:
            conn.close()
            
    except Exception as e:
        print(f"DEBUG: Projection error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/projection-2d/{collection_id}")
async def get_projection_2d(collection_id: int):
    """Get 2D projection of all vectors in a collection (legacy)"""
    try:
        conn = get_conn()
        try:
            with conn.cursor() as cur:
                # Get all vectors and texts for the collection
                cur.execute("""
                    SELECT c.id, c.text, e.vector
                    FROM embeddings e
                    JOIN chunks c ON c.id = e.chunk_id
                    WHERE c.collection_id = %s
                """, (collection_id,))
                
                rows = cur.fetchall()
                if not rows:
                    return {"points": []}
                
                # Extract data
                chunk_ids = [row[0] for row in rows]
                texts = [row[1] for row in rows]
                vectors = [row[2] for row in rows]
                
                # Project to 2D
                points_2d = project_umap(vectors)
                
                # Format response
                points = []
                for i, (chunk_id, text, coords) in enumerate(zip(chunk_ids, texts, points_2d)):
                    points.append({
                        "id": chunk_id,
                        "x": coords[0],
                        "y": coords[1],
                        "text": text[:100] + "..." if len(text) > 100 else text
                    })
                
                return {"points": points}
        finally:
            conn.close()
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000) 