use clap::{Parser, Subcommand};
use colored::*;
use std::io::Write;
use std::path::PathBuf;

mod cli;
mod core;
mod db;
mod embeddings;
mod search;
mod ui;

use cli::interactive::run_interactive;
use core::app::TldrApp;
use search::Indexer;

#[derive(Parser)]
#[command(
    name = "tldr",
    about = "TLDR - Too Long; Didn't Read. Blazing-fast semantic search through any directory.",
    version,
    long_about = "A powerful CLI tool for semantic search through codebases, documentation, and text files using vector embeddings."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directory to work with
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Database path
    #[arg(short, long, default_value = "tldr.db")]
    db: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a directory for semantic search
    Index {
        /// Directory to index
        path: PathBuf,
        
        /// File patterns to include (e.g., "*.rs,*.md,*.txt")
        #[arg(short, long, default_value = "*.rs,*.md,*.txt,*.py,*.js,*.ts,*.json,*.yaml,*.yml")]
        patterns: String,
        
        /// Chunk size in characters
        #[arg(short, long, default_value = "800")]
        chunk_size: usize,
        
        /// Overlap between chunks
        #[arg(short, long, default_value = "150")]
        overlap: usize,
    },
    
    /// Search for content semantically
    Search {
        /// Search query
        query: String,
        
        /// Number of results to return
        #[arg(short, long, default_value = "5")]
        limit: usize,
        
        /// Minimum similarity score (0.0-1.0)
        #[arg(short, long, default_value = "0.3")]
        threshold: f32,
    },
    
    /// Ask questions about indexed content (RAG)
    Ask {
        /// Question to ask
        question: String,
        
        /// Number of chunks to use for context
        #[arg(short, long, default_value = "3")]
        context_chunks: usize,
    },
    
    /// Show database statistics
    Stats,
    
    /// Clear the database
    Clear,
    
    /// Start interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable colored output
    colored::control::set_override(true);
    
    let cli = Cli::parse();
    
    // Initialize the application
    let mut app = TldrApp::new(cli.db.unwrap_or_else(|| PathBuf::from("tldr.db"))).await?;
    
    match &cli.command {
        Some(Commands::Index { path, patterns, chunk_size, overlap }) => {
            println!("{}", "🔍 TLDR - Indexing Directory".bold().blue());
            println!("📁 Directory: {}", path.display());
            println!("🎯 Patterns: {}", patterns);
            
            let indexer = Indexer::new(*chunk_size, *overlap);
            indexer.index_directory(&mut app, path, patterns).await?;
            
            println!("{}", "✅ Indexing completed!".green());
        }
        
        Some(Commands::Search { query, limit, threshold }) => {
            println!("{}", "🔍 TLDR - Semantic Search".bold().blue());
            println!("🔎 Query: {}", query);
            
            let results = app.search(query, *limit, *threshold).await?;
            
            if results.is_empty() {
                println!("{}", "❌ No results found".yellow());
            } else {
                println!("\n{}", "📋 Search Results:".bold());
                for (i, result) in results.iter().enumerate() {
                    println!(
                        "{}. {} (similarity: {:.3})",
                        i + 1,
                        result.file_path.display(),
                        result.similarity
                    );
                    println!("   {}", result.text.lines().next().unwrap_or("").trim());
                    println!();
                }
            }
        }
        
        Some(Commands::Ask { question, context_chunks }) => {
            println!("{}", "❓ TLDR - RAG Question".bold().purple());
            println!("🤔 Question: {}", question);
            
            let answer = app.ask_question(question, *context_chunks).await?;
            
            println!("\n{}", "🤖 Answer:".bold());
            println!("{}", answer.text);
            
            if !answer.sources.is_empty() {
                println!("\n{}", "📚 Sources:".bold());
                for source in &answer.sources {
                    println!("• {} (similarity: {:.3})", source.file_path.display(), source.similarity);
                }
            }
        }
        
        Some(Commands::Stats) => {
            println!("{}", "📊 TLDR - Database Statistics".bold().cyan());
            
            let stats = app.get_stats().await?;
            println!("📄 Documents: {}", stats.documents);
            println!("📝 Chunks: {}", stats.chunks);
            println!("🧠 Embeddings: {}", stats.embeddings);
            println!("💾 Database size: {:.2} MB", stats.db_size_mb);
        }
        
        Some(Commands::Clear) => {
            println!("{}", "🧹 TLDR - Clear Database".bold().red());
            
            print!("Are you sure you want to clear all indexed data? (y/N): ");
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                app.clear_database().await?;
                println!("{}", "✅ Database cleared".green());
            } else {
                println!("{}", "❌ Operation cancelled".yellow());
            }
        }
        
        Some(Commands::Interactive) => {
            run_interactive(&mut app).await?;
        }
        
        None => {
            // No command specified, show help
            println!("{}", "🔍 TLDR - Too Long; Didn't Read".bold().blue());
            println!("Blazing-fast semantic search through any directory\n");
            
            println!("{}", "Quick Start:".bold());
            println!("  tldr index /path/to/your/project");
            println!("  tldr search \"authentication function\"");
            println!("  tldr ask \"How does the API work?\"");
            println!("  tldr interactive");
            
            println!("\n{}", "For more information:".bold());
            println!("  tldr --help");
        }
    }
    
    Ok(())
} 