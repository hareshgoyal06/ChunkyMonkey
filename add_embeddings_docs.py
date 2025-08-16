#!/usr/bin/env python3
"""
Script to add educational content about vector embeddings to Pinecone
This will provide richer context for RAG responses
"""

import requests
import json
import time

# Pinecone configuration
PINECONE_API_KEY = "pcsk_4bSFWF_KpC7MSD3uKP9p13zGgd5HVgGgAB7ioqaCaeP3f4uT49CYNFcDRHW72E"
PINECONE_HOST = "https://chunky-monkey-test-a2r60ad.svc.aped-4627-b74a.pinecone.io"

# Educational content about vector embeddings
EDUCATIONAL_CONTENT = [
    {
        "id": "embeddings-intro-1",
        "text": "Vector embeddings are numerical representations of text, images, or other data that capture semantic meaning in a high-dimensional space. They convert complex information into arrays of numbers where similar meanings are positioned close together.",
        "source": "vector-embeddings-guide",
        "category": "introduction"
    },
    {
        "id": "embeddings-how-it-works-1", 
        "text": "The process of creating vector embeddings involves training neural networks on large amounts of text data. These networks learn to map words, phrases, and documents into a vector space where semantic relationships are preserved through geometric distances.",
        "source": "vector-embeddings-guide",
        "category": "technical"
    },
    {
        "id": "embeddings-applications-1",
        "text": "Vector embeddings enable powerful applications like semantic search, recommendation systems, question answering, and content clustering. They allow computers to understand meaning rather than just matching keywords.",
        "source": "vector-embeddings-guide", 
        "category": "applications"
    },
    {
        "id": "embeddings-similarity-1",
        "text": "Similarity between vectors is typically measured using cosine similarity or Euclidean distance. Cosine similarity measures the angle between vectors, making it ideal for comparing document embeddings regardless of their magnitude.",
        "source": "vector-embeddings-guide",
        "category": "similarity"
    },
    {
        "id": "embeddings-rag-1",
        "text": "In Retrieval-Augmented Generation (RAG), vector embeddings are used to find the most relevant documents or text chunks for a given question. The retrieved context is then used to generate accurate and informative answers.",
        "source": "vector-embeddings-guide",
        "category": "rag"
    },
    {
        "id": "embeddings-dimensions-1",
        "text": "The dimensionality of embeddings (like 768 in your system) determines how much semantic information can be captured. Higher dimensions allow for more nuanced representations but require more computational resources.",
        "source": "vector-embeddings-guide",
        "category": "technical"
    },
    {
        "id": "embeddings-training-1",
        "text": "Modern embedding models are trained using techniques like contrastive learning, where the model learns to bring similar concepts closer together and push different concepts further apart in the vector space.",
        "source": "vector-embeddings-guide",
        "category": "training"
    },
    {
        "id": "embeddings-evaluation-1",
        "text": "Embedding quality is evaluated using benchmarks like semantic similarity tasks, analogy solving, and downstream task performance. Good embeddings should capture both syntactic and semantic relationships.",
        "source": "vector-embeddings-guide",
        "category": "evaluation"
    }
]

def add_educational_content():
    """Add educational content to Pinecone"""
    
    headers = {
        "Api-Key": PINECONE_API_KEY,
        "Content-Type": "application/json"
    }
    
    # For now, we'll just print the content that could be added
    # In a real implementation, you'd need to:
    # 1. Generate embeddings for each text using your embedding model
    # 2. Create proper vector objects with the embeddings
    # 3. Send them to Pinecone
    
    print("🎓 Educational Content About Vector Embeddings")
    print("=" * 60)
    
    for i, content in enumerate(EDUCATIONAL_CONTENT, 1):
        print(f"\n{i}. {content['category'].upper()}: {content['id']}")
        print(f"   Source: {content['source']}")
        print(f"   Text: {content['text'][:100]}...")
    
    print(f"\n📚 Total: {len(EDUCATIONAL_CONTENT)} educational chunks")
    print("\n💡 To add these to your Pinecone database:")
    print("   1. Use your Rust application to index these as text files")
    print("   2. Or implement direct Pinecone upsert with pre-generated embeddings")
    print("   3. This will provide much richer context for RAG responses!")

if __name__ == "__main__":
    add_educational_content()
