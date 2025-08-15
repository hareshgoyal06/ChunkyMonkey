import umap
import numpy as np

def project_umap(vectors: list[list[float]]) -> list[list[float]]:
    """
    Project high-dimensional vectors to 2D using UMAP
    
    Args:
        vectors: List of embedding vectors (each vector is a list of floats)
        
    Returns:
        List of 2D coordinates [[x, y], ...]
    """
    if not vectors:
        return []
    
    # Convert to numpy array
    vectors_array = np.array(vectors)
    
    # Create UMAP reducer with fixed parameters
    reducer = umap.UMAP(
        n_neighbors=15,
        min_dist=0.1,
        random_state=42,
        n_components=2
    )
    
    # Fit and transform
    embedding_2d = reducer.fit_transform(vectors_array)
    
    # Convert back to list of lists
    return embedding_2d.tolist() 