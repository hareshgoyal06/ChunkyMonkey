use anyhow::Result;
use colored::*;
use console::Term;
use crate::core::app::TldrApp;
use crate::core::types::*;

pub async fn run_interactive(app: &mut TldrApp) -> Result<()> {
    let _term = Term::stdout();
    
    // Show welcome screen
    show_welcome_screen();
    
    // Check if this is first time setup
    let mut stats = app.get_stats().await?;
    if stats.document_count == 0 {
        show_first_time_setup();
        handle_index_directory(&mut *app).await?;
    }
    
    // Main interactive loop
    loop {
        show_main_menu(&stats).await?;
        
        match get_user_choice()?.as_str() {
            "1" => {
                handle_index_directory(&mut *app).await?;
                // Refresh stats
                let new_stats = app.get_stats().await?;
                stats = new_stats;
            }
            "2" => {
                handle_search_flow(app).await?;
            }
            "3" => {
                handle_ask_flow(app).await?;
            }
            "4" => {
                handle_show_stats(app).await?;
            }
            "5" => {
                handle_clear_database(app).await?;
                stats = DatabaseStats {
                    document_count: 0,
                    chunk_count: 0,
                    database_size_mb: 0.0,
                };
            }
            "6" => {
                handle_settings();
            }
            "7" | "q" | "quit" | "exit" => {
                show_exit_message();
                break;
            }
            _ => {
                show_invalid_choice();
            }
        }
        
        wait_for_enter();
    }
    
    Ok(())
}

fn show_welcome_screen() {
    println!("\n{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                                                              ║".blue());
    println!("{}", "║  ████████╗██╗     ██████╗ ██████╗                          ║".blue());
    println!("{}", "║  ╚══██╔══╝██║     ██╔══██╗██╔══██╗                         ║".blue());
    println!("{}", "║     ██║   ██║     ██║  ██║██║  ██║                         ║".blue());
    println!("{}", "║     ██║   ██║     ██║  ██║██║  ██║                         ║".blue());
    println!("{}", "║     ██║   ███████╗██████╔╝██████╔╝                         ║".blue());
    println!("{}", "║     ╚═╝   ╚══════╝╚═════╝ ╚═════╝                          ║".blue());
    println!("{}", "║                                                              ║".blue());
    println!("{}", "║  Too Long; Didn't Read - Semantic Search Made Simple        ║".blue());
    println!("{}", "║  Blazing-fast search through any directory using HNSW!      ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝\n".blue());
}

fn show_first_time_setup() {
    println!("🎉 Welcome to TLDR! Let's get you started.");
    println!("First, you'll need to index a directory of files to search through.\n");
}

async fn show_main_menu(stats: &DatabaseStats) -> Result<()> {
    println!("\n{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                         📋 Main Menu                           ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".blue());
    
    println!("📊 Current Status:");
    println!("   📄 Documents: {}", stats.document_count);
    println!("   📝 Chunks: {}", stats.chunk_count);
    println!("   💾 Database: {:.2} MB", stats.database_size_mb);
    
    println!("\n🚀 Actions:");
    println!("   1. 📁 Index Directory     - Add files to search");
    println!("   2. 🔍 Search Content      - Find relevant content");
    println!("   3. ❓ Ask Questions       - Get AI-powered answers");
    println!("   4. 📊 View Statistics     - See database info");
    println!("   5. 🧹 Clear Database      - Remove all data");
    println!("   6. ⚙️  Settings           - Configure TLDR");
    println!("   7. ❌ Exit                - Close TLDR");
    println!("💡 Tip: Type 'q', 'quit', or 'exit' to leave");
    Ok(())
}

fn get_user_choice() -> Result<String> {
    let term = Term::stdout();
    term.write_str("\nEnter your choice: ")?;
    let choice = term.read_line()?;
    Ok(choice.trim().to_lowercase())
}

async fn handle_index_directory(app: &mut TldrApp) -> Result<()> {
    println!("\n{}", "📁 Directory Indexing".cyan().bold());
    println!("{}", "─".repeat(40));
    
    let directory_path = get_directory_path()?;
    let file_patterns = get_file_patterns()?;
    
    if confirm_indexing(&directory_path, &file_patterns)? {
        println!("\n🚀 Starting indexing process...");
        
        let indexer = crate::search::Indexer::new();
        indexer.index_directory(&directory_path, Some(&file_patterns), app).await?;
        
        println!("✅ Indexing completed successfully!");
    } else {
        println!("❌ Indexing cancelled.");
    }
    
    Ok(())
}

fn get_directory_path() -> Result<String> {
    let term = Term::stdout();
    term.write_str("Enter directory path to index: ")?;
    let path = term.read_line()?;
    Ok(path.trim().to_string())
}

fn get_file_patterns() -> Result<String> {
    let term = Term::stdout();
    term.write_str("Enter file patterns (e.g., *.txt,*.md,*.py) or press Enter for all files: ")?;
    let patterns = term.read_line()?;
    let patterns = patterns.trim();
    if patterns.is_empty() {
        Ok("*".to_string())
    } else {
        Ok(patterns.to_string())
    }
}

fn confirm_indexing(directory: &str, patterns: &str) -> Result<bool> {
    let term = Term::stdout();
    term.write_str(&format!("\nReady to index:\n"))?;
    term.write_str(&format!("   Directory: {}\n", directory))?;
    term.write_str(&format!("   Patterns: {}\n", patterns))?;
    term.write_str("Proceed? (y/N): ")?;
    
    let response = term.read_line()?;
    Ok(response.trim().to_lowercase() == "y")
}

