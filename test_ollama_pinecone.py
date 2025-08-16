#!/usr/bin/env python3
"""
Ollama + Pinecone Integration Test Script
This script tests both Ollama embeddings and Pinecone integration.
"""

import requests
import json
import sys
import os

# Your Pinecone configuration
PINECONE_API_KEY = "pcsk_4bSFWF_KpC7MSD3uKP9p13zGgd5HVgGgAB7ioqaCaeP3f4uT49CYNFcGnzgPD8aDRHW72E"
PINECONE_HOST = "https://chunky-monkey-test-a2r60ad.svc.aped-4627-b74a.pinecone.io"
OLLAMA_BASE_URL = "http://localhost:11434"
OLLAMA_MODEL = "nomic-embed-text:latest"  # Using the available embedding model

def test_ollama_embeddings():
    """Test Ollama embeddings generation"""
    print("🔍 Testing Ollama Embeddings...")
    print(f"Base URL: {OLLAMA_BASE_URL}")
    print(f"Model: {OLLAMA_MODEL}")
    print("-" * 50)
    
    try:
        # Test Ollama connection
        response = requests.get(f"{OLLAMA_BASE_URL}/api/tags")
        if response.status_code == 200:
            models = response.json()
            print(f"✅ Ollama connection successful!")
            print(f"Available models: {[m['name'] for m in models.get('models', [])]}")
        else:
            print(f"❌ Ollama connection failed: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Ollama connection error: {e}")
        return False
    
    # Test embedding generation
    try:
        embedding_request = {
            "model": OLLAMA_MODEL,
            "prompt": "This is a test sentence for embedding generation."
        }
        
        response = requests.post(
            f"{OLLAMA_BASE_URL}/api/embeddings",
            json=embedding_request
        )
        
        if response.status_code == 200:
            embedding_data = response.json()
            embedding = embedding_data.get('embedding', [])
            print(f"✅ Embedding generated successfully!")
            print(f"   Dimensions: {len(embedding)}")
            print(f"   First 5 values: {embedding[:5]}")
            return True
        else:
            print(f"❌ Embedding generation failed: {response.status_code} - {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Embedding generation error: {e}")
        return False

def test_pinecone_connection():
    """Test Pinecone connection with custom host"""
    print("\n🔍 Testing Pinecone Connection...")
    print(f"Host: {PINECONE_HOST}")
    print(f"API Key: {PINECONE_API_KEY[:20]}...{PINECONE_API_KEY[-4:]}")
    print("-" * 50)
    
    try:
        # Test index stats
        response = requests.get(
            f"{PINECONE_HOST}/describe_index_stats",
            headers={"Api-Key": PINECONE_API_KEY}
        )
        
        if response.status_code == 200:
            stats = response.json()
            print(f"✅ Pinecone connection successful!")
            print(f"   Total vectors: {stats.get('total_vector_count', 'Unknown')}")
            print(f"   Namespaces: {list(stats.get('namespaces', {}).keys())}")
            return True
        else:
            print(f"❌ Pinecone connection failed: {response.status_code} - {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Pinecone connection error: {e}")
        return False

def test_pinecone_upsert():
    """Test Pinecone vector upsert"""
    print("\n🔍 Testing Pinecone Vector Upsert...")
    
    # First generate an embedding
    try:
        embedding_request = {
            "model": OLLAMA_MODEL,
            "prompt": "Test vector for Pinecone upsert."
        }
        
        response = requests.post(
            f"{OLLAMA_BASE_URL}/api/embeddings",
            json=embedding_request
        )
        
        if response.status_code != 200:
            print(f"❌ Failed to generate embedding for test: {response.status_code}")
            return False
            
        embedding_data = response.json()
        embedding = embedding_data.get('embedding', [])
        
        # Test upsert to Pinecone
        upsert_request = {
            "vectors": [
                {
                    "id": "test-vector-1",
                    "values": embedding,
                    "metadata": {
                        "text": "Test vector for Pinecone upsert.",
                        "source": "test-script"
                    }
                }
            ]
        }
        
        response = requests.post(
            f"{PINECONE_HOST}/vectors/upsert",
            headers={
                "Api-Key": PINECONE_API_KEY,
                "Content-Type": "application/json"
            },
            json=upsert_request
        )
        
        if response.status_code == 200:
            print(f"✅ Vector upsert successful!")
            return True
        else:
            print(f"❌ Vector upsert failed: {response.status_code} - {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Vector upsert error: {e}")
        return False

def main():
    print("🚀 Ollama + Pinecone Integration Test")
    print("=" * 60)
    
    # Test Ollama
    ollama_ok = test_ollama_embeddings()
    
    # Test Pinecone
    pinecone_ok = test_pinecone_connection()
    
    # Test integration
    if ollama_ok and pinecone_ok:
        integration_ok = test_pinecone_upsert()
    else:
        integration_ok = False
    
    print("\n" + "=" * 60)
    print("📋 Test Results Summary:")
    print(f"   Ollama Embeddings: {'✅ PASS' if ollama_ok else '❌ FAIL'}")
    print(f"   Pinecone Connection: {'✅ PASS' if pinecone_ok else '❌ FAIL'}")
    print(f"   Integration Test: {'✅ PASS' if integration_ok else '❌ FAIL'}")
    
    if ollama_ok and pinecone_ok and integration_ok:
        print("\n🎉 All tests passed! Your Ollama + Pinecone setup is working correctly.")
        print("You can now run: cargo run -- index demo")
    else:
        print("\n⚠️  Some tests failed. Please check the issues above.")
        
        if not ollama_ok:
            print("\n🔧 Ollama Troubleshooting:")
            print("   1. Make sure Ollama is running: ollama serve")
            print("   2. Check if your model is available: ollama list")
            print("   3. Verify the model name in config.toml")
            
        if not pinecone_ok:
            print("\n🔧 Pinecone Troubleshooting:")
            print("   1. Check your API key in Pinecone console")
            print("   2. Verify the host URL is correct")
            print("   3. Ensure your index is ready and accessible")

if __name__ == "__main__":
    main() 