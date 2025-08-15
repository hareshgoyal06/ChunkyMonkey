import psycopg
from psycopg.types import TypeInfo
from pgvector.psycopg import register_vector
import os
import time

def get_conn():
    """Get a database connection with pgvector support"""
    conn = psycopg.connect(
        host="localhost",
        port=5432,
        dbname="vectordb",
        user="postgres",
        password="postgres",
        autocommit=True
    )
    
    # Register pgvector types
    try:
        register_vector(conn)
    except Exception as e:
        print(f"Warning: Could not register vector types: {e}")
        # This might happen if the extension isn't installed yet
    
    return conn

def init_db():
    """Initialize database schema"""
    max_retries = 5
    retry_delay = 2
    
    for attempt in range(max_retries):
        try:
            conn = get_conn()
            
            # First, ensure the vector extension is created
            with conn.cursor() as cur:
                cur.execute("CREATE EXTENSION IF NOT EXISTS vector;")
            
            # Now read and execute the schema
            with open('models.sql', 'r') as f:
                sql = f.read()
            
            with conn.cursor() as cur:
                cur.execute(sql)
            
            print("Database initialized successfully")
            conn.close()
            return
            
        except Exception as e:
            print(f"Attempt {attempt + 1}/{max_retries}: Error initializing database: {e}")
            if attempt < max_retries - 1:
                print(f"Retrying in {retry_delay} seconds...")
                time.sleep(retry_delay)
                retry_delay *= 2
            else:
                print("Failed to initialize database after all retries")
                raise 