#!/usr/bin/env python3
"""
Pinecone Connection Test Script
This script helps troubleshoot Pinecone integration issues.
"""

import requests
import json
import sys

# Your Pinecone configuration
API_KEY = "pcsk_4bSFWF_KpC7MSD3uKP9p13zGgd5HVgGgAB7ioqaCaeP3f4uT49CYNFcGnzgPD8aDRHW72E"
ENVIRONMENT = "chunky-monkey-test"
INDEX_NAME = "aped-4627-b74a"

def test_pinecone_connection():
    """Test basic Pinecone connectivity"""
    print("🔍 Testing Pinecone Connection...")
    print(f"Environment: {ENVIRONMENT}")
    print(f"Index: {INDEX_NAME}")
    print(f"API Key: {API_KEY[:20]}...{API_KEY[-4:]}")
    print("-" * 50)
    
    # Test 1: List databases
    print("\n1️⃣ Testing: List databases")
    try:
        url = f"https://controller.{ENVIRONMENT}.pinecone.io/databases"
        response = requests.get(url, headers={"Api-Key": API_KEY})
        
        if response.status_code == 200:
            databases = response.json()
            print(f"✅ Success! Found {len(databases)} databases:")
            for db in databases:
                print(f"   - {db.get('name', 'Unknown')} (Status: {db.get('status', 'Unknown')})")
        else:
            print(f"❌ Failed with status {response.status_code}: {response.text}")
            
    except Exception as e:
        print(f"❌ Error: {e}")
    
    # Test 2: Check specific index
    print("\n2️⃣ Testing: Check specific index")
    try:
        url = f"https://controller.{ENVIRONMENT}.pinecone.io/databases/{INDEX_NAME}"
        response = requests.get(url, headers={"Api-Key": API_KEY})
        
        if response.status_code == 200:
            index_info = response.json()
            print(f"✅ Index found!")
            print(f"   Name: {index_info.get('name', 'Unknown')}")
            print(f"   Status: {index_info.get('status', 'Unknown')}")
            print(f"   Dimensions: {index_info.get('dimension', 'Unknown')}")
            print(f"   Metric: {index_info.get('metric', 'Unknown')}")
        else:
            print(f"❌ Index not found or error: {response.status_code} - {response.text}")
            
    except Exception as e:
        print(f"❌ Error: {e}")
    
    # Test 3: Test index operations
    print("\n3️⃣ Testing: Index operations")
    try:
        url = f"https://{INDEX_NAME}-0.{ENVIRONMENT}.svc.pinecone.io/describe_index_stats"
        response = requests.get(url, headers={"Api-Key": API_KEY})
        
        if response.status_code == 200:
            stats = response.json()
            print(f"✅ Index operations working!")
            print(f"   Total vector count: {stats.get('total_vector_count', 'Unknown')}")
            print(f"   Namespaces: {list(stats.get('namespaces', {}).keys())}")
        else:
            print(f"❌ Index operations failed: {response.status_code} - {response.text}")
            
    except Exception as e:
        print(f"❌ Error: {e}")

def check_environment_format():
    """Check if environment format is correct"""
    print("\n4️⃣ Environment Format Check")
    print("Common Pinecone environments:")
    print("   - us-west1-gcp")
    print("   - us-east1-gcp") 
    print("   - us-central1-gcp")
    print("   - eu-west1-aws")
    print("   - ap-southeast1-aws")
    print(f"\nYour environment: {ENVIRONMENT}")
    
    if "gcp" in ENVIRONMENT or "aws" in ENVIRONMENT:
        print("✅ Environment format looks correct")
    else:
        print("⚠️  Environment format might be incorrect")
        print("   Expected format: region-provider (e.g., us-west1-gcp)")

def main():
    print("🚀 Pinecone Troubleshooting Tool")
    print("=" * 50)
    
    test_pinecone_connection()
    check_environment_format()
    
    print("\n" + "=" * 50)
    print("📋 Troubleshooting Summary:")
    print("1. Check if your Pinecone account is active")
    print("2. Verify the API key hasn't expired")
    print("3. Ensure the environment format is correct")
    print("4. Confirm the index exists and is ready")
    print("5. Check if you have the right permissions")

if __name__ == "__main__":
    main() 