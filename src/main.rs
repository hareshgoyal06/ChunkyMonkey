use clap::{Parser, Subcommand};

use std::io::{self, Write};
use std::path::PathBuf;

use crate::core::app::TldrApp;
use crate::core::config::AppConfig;
use crate::search::Indexer;

mod cli;
mod core;
mod db;
mod embeddings;
mod pinecone;
mod search;

#[derive(Parser)]
#[command(name = "tldr")]
#[command(about = "Too Long; Didn't Read - Semantic Search Made Simple")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a directory for semantic search
    Index {
        /// Directory to index
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,
        
        /// File patterns to include (e.g., "*.txt", "*.md")
        #[arg(short, long, value_delimiter = ',')]
        patterns: Option<Vec<String>>,
    },
    
    /// Search for similar content
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
        
        /// Similarity threshold (0.0 to 1.0)
        #[arg(short, long, default_value = "0.3")]
        threshold: f32,
    },
    
    /// Ask a question about indexed content
    Ask {
        /// Question to ask
        #[arg(value_name = "QUESTION")]
        question: String,
        
        /// Number of context chunks to use
        #[arg(short, long, default_value = "3")]
        context_chunks: usize,
    },
    
    /// Show database statistics
    Stats,
    
    /// Clear all indexed data
    Clear,
    
    /// Recreate database schema (fixes schema issues)
    RecreateSchema,
    
    /// Interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    colored::control::set_override(true);
    
    let cli = Cli::parse();
    
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Initialize the application
    let mut app = TldrApp::new(
        "tldr.db", // Use default database path
        config.ollama,  // Pass Ollama config instead of OpenAI API key
        config.pinecone,
    ).await?;
    
    match cli.command {
        Commands::Index { directory, patterns } => {
            let patterns = patterns.unwrap_or_else(|| vec!["*".to_string()]);
            let indexer = Indexer::new(patterns);
            indexer.index_directory(&directory, &mut app).await?;
        }
        
        Commands::Search { query, limit, threshold: _ } => {
            let results = app.search_similar(&query, limit).await?;
            if results.is_empty() {
                println!("❌ No results found for: {}", query);
            } else {
                println!("🔍 Found {} results for: {}", results.len(), query);
                for (i, result) in results.iter().enumerate() {
                    println!("\n{}. {} (score: {:.3})", i + 1, result.file_path, result.score);
                    println!("   {}", result.text.chars().take(100).collect::<String>());
                }
            }
        }
        
        Commands::Ask { question, context_chunks } => {
            println!("❓ Question: {}", question);
            println!("⏳ Generating answer...");
            
            match app.ask_question(&question, context_chunks).await {
                Ok(answer) => {
                                    println!("✅ Answer: {}", answer.text);
                println!("📚 Sources: {}", answer.sources.iter().map(|s| s.file_path.clone()).collect::<Vec<_>>().join(", "));
                    println!("🎯 Confidence: {:.3}", answer.confidence);
                }
                Err(e) => println!("❌ Error: {}", e),
            }
        }
        
        Commands::Stats => {
            let stats = app.get_stats().await?;
            println!("📊 Database Statistics:");
            println!("   Documents: {}", stats.total_documents);
            println!("   Chunks: {}", stats.total_chunks);
            println!("   Size: {:.2} MB", stats.db_size_mb);
        }
        
        Commands::Clear => {
            print!("🗑️  Are you sure you want to clear all data? (y/N): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                app.clear_database().await?;
                println!("✅ Database cleared successfully");
            } else {
                println!("❌ Operation cancelled");
            }
        }
        
        Commands::RecreateSchema => {
            println!("🔄 Recreating database schema...");
            app.recreate_schema().await?;
            println!("✅ Database schema recreated successfully");
        }
        
        Commands::Interactive => {
            cli::interactive::run_interactive(&mut app).await?;
        }
    }
    
    Ok(())
} 