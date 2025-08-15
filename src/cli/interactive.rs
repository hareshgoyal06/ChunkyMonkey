use crate::core::app::TldrApp;
use anyhow::Result;
use colored::*;
use console::Term;
use std::io::{self, Write};

pub async fn run_interactive(app: &mut TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    show_welcome_screen()?;
    
    loop {
        show_main_menu()?;
        
        let choice = get_user_choice()?;
        
        match choice.as_str() {
            "1" => {
                // Index directory
                if let Err(e) = handle_index_directory(app).await {
                    println!("{} Error: {}", "❌".red(), e);
                    wait_for_enter()?;
                }
            }
            "2" => {
                // Search
                if let Err(e) = handle_search(app).await {
                    println!("{} Error: {}", "❌".red(), e);
                    wait_for_enter()?;
                }
            }
            "3" => {
                // Ask question
                if let Err(e) = handle_ask_question(app).await {
                    println!("{} Error: {}", "❌".red(), e);
                    wait_for_enter()?;
                }
            }
            "4" => {
                // Show stats
                if let Err(e) = handle_show_stats(app).await {
                    println!("{} Error: {}", "❌".red(), e);
                    wait_for_enter()?;
                }
            }
            "5" => {
                // Clear database
                if let Err(e) = handle_clear_database(app).await {
                    println!("{} Error: {}", "❌".red(), e);
                    wait_for_enter()?;
                }
            }
            "6" => {
                // Exit
                println!("{}", "👋 Goodbye!".green());
                break;
            }
            _ => {
                println!("{} Invalid choice. Please try again.", "⚠️".yellow());
                wait_for_enter()?;
            }
        }
    }
    
    Ok(())
}

fn show_welcome_screen() -> Result<()> {
    println!("{}", "🔍 TLDR - Too Long; Didn't Read".bold().blue());
    println!("{}", "Blazing-fast semantic search through any directory".cyan());
    println!("{}", "Powered by vector embeddings and AI".dimmed());
    println!();
    Ok(())
}

fn show_main_menu() -> Result<()> {
    println!("{}", "📋 Main Menu".bold());
    println!("{}", "=".repeat(40));
    println!("1. 📁 Index Directory");
    println!("2. 🔍 Search Content");
    println!("3. ❓ Ask Questions (RAG)");
    println!("4. 📊 View Statistics");
    println!("5. 🧹 Clear Database");
    println!("6. ❌ Exit");
    println!("{}", "=".repeat(40));
    Ok(())
}

fn get_user_choice() -> Result<String> {
    print!("{} ", "Enter your choice (1-6):".bold());
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    Ok(choice.trim().to_string())
}

async fn handle_index_directory(app: &mut TldrApp) -> Result<()> {
    println!("\n{}", "📁 Index Directory".bold().green());
    println!("{}", "-".repeat(30));
    
    print!("Enter directory path: ");
    io::stdout().flush()?;
    
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
    let path = path.trim();
    
    if path.is_empty() {
        println!("{} No path provided", "❌".red());
        return Ok(());
    }
    
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        println!("{} Directory does not exist", "❌".red());
        return Ok(());
    }
    
    print!("File patterns (comma-separated, default: *.rs,*.md,*.txt): ");
    io::stdout().flush()?;
    
    let mut patterns = String::new();
    io::stdin().read_line(&mut patterns)?;
    let patterns = patterns.trim();
    
    let patterns = if patterns.is_empty() {
        "*.rs,*.md,*.txt,*.py,*.js,*.ts,*.json,*.yaml,*.yml".to_string()
    } else {
        patterns.to_string()
    };
    
    println!("\n{} Indexing directory...", "⏳".yellow());
    
    use crate::search::Indexer;
    let indexer = Indexer::new(800, 150);
    indexer.index_directory(app, &path, &patterns).await?;
    
    println!("{} Indexing completed!", "✅".green());
    wait_for_enter()?;
    
    Ok(())
}

async fn handle_search(app: &TldrApp) -> Result<()> {
    println!("\n{}", "🔍 Search Content".bold().blue());
    println!("{}", "-".repeat(30));
    
    print!("Enter search query: ");
    io::stdout().flush()?;
    
    let mut query = String::new();
    io::stdin().read_line(&mut query)?;
    let query = query.trim();
    
    if query.is_empty() {
        println!("{} No query provided", "❌".red());
        return Ok(());
    }
    
    print!("Number of results (default: 5): ");
    io::stdout().flush()?;
    
    let mut limit_str = String::new();
    io::stdin().read_line(&mut limit_str)?;
    let limit = limit_str.trim().parse::<usize>().unwrap_or(5);
    
    println!("\n{} Searching...", "⏳".yellow());
    
    let results = app.search(query, limit, 0.3).await?;
    
    if results.is_empty() {
        println!("{} No results found", "❌".yellow());
    } else {
        println!("\n{}", "📋 Search Results:".bold());
        println!("{}", "-".repeat(50));
        
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
    
    wait_for_enter()?;
    Ok(())
}

async fn handle_ask_question(app: &TldrApp) -> Result<()> {
    println!("\n{}", "❓ Ask Questions (RAG)".bold().purple());
    println!("{}", "-".repeat(30));
    
    print!("Enter your question: ");
    io::stdout().flush()?;
    
    let mut question = String::new();
    io::stdin().read_line(&mut question)?;
    let question = question.trim();
    
    if question.is_empty() {
        println!("{} No question provided", "❌".red());
        return Ok(());
    }
    
    print!("Number of context chunks (default: 3): ");
    io::stdout().flush()?;
    
    let mut chunks_str = String::new();
    io::stdin().read_line(&mut chunks_str)?;
    let chunks = chunks_str.trim().parse::<usize>().unwrap_or(3);
    
    println!("\n{} Generating answer...", "⏳".yellow());
    
    let answer = app.ask_question(question, chunks).await?;
    
    println!("\n{}", "🤖 Answer:".bold());
    println!("{}", "-".repeat(30));
    println!("{}", answer.text);
    
    if !answer.sources.is_empty() {
        println!("\n{}", "📚 Sources:".bold());
        for source in &answer.sources {
            println!("• {} (similarity: {:.3})", source.file_path.display(), source.similarity);
        }
    }
    
    wait_for_enter()?;
    Ok(())
}

async fn handle_show_stats(app: &TldrApp) -> Result<()> {
    println!("\n{}", "📊 Database Statistics".bold().cyan());
    println!("{}", "-".repeat(30));
    
    let stats = app.get_stats().await?;
    
    println!("📄 Documents: {}", stats.documents);
    println!("📝 Chunks: {}", stats.chunks);
    println!("🧠 Embeddings: {}", stats.embeddings);
    println!("💾 Database size: {:.2} MB", stats.db_size_mb);
    
    wait_for_enter()?;
    Ok(())
}

async fn handle_clear_database(app: &mut TldrApp) -> Result<()> {
    println!("\n{}", "🧹 Clear Database".bold().red());
    println!("{}", "-".repeat(30));
    
    print!("Are you sure you want to clear all indexed data? (y/N): ");
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    
    if confirm == "y" || confirm == "yes" {
        app.clear_database().await?;
        println!("{} Database cleared", "✅".green());
    } else {
        println!("{} Operation cancelled", "❌".yellow());
    }
    
    wait_for_enter()?;
    Ok(())
}

fn wait_for_enter() -> Result<()> {
    print!("Press Enter to continue...");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(())
} 