async fn handle_search_flow(app: &TldrApp) -> Result<()> {
    println!("\n{}", "🔍 Semantic Search".cyan().bold());
    println!("{}", "─".repeat(40));
    
    let term = Term::stdout();
    
    loop {
        term.write_str("Enter search query (or 'back' to return): ")?;
        let query = term.read_line()?;
        let query = query.trim();
        
        if query.to_lowercase() == "back" {
            break;
        }
        
        if query.is_empty() {
            println!("❌ Query cannot be empty");
            continue;
        }
        
        let limit = get_search_limit()?;
        let threshold = get_search_threshold()?;
        
        println!("🔍 Searching for: {}", query);
        println!("⏳ Please wait...");
        
        match app.search(query, limit, threshold).await {
            Ok(results) => {
                display_search_results(&results);
            }
            Err(e) => {
                show_error(&format!("Search failed: {}", e));
            }
        }
        
        term.write_str("\nPress Enter to search again, or type 'back' to return: ")?;
        let response = term.read_line()?;
        if response.trim().to_lowercase() == "back" {
            break;
        }
    }
    
    Ok(())
}

fn get_search_limit() -> Result<usize> {
    let term = Term::stdout();
    term.write_str("Maximum results (1-50): ")?;
    let input = term.read_line()?;
    let limit: usize = input.trim().parse().unwrap_or(10);
    Ok(limit.max(1).min(50))
}

fn get_search_threshold() -> Result<f32> {
    let term = Term::stdout();
    term.write_str("Similarity threshold (0.0-1.0, default 0.7): ")?;
    let input = term.read_line()?;
    let threshold: f32 = input.trim().parse().unwrap_or(0.7);
    Ok(threshold.max(0.0).min(1.0))
}

fn display_search_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("❌ No results found");
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

async fn handle_ask_flow(app: &TldrApp) -> Result<()> {
    println!("\n{}", "❓ RAG Question Answering".cyan().bold());
    println!("{}", "─".repeat(40));
    
    let term = Term::stdout();
    
    loop {
        term.write_str("Enter your question (or 'back' to return): ")?;
        let question = term.read_line()?;
        let question = question.trim();
        
        if question.to_lowercase() == "back" {
            break;
        }
        
        if question.is_empty() {
            println!("❌ Question cannot be empty");
            continue;
        }
        
        let context_chunks = get_context_chunks()?;
        
        println!("❓ Question: {}", question);
        println!("⏳ Generating answer...");
        
        match app.ask_question(question, context_chunks).await {
            Ok(answer) => {
                display_rag_answer(&answer);
            }
            Err(e) => {
                show_error(&format!("Failed to generate answer: {}", e));
            }
        }
        
        term.write_str("\nPress Enter to ask another question, or type 'back' to return: ")?;
        let response = term.read_line()?;
        if response.trim().to_lowercase() == "back" {
            break;
        }
    }
    
    Ok(())
}

fn get_context_chunks() -> Result<usize> {
    let term = Term::stdout();
    term.write_str("Number of context chunks (1-20, default 5): ")?;
    let input = term.read_line()?;
    let chunks: usize = input.trim().parse().unwrap_or(5);
    Ok(chunks.max(1).min(20))
}

fn display_rag_answer(answer: &RAGAnswer) {
    println!("\n❓ Question: {}", answer.question.yellow());
    println!("💭 Answer:\n{}", answer.answer);
    
    if !answer.sources.is_empty() {
        println!("\n📚 Sources:");
        for source in &answer.sources {
            println!("   • {}", source.document_path.blue());
        }
    }
}

async fn handle_show_stats(app: &TldrApp) -> Result<()> {
    println!("\n{}", "📊 Database Statistics".cyan().bold());
    println!("{}", "─".repeat(40));
    
    match app.get_stats().await {
        Ok(stats) => {
            println!("📄 Documents: {}", stats.document_count);
            println!("📝 Chunks: {}", stats.chunk_count);
            println!("💾 Database size: {:.2} MB", stats.database_size_mb);
            
            if stats.document_count > 0 {
                let avg_chunks = stats.chunk_count as f64 / stats.document_count as f64;
                println!("📊 Average chunks per document: {:.1}", avg_chunks);
            }
        }
        Err(e) => {
            show_error(&format!("Failed to get stats: {}", e));
        }
    }
    
    Ok(())
}

async fn handle_clear_database(app: &mut TldrApp) -> Result<()> {
    println!("\n{}", "🧹 Clear Database".red().bold());
    println!("{}", "─".repeat(40));
    println!("⚠️  This will permanently delete ALL indexed data!");
    
    let term = Term::stdout();
    term.write_str("Are you absolutely sure? Type 'DELETE' to confirm: ")?;
    let confirmation = term.read_line()?;
    
    if confirmation.trim() == "DELETE" {
        match app.clear_database().await {
            Ok(_) => {
                println!("✅ Database cleared successfully!");
            }
            Err(e) => {
                show_error(&format!("Failed to clear database: {}", e));
            }
        }
    } else {
        println!("❌ Operation cancelled");
    }
    
    Ok(())
}

fn handle_settings() {
    println!("\n{}", "⚙️  Settings".cyan().bold());
    println!("{}", "─".repeat(40));
    println!("Settings configuration coming soon!");
}

fn show_error(message: &str) {
    println!("❌ {}", message.red());
}

fn show_invalid_choice() {
    println!("❌ Invalid choice. Please try again.");
}

fn show_exit_message() {
    println!("\n👋 Thanks for using TLDR! Goodbye!");
}

fn wait_for_enter() {
    let term = Term::stdout();
    term.write_str("\nPress Enter to continue...").ok();
    term.read_line().ok();
} 