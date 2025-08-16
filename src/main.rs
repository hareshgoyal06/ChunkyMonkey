use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;
use crate::core::app::ChunkyMonkeyApp;
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
#[command(name = "chunkymonkey")]
#[command(about = "🐒 ChunkyMonkey - Going Bananas for Chunks! 🍌")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start ChunkyMonkey in interactive mode
    Start,
    
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
    
    /// Show RAG pipeline statistics
    RagStats,
    
    /// Clear all indexed data
    Clear,
    
    /// Start interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize the app
    let mut app = ChunkyMonkeyApp::new()?;
    
    match cli.command {
        Commands::Start => {
            cli::interactive::run_interactive(&mut app).await?;
        }
        
        Commands::Index { directory, patterns } => {
            let indexer = Indexer::new();
            // For CLI indexing, we'll create a default project or use None
            let project_id = None; // CLI users can manage projects through interactive mode
            indexer.index_directory(&directory, patterns.as_deref(), &mut app, project_id).await?;
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
        
        Commands::RagStats => {
            let rag_stats = app.get_rag_stats().await?;
            display_rag_stats(&rag_stats);
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
        
        // Show a cleaner preview of the content
        let preview = result.chunk_text.chars().take(60).collect::<String>();
        if !preview.is_empty() {
            println!("   {}", preview);
        }
        
        if result.chunk_text.len() > 60 {
            println!("   ...");
        }
        println!();
    }
}

fn display_rag_answer(answer: &crate::core::types::RAGAnswer) {
    println!("\n❓ Question: {}", answer.question.yellow());
    println!("\n💭 Answer:");
    println!("{}", "─".repeat(50));
    println!("{}", answer.answer);
    println!("{}", "─".repeat(50));
    
    if !answer.sources.is_empty() {
        println!("\n📚 Sources:");
        for source in &answer.sources {
            println!("   • {}", source.document_path.blue());
        }
    }
}

fn display_stats(stats: &crate::core::types::DatabaseStats) {
    println!("\n📊 Database Statistics:");
    println!("   🗂️  Projects: {}", stats.project_count);
    println!("   📄 Documents: {}", stats.document_count);
    println!("   📝 Chunks: {}", stats.chunk_count);
    println!("   💾 Database size: {:.2} MB", stats.database_size_mb);
}

fn display_rag_stats(stats: &crate::core::types::RAGPipelineStats) {
    println!("\n🤖 RAG Pipeline Statistics:");
    println!("   ⚙️  Advanced RAG: {}", if stats.config_enabled { "✅ Enabled".green() } else { "❌ Disabled".red() });
    println!("   🔍 Quality Assessment: {}", if stats.quality_assessment_enabled { "✅ Enabled".green() } else { "❌ Disabled".red() });
    println!("   ✅ Answer Validation: {}", if stats.answer_validation_enabled { "✅ Enabled".green() } else { "❌ Disabled".red() });
    println!("   🚀 Semantic Expansion: {}", if stats.semantic_expansion_enabled { "✅ Enabled".green() } else { "❌ Disabled".red() });
    println!("   🛡️  Fallback Strategies: {}", if stats.fallback_strategies_enabled { "✅ Enabled".green() } else { "❌ Disabled".red() });
    println!("\n📊 System Status:");
    println!("   🗄️  Local Vectors: {}", stats.local_vector_count);
    println!("   🌲 Pinecone: {}", if stats.pinecone_available { "✅ Available".green() } else { "❌ Unavailable".red() });
    println!("   🧠 Ollama: {}", if stats.ollama_available { "✅ Available".green() } else { "❌ Unavailable".red() });
    println!("   📐 Embedding Dimension: {}", stats.embedding_dimension);
} 