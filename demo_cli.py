#!/usr/bin/env python3
"""
Demo script for CLI Vector Search Tool
Creates sample files and demonstrates the CLI capabilities
"""

import os
import tempfile
from pathlib import Path
from cli_vector_search import VectorSearchCLI

def create_sample_files():
    """Create sample files for demonstration"""
    sample_files = {
        "authentication.py": '''
# Authentication System
class UserAuth:
    def __init__(self):
        self.users = {}
    
    def login(self, username, password):
        """Authenticate user with username and password"""
        if username in self.users:
            if self.users[username] == password:
                return {"status": "success", "message": "Login successful"}
        return {"status": "error", "message": "Invalid credentials"}
    
    def register(self, username, password):
        """Register a new user"""
        if username in self.users:
            return {"status": "error", "message": "User already exists"}
        self.users[username] = password
        return {"status": "success", "message": "User registered"}
''',
        "database.py": '''
# Database Connection Manager
import sqlite3
from typing import Dict, Any

class DatabaseManager:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.connection = None
    
    def connect(self):
        """Establish database connection"""
        try:
            self.connection = sqlite3.connect(self.db_path)
            return {"status": "success", "message": "Connected to database"}
        except Exception as e:
            return {"status": "error", "message": f"Connection failed: {e}"}
    
    def execute_query(self, query: str, params: tuple = ()) -> Dict[str, Any]:
        """Execute a SQL query"""
        if not self.connection:
            return {"status": "error", "message": "No database connection"}
        
        try:
            cursor = self.connection.cursor()
            cursor.execute(query, params)
            results = cursor.fetchall()
            return {"status": "success", "data": results}
        except Exception as e:
            return {"status": "error", "message": f"Query failed: {e}"}
''',
        "api_endpoints.md": '''
# API Endpoints Documentation

## Authentication Endpoints

### POST /auth/login
Authenticate a user with username and password.

**Request Body:**
```json
{
    "username": "user@example.com",
    "password": "secure_password"
}
```

**Response:**
```json
{
    "status": "success",
    "token": "jwt_token_here"
}
```

### POST /auth/register
Register a new user account.

**Request Body:**
```json
{
    "username": "newuser@example.com",
    "password": "secure_password",
    "email": "user@example.com"
}
```

## User Management Endpoints

### GET /users/profile
Get current user profile information.

### PUT /users/profile
Update user profile information.

## Error Handling

All endpoints return consistent error responses:

```json
{
    "status": "error",
    "message": "Error description",
    "code": "ERROR_CODE"
}
```
''',
        "config.yaml": '''
# Application Configuration
app:
  name: "Vector Search Demo"
  version: "1.0.0"
  debug: true

database:
  type: "sqlite"
  path: "./data/app.db"
  pool_size: 10

authentication:
  jwt_secret: "your-secret-key"
  token_expiry: 3600
  password_min_length: 8

api:
  host: "0.0.0.0"
  port: 8000
  cors_origins:
    - "http://localhost:3000"
    - "https://app.example.com"

logging:
  level: "INFO"
  format: "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
  file: "./logs/app.log"
''',
        "README.md": '''
# Vector Search CLI Demo

This is a demonstration of the CLI Vector Search Tool capabilities.

## Features Demonstrated

1. **Semantic Search**: Find content by meaning, not just keywords
2. **RAG Pipeline**: Ask questions about your codebase
3. **Multiple File Types**: Python, Markdown, YAML, and more
4. **Interactive Interface**: Beautiful terminal UI

## Sample Queries to Try

### Search Examples:
- "authentication login function"
- "database connection management"
- "API endpoint documentation"
- "configuration settings"

### RAG Questions:
- "How does the authentication system work?"
- "What database features are available?"
- "Explain the API structure"
- "What configuration options exist?"

## Getting Started

1. Run the CLI: `python cli_vector_search.py interactive`
2. Index this directory: Select "Index Directory" and choose this folder
3. Start searching and asking questions!

## File Structure

- `authentication.py`: User authentication system
- `database.py`: Database connection manager
- `api_endpoints.md`: API documentation
- `config.yaml`: Application configuration
- `README.md`: This file

Enjoy exploring the power of semantic search! 🚀
'''
    }
    
    # Create temp directory
    temp_dir = tempfile.mkdtemp(prefix="vector_search_demo_")
    print(f"📁 Created demo directory: {temp_dir}")
    
    # Create sample files
    for filename, content in sample_files.items():
        file_path = Path(temp_dir) / filename
        with open(file_path, 'w') as f:
            f.write(content.strip())
        print(f"📄 Created: {filename}")
    
    return temp_dir

def run_demo():
    """Run the CLI demo"""
    print("🚀 CLI Vector Search Tool Demo")
    print("=" * 50)
    
    # Create sample files
    demo_dir = create_sample_files()
    
    print(f"\n📂 Demo files created in: {demo_dir}")
    print("\n🎯 Now you can:")
    print("1. Run: python cli_vector_search.py interactive")
    print("2. Select 'Index Directory' and choose the demo directory")
    print("3. Try searching for: 'authentication', 'database', 'API'")
    print("4. Ask questions like: 'How does login work?'")
    
    print(f"\n💡 Demo directory will be cleaned up automatically")
    print("   (You can also manually delete: {demo_dir})")
    
    return demo_dir

if __name__ == "__main__":
    run_demo() 