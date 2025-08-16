use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;
use crate::core::app::TldrApp;
use crate::search::Indexer;

mod core;
mod db;
mod embeddings;
mod search;
mod cli;
mod ui;
mod vector_search;
mod pinecone;

#[derive(Parser)]
#[command(name = "tldr")]
#[command(about = "Too Long; Didn't Read - Semantic Search Made Simple")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a directory of files
    Index {
        /// Directory path to index
        #[arg(value_name = "DIRECTORY")]
        directory: String,
        
        /// File patterns to include (e.g., "*.txt,*.md,*.py")
        #[arg(short, long, value_name = "PATTERNS")]
        patterns: Option<String>,
    },
    
    /// Search for content
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Similarity threshold (0.0 to 1.0)
        #[arg(short, long, default_value = "0.7")]
        threshold: f32,
    },
    
    /// Ask a question using RAG
    Ask {
        /// Question to ask
        #[arg(value_name = "QUESTION")]
        question: String,
        
        /// Number of context chunks to use
        #[arg(short, long, default_value = "5")]
        context: usize,
    },
    
    /// Show database statistics
    Stats,
    
    /// Clear all indexed data
    Clear,
    
    /// Start interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize the app
    let mut app = TldrApp::new()?;
    
    match cli.command {
        Commands::Index { directory, patterns } => {
            let indexer = Indexer::new();
            indexer.index_directory(&directory, patterns.as_deref(), &mut app).await?;
        }
        
        Commands::Search { query, limit, threshold } => {
            let results = app.search(&query, limit, threshold).await?;
            display_search_results(&results);
        }
        
        Commands::Ask { question, context } => {
            let answer = app.ask_question(&question, context).await?;
            display_rag_answer(&answer);
        }
        
        Commands::Stats => {
            let stats = app.get_stats().await?;
            display_stats(&stats);
        }
        
        Commands::Clear => {
            app.clear_database().await?;
            println!("{}", "✅ Database cleared successfully!".green());
        }
        
        Commands::Interactive => {
            cli::interactive::run_interactive(&mut app).await?;
        }
    }
    
    Ok(())
}

fn display_search_results(results: &[crate::core::types::SearchResult]) {
    if results.is_empty() {
        println!("{}", "❌ No results found".red());
        return;
    }
    
    println!("\n🔍 Search Results ({} found):\n", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("{}. 📄 {} (Similarity: {:.3})", 
            i + 1, 
            result.document_path.blue(), 
            result.similarity
        );
        println!("   📝 {}", result.chunk_text.chars().take(100).collect::<String>());
        if result.chunk_text.len() > 100 {
            println!("   ...");
        }
        println!();
    }
}

fn display_rag_answer(answer: &crate::core::types::RAGAnswer) {
    println!("\n❓ Question: {}", answer.question.yellow());
    println!("💭 Answer:\n{}", answer.answer);
    
    if !answer.sources.is_empty() {
        println!("\n📚 Sources:");
        for source in &answer.sources {
            println!("   • {}", source.document_path.blue());
        }
    }
}

fn display_stats(stats: &crate::core::types::DatabaseStats) {
    println!("\n📊 Database Statistics:");
    println!("   📄 Documents: {}", stats.document_count);
    println!("   📝 Chunks: {}", stats.chunk_count);
    println!("   💾 Database size: {:.2} MB", stats.database_size_mb);
} 