from sentence_transformers import SentenceTransformer
import numpy as np

# Load the model once at module level
_model = None

def get_model():
    """Get or load the sentence transformer model"""
    global _model
    if _model is None:
        _model = SentenceTransformer('all-MiniLM-L6-v2')
    return _model

def embed_texts(texts: list[str]) -> list[list[float]]:
    """
    Embed a list of texts using sentence-transformers
    
    Args:
        texts: List of text strings to embed
        
    Returns:
        List of normalized embedding vectors (384 dimensions)
    """
    if not texts:
        return []
    
    model = get_model()
    embeddings = model.encode(texts, normalize_embeddings=True)
    
    # Convert to list of lists for JSON serialization
    return embeddings.tolist() 