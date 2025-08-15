#!/usr/bin/env python3
"""
CLI Vector Search Tool
Beautiful interactive semantic search through any directory of files
"""

import os
import sys
import json
import sqlite3
from pathlib import Path
from typing import List, Dict, Any, Optional
import click
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt, Confirm, IntPrompt
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.syntax import Syntax
from rich.markdown import Markdown
from rich.layout import Layout
from rich.live import Live
from rich.align import Align
from rich.text import Text
from rich.padding import Padding
from rich.columns import Columns
from rich.rule import Rule
from rich.box import ROUNDED
import psycopg
from pgvector.psycopg import register_vector
from sentence_transformers import SentenceTransformer
import umap
import numpy as np
import hashlib
import time

console = Console()

class VectorSearchCLI:
    def __init__(self, db_path: str = "vector_search.db"):
        self.db_path = db_path
        self.model = None
        self.conn = None
        self.collection_id = 1
        self.current_directory = None
        self.is_initialized = False
        
    def init_model(self):
        """Initialize the embedding model"""
        if self.model is None:
            with console.status("[bold green]Loading AI model...", spinner="dots"):
                self.model = SentenceTransformer('all-MiniLM-L6-v2')
            console.print("✅ [green]AI Model loaded successfully[/green]")
    
    def init_database(self):
        """Initialize database"""
        try:
            # Try PostgreSQL first
            self.conn = psycopg.connect(
                host="localhost", port=5432, dbname="vectordb",
                user="postgres", password="postgres", autocommit=True
            )
            register_vector(self.conn)
            console.print("✅ [green]Connected to PostgreSQL with pgvector[/green]")
            return
        except:
            pass
        
        # Fallback to SQLite
        self.conn = sqlite3.connect(self.db_path)
        self.conn.execute("PRAGMA foreign_keys = ON")
        
        # Create tables
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)
        
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                metadata TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (collection_id) REFERENCES collections (id)
            )
        """)
        
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                document_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                metadata TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (collection_id) REFERENCES collections (id),
                FOREIGN KEY (document_id) REFERENCES documents (id)
            )
        """)
        
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                chunk_id INTEGER NOT NULL,
                vector TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (chunk_id) REFERENCES chunks (id)
            )
        """)
        
        self.conn.execute("""
            INSERT OR IGNORE INTO collections (id, name) VALUES (1, 'default')
        """)
        
        self.conn.commit()
        console.print("✅ [green]SQLite database initialized[/green]")
    
    def get_file_hash(self, file_path: str) -> str:
        """Get SHA256 hash of file content"""
        with open(file_path, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    
    def is_file_indexed(self, file_path: str) -> bool:
        """Check if file is already indexed"""
        file_hash = self.get_file_hash(file_path)
        cursor = self.conn.cursor()
        cursor.execute("""
            SELECT id FROM documents 
            WHERE file_path = ? AND file_hash = ?
        """, (file_path, file_hash))
        return cursor.fetchone() is not None
    
    def chunk_text(self, text: str, chunk_size: int = 800, overlap: int = 150) -> List[str]:
        """Split text into overlapping chunks"""
        if not text:
            return []
        
        chunks = []
        start = 0
        
        while start < len(text):
            end = start + chunk_size
            
            if end < len(text):
                last_space = text.rfind(' ', start, end)
                if last_space > start + chunk_size // 2:
                    end = last_space
            
            chunk = text[start:end].strip()
            if chunk:
                chunks.append(chunk)
            
            start = end - overlap
            if start >= len(text):
                break
        
        return chunks
    
    def embed_texts(self, texts: List[str]) -> List[List[float]]:
        """Create embeddings for a list of texts"""
        if not texts:
            return []
        
        embeddings = self.model.encode(texts, normalize_embeddings=True)
        return embeddings.tolist()
    
    def index_directory(self, directory: str, file_patterns: List[str] = None):
        """Index all files in a directory"""
        if file_patterns is None:
            file_patterns = ['*.txt', '*.md', '*.py', '*.js', '*.ts', '*.jsx', '*.tsx', '*.json', '*.yaml', '*.yml']
        
        directory_path = Path(directory)
        if not directory_path.exists():
            console.print(f"[red]❌ Directory {directory} does not exist[/red]")
            return
        
        files_to_index = []
        for pattern in file_patterns:
            files_to_index.extend(directory_path.rglob(pattern))
        
        files_to_index = [f for f in files_to_index if not self.is_file_indexed(str(f))]
        
        if not files_to_index:
            console.print("✅ [green]All files are already indexed[/green]")
            return
        
        console.print(f"📁 [blue]Found {len(files_to_index)} files to index[/blue]")
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console
        ) as progress:
            task = progress.add_task("Indexing files...", total=len(files_to_index))
            
            for file_path in files_to_index:
                progress.update(task, description=f"Indexing {file_path.name}")
                
                try:
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                    
                    if not content.strip():
                        continue
                    
                    chunks = self.chunk_text(content)
                    if not chunks:
                        continue
                    
                    embeddings = self.embed_texts(chunks)
                    
                    cursor = self.conn.cursor()
                    
                    file_hash = self.get_file_hash(str(file_path))
                    cursor.execute("""
                        INSERT INTO documents (collection_id, file_path, file_hash, metadata)
                        VALUES (?, ?, ?, ?)
                    """, (self.collection_id, str(file_path), file_hash, json.dumps({
                        'size': len(content),
                        'chunks': len(chunks)
                    })))
                    document_id = cursor.lastrowid
                    
                    for i, (chunk, embedding) in enumerate(zip(chunks, embeddings)):
                        cursor.execute("""
                            INSERT INTO chunks (collection_id, document_id, text, chunk_index, metadata)
                            VALUES (?, ?, ?, ?, ?)
                        """, (self.collection_id, document_id, chunk, i, json.dumps({})))
                        chunk_id = cursor.lastrowid
                        
                        cursor.execute("""
                            INSERT INTO embeddings (chunk_id, vector)
                            VALUES (?, ?)
                        """, (chunk_id, json.dumps(embedding)))
                    
                    self.conn.commit()
                    
                except Exception as e:
                    console.print(f"[red]❌ Error indexing {file_path}: {e}[/red]")
                
                progress.advance(task)
        
        console.print("✅ [green]Directory indexing completed![/green]")
        self.current_directory = directory
    
    def search(self, query: str, top_k: int = 5) -> List[Dict[str, Any]]:
        """Search for similar chunks"""
        query_embedding = self.embed_texts([query])[0]
        
        cursor = self.conn.cursor()
        
        cursor.execute("""
            SELECT 
                c.id,
                c.text,
                d.file_path,
                c.chunk_index,
                e.vector
            FROM chunks c
            JOIN documents d ON c.document_id = d.id
            JOIN embeddings e ON c.chunk_id = e.id
            WHERE c.collection_id = ?
        """, (self.collection_id,))
        
        results = []
        query_vec = np.array(query_embedding)
        
        for row in cursor.fetchall():
            chunk_id, text, file_path, chunk_index, vector_json = row
            chunk_vec = np.array(json.loads(vector_json))
            
            similarity = np.dot(query_vec, chunk_vec) / (np.linalg.norm(query_vec) * np.linalg.norm(chunk_vec))
            
            results.append({
                'chunk_id': chunk_id,
                'text': text,
                'file_path': file_path,
                'chunk_index': chunk_index,
                'similarity': float(similarity)
            })
        
        results.sort(key=lambda x: x['similarity'], reverse=True)
        return results[:top_k]
    
    def rag_query(self, query: str, top_k: int = 5) -> Dict[str, Any]:
        """RAG-style question answering"""
        results = self.search(query, top_k)
        
        if not results:
            return {
                'query': query,
                'answer': 'No relevant information found.',
                'sources': []
            }
        
        context = "\n\n".join([r['text'] for r in results])
        answer = f"Based on the retrieved information:\n\n{context}"
        
        return {
            'query': query,
            'answer': answer,
            'sources': results
        }
    
    def get_stats(self) -> Dict[str, int]:
        """Get database statistics"""
        cursor = self.conn.cursor()
        
        cursor.execute("SELECT COUNT(*) FROM documents WHERE collection_id = ?", (self.collection_id,))
        doc_count = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM chunks WHERE collection_id = ?", (self.collection_id,))
        chunk_count = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM embeddings")
        embedding_count = cursor.fetchone()[0]
        
        return {
            'documents': doc_count,
            'chunks': chunk_count,
            'embeddings': embedding_count
        }
    
    def show_welcome_screen(self):
        """Display welcome screen"""
        welcome_text = Text()
        welcome_text.append("🔍 ", style="bold blue")
        welcome_text.append("Vector Search CLI", style="bold white")
        welcome_text.append("\n\n", style="white")
        welcome_text.append("Semantic search through any directory of files\n", style="cyan")
        welcome_text.append("Powered by AI embeddings and RAG technology", style="dim white")
        
        console.print(Panel(
            Align.center(welcome_text),
            border_style="blue",
            box=ROUNDED,
            padding=(1, 2)
        ))
    
    def show_main_menu(self):
        """Display main menu"""
        stats = self.get_stats()
        
        menu_items = [
            ("📁 Index Directory", "index"),
            ("🔍 Search Content", "search"),
            ("❓ Ask Questions", "ask"),
            ("📊 View Statistics", "stats"),
            ("🔄 Re-index Current", "reindex"),
            ("🧹 Clear Database", "clear"),
            ("❌ Exit", "exit")
        ]
        
        table = Table(show_header=False, box=ROUNDED, border_style="blue")
        table.add_column("Option", style="cyan", width=20)
        table.add_column("Description", style="white")
        
        for i, (desc, _) in enumerate(menu_items, 1):
            table.add_row(f"{i}. {desc.split()[0]}", desc)
        
        console.print("\n")
        console.print(Panel(
            table,
            title="[bold blue]Main Menu[/bold blue]",
            border_style="blue",
            box=ROUNDED
        ))
        
        if self.current_directory:
            console.print(f"📂 [green]Current directory:[/green] {self.current_directory}")
        
        if stats['documents'] > 0:
            console.print(f"📊 [yellow]Indexed:[/yellow] {stats['documents']} files, {stats['chunks']} chunks")
        
        return menu_items
    
    def show_search_interface(self):
        """Interactive search interface"""
        console.clear()
        console.print(Panel(
            "[bold blue]🔍 Search Interface[/bold blue]\n"
            "Enter your search query to find similar content",
            border_style="blue",
            box=ROUNDED
        ))
        
        while True:
            query = Prompt.ask("\n[bold yellow]Search query[/bold yellow] (or 'back' to return)")
            
            if query.lower() == 'back':
                break
            
            if not query.strip():
                continue
            
            top_k = IntPrompt.ask("Number of results", default=5)
            
            with console.status("[bold green]Searching...", spinner="dots"):
                results = self.search(query, top_k)
            
            if not results:
                console.print("[yellow]No results found[/yellow]")
                continue
            
            self.display_search_results(query, results)
            
            if not Confirm.ask("Search again?"):
                break
    
    def show_ask_interface(self):
        """Interactive RAG interface"""
        console.clear()
        console.print(Panel(
            "[bold purple]❓ RAG Question Interface[/bold purple]\n"
            "Ask questions about your indexed content",
            border_style="purple",
            box=ROUNDED
        ))
        
        while True:
            question = Prompt.ask("\n[bold purple]Your question[/bold purple] (or 'back' to return)")
            
            if question.lower() == 'back':
                break
            
            if not question.strip():
                continue
            
            top_k = IntPrompt.ask("Number of chunks to use", default=3)
            
            with console.status("[bold green]Generating answer...", spinner="dots"):
                result = self.rag_query(question, top_k)
            
            self.display_rag_answer(question, result)
            
            if not Confirm.ask("Ask another question?"):
                break
    
    def display_search_results(self, query: str, results: List[Dict[str, Any]]):
        """Display search results beautifully"""
        console.print(f"\n[bold green]Search Results for:[/bold green] '{query}'")
        console.print(Rule(style="blue"))
        
        for i, result in enumerate(results, 1):
            file_name = Path(result['file_path']).name
            similarity = f"{result['similarity']:.3f}"
            text = result['text'][:150] + "..." if len(result['text']) > 150 else result['text']
            
            console.print(Panel(
                f"[bold cyan]{i}.[/bold cyan] [green]{file_name}[/green] "
                f"[yellow]({similarity})[/yellow]\n\n{text}",
                border_style="green" if i == 1 else "blue",
                box=ROUNDED,
                padding=(0, 1)
            ))
        
        # Option to view full text
        if Confirm.ask("\n[bold]View full text of a result?"):
            try:
                rank = IntPrompt.ask("Enter result number") - 1
                if 0 <= rank < len(results):
                    result = results[rank]
                    console.print(Panel(
                        Syntax(result['text'], "text", theme="monokai"),
                        title=f"📄 {Path(result['file_path']).name}",
                        border_style="green",
                        box=ROUNDED
                    ))
            except (ValueError, IndexError):
                console.print("[red]Invalid result number[/red]")
    
    def display_rag_answer(self, question: str, result: Dict[str, Any]):
        """Display RAG answer beautifully"""
        console.print(f"\n[bold purple]Question:[/bold purple] {question}")
        console.print(Rule(style="purple"))
        
        console.print(Panel(
            Markdown(result['answer']),
            title="🤖 AI Answer",
            border_style="purple",
            box=ROUNDED
        ))
        
        if result['sources']:
            console.print("\n[bold yellow]Sources used:[/bold yellow]")
            for i, source in enumerate(result['sources'], 1):
                file_name = Path(source['file_path']).name
                similarity = f"{source['similarity']:.3f}"
                text = source['text'][:100] + "..." if len(source['text']) > 100 else source['text']
                
                console.print(f"  {i}. [green]{file_name}[/green] [yellow]({similarity})[/yellow]")
                console.print(f"     {text}")
                console.print()
    
    def show_stats_interface(self):
        """Display statistics interface"""
        console.clear()
        stats = self.get_stats()
        
        table = Table(title="📊 Database Statistics", box=ROUNDED, border_style="blue")
        table.add_column("Metric", style="cyan", justify="left")
        table.add_column("Count", style="magenta", justify="right")
        
        table.add_row("📄 Documents", str(stats['documents']))
        table.add_row("📝 Chunks", str(stats['chunks']))
        table.add_row("🧠 Embeddings", str(stats['embeddings']))
        
        console.print(Panel(
            table,
            border_style="blue",
            box=ROUNDED
        ))
        
        if self.current_directory:
            console.print(f"\n📂 [green]Indexed directory:[/green] {self.current_directory}")
        
        Prompt.ask("\n[bold]Press Enter to continue")
    
    def show_index_interface(self):
        """Interactive indexing interface"""
        console.clear()
        console.print(Panel(
            "[bold green]📁 Index Directory[/bold green]\n"
            "Index files for semantic search",
            border_style="green",
            box=ROUNDED
        ))
        
        directory = Prompt.ask("\n[bold yellow]Directory path[/bold yellow]")
        
        if not directory or not Path(directory).exists():
            console.print("[red]❌ Invalid directory path[/red]")
            Prompt.ask("\n[bold]Press Enter to continue")
            return
        
        # File patterns
        patterns = Prompt.ask(
            "[bold cyan]File patterns[/bold cyan] (comma-separated)",
            default="*.txt,*.md,*.py,*.js,*.ts,*.jsx,*.tsx,*.json,*.yaml,*.yml"
        )
        file_patterns = [p.strip() for p in patterns.split(",")]
        
        console.print(f"\n[blue]Indexing directory:[/blue] {directory}")
        console.print(f"[blue]File patterns:[/blue] {', '.join(file_patterns)}")
        
        if Confirm.ask("\n[bold]Proceed with indexing?"):
            self.index_directory(directory, file_patterns)
        
        Prompt.ask("\n[bold]Press Enter to continue")
    
    def clear_database(self):
        """Clear the database"""
        if Confirm.ask("[bold red]Are you sure you want to clear all indexed data?"):
            cursor = self.conn.cursor()
            cursor.execute("DELETE FROM embeddings")
            cursor.execute("DELETE FROM chunks")
            cursor.execute("DELETE FROM documents")
            self.conn.commit()
            console.print("✅ [green]Database cleared[/green]")
            self.current_directory = None
    
    def run_interactive(self):
        """Run the main interactive loop"""
        console.clear()
        self.show_welcome_screen()
        
        # Initialize if needed
        if not self.is_initialized:
            with console.status("[bold green]Initializing...", spinner="dots"):
                self.init_model()
                self.init_database()
            self.is_initialized = True
        
        while True:
            console.clear()
            self.show_welcome_screen()
            menu_items = self.show_main_menu()
            
            try:
                choice = Prompt.ask("\n[bold cyan]Select option[/bold cyan]", choices=[str(i) for i in range(1, len(menu_items) + 1)])
                choice_idx = int(choice) - 1
                
                if choice_idx < 0 or choice_idx >= len(menu_items):
                    continue
                
                _, action = menu_items[choice_idx]
                
                if action == "exit":
                    console.print("\n👋 [green]Goodbye![/green]")
                    break
                elif action == "index":
                    self.show_index_interface()
                elif action == "search":
                    self.show_search_interface()
                elif action == "ask":
                    self.show_ask_interface()
                elif action == "stats":
                    self.show_stats_interface()
                elif action == "reindex":
                    if self.current_directory:
                        if Confirm.ask(f"Re-index {self.current_directory}?"):
                            self.index_directory(self.current_directory)
                    else:
                        console.print("[yellow]No directory currently indexed[/yellow]")
                        Prompt.ask("\n[bold]Press Enter to continue")
                elif action == "clear":
                    self.clear_database()
                    Prompt.ask("\n[bold]Press Enter to continue")
                    
            except KeyboardInterrupt:
                console.print("\n👋 [green]Goodbye![/green]")
                break
            except Exception as e:
                console.print(f"[red]❌ Error: {e}[/red]")
                Prompt.ask("\n[bold]Press Enter to continue")

@click.group()
def cli():
    """CLI Vector Search Tool - Beautiful interactive semantic search"""
    pass

@cli.command()
def interactive():
    """Start the beautiful interactive interface"""
    vs = VectorSearchCLI()
    vs.run_interactive()

@cli.command()
@click.argument('directory')
@click.option('--patterns', '-p', multiple=True, default=['*.txt', '*.md', '*.py'], 
              help='File patterns to index')
def index(directory, patterns):
    """Index a directory of files"""
    vs = VectorSearchCLI()
    vs.init_model()
    vs.init_database()
    vs.index_directory(directory, list(patterns))

@cli.command()
@click.argument('query')
@click.option('--top-k', '-k', default=5, help='Number of results to return')
def search(query, top_k):
    """Search for similar content"""
    vs = VectorSearchCLI()
    vs.init_model()
    vs.init_database()
    results = vs.search(query, top_k)
    vs.display_search_results(query, results)

@cli.command()
@click.argument('question')
@click.option('--top-k', '-k', default=3, help='Number of chunks to use for answer')
def ask(question, top_k):
    """Ask a question about the indexed content"""
    vs = VectorSearchCLI()
    vs.init_model()
    vs.init_database()
    result = vs.rag_query(question, top_k)
    vs.display_rag_answer(question, result)

@cli.command()
def stats():
    """Show database statistics"""
    vs = VectorSearchCLI()
    vs.init_database()
    vs.show_stats_interface()

if __name__ == '__main__':
    cli() 