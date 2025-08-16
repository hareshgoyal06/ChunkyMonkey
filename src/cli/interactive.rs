use crate::core::app::TldrApp;
use anyhow::Result;
use colored::*;
use console::Term;
use std::io::{self, Write};
use std::path::PathBuf;

pub async fn run_interactive(app: &mut TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    show_welcome_screen()?;
    
    // Check if database has data
    let mut stats = app.get_stats().await?;
    if stats.total_documents == 0 {
        show_first_time_setup(app).await?;
    }
    
    loop {
        show_main_menu(&stats).await?;
        
        let choice = get_user_choice()?;
        
        match choice.as_str() {
            "1" => {
                if let Err(e) = handle_index_directory(app).await {
                    show_error("Failed to index directory", &e)?;
                }
            }
            "2" => {
                if let Err(e) = handle_search_flow(app).await {
                    show_error("Search failed", &e)?;
                }
            }
            "3" => {
                if let Err(e) = handle_ask_flow(app).await {
                    show_error("Question answering failed", &e)?;
                }
            }
            "4" => {
                if let Err(e) = handle_show_stats(app).await {
                    show_error("Failed to get statistics", &e)?;
                }
            }
            "5" => {
                if let Err(e) = handle_clear_database(app).await {
                    show_error("Failed to clear database", &e)?;
                }
            }
            "6" => {
                if let Err(e) = handle_settings(app).await {
                    show_error("Settings failed", &e)?;
                }
            }
            "7" => {
                show_exit_message()?;
                break;
            }
            "q" | "quit" | "exit" => {
                show_exit_message()?;
                break;
            }
            _ => {
                show_invalid_choice()?;
            }
        }
        
        // Refresh stats for next iteration
        stats = app.get_stats().await?;
    }
    
    Ok(())
}

fn show_welcome_screen() -> Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                                                              ║".blue());
    println!("{}", "║  ████████╗██╗     ██████╗ ██████╗                          ║".blue().bold());
    println!("{}", "║  ╚══██╔══╝██║     ██╔══██╗██╔══██╗                         ║".blue().bold());
    println!("{}", "║     ██║   ██║     ██║  ██║██║  ██║                         ║".blue().bold());
    println!("{}", "║     ██║   ██║     ██║  ██║██║  ██║                         ║".blue().bold());
    println!("{}", "║     ██║   ███████╗██████╔╝██████╔╝                         ║".blue().bold());
    println!("{}", "║     ╚═╝   ╚══════╝╚═════╝ ╚═════╝                          ║".blue().bold());
    println!("{}", "║                                                              ║".blue());
    println!("{}", "║  Too Long; Didn't Read - Semantic Search Made Simple        ║".cyan());
    println!("{}", "║  Blazing-fast search through any directory                  ║".cyan());
    println!("{}", "║  Powered by vector embeddings and AI                        ║".cyan());
    println!("{}", "║                                                              ║".blue());
    println!("{}", "║  Index • Search • Ask • Explore                             ║".white());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".blue());
    println!();
    Ok(())
}

async fn show_first_time_setup(app: &mut TldrApp) -> Result<()> {
    println!("{}", "🎉 Welcome to TLDR!".bold().green());
    println!("{}", "Let's get you started by indexing your first directory.".cyan());
    println!();
    
    if let Err(e) = handle_index_directory(app).await {
        println!("{} Setup failed: {}", "❌".red(), e);
        println!("{} You can always index a directory later from the main menu.", "💡".yellow());
        wait_for_enter()?;
    }
    
    Ok(())
}

async fn show_main_menu(stats: &crate::core::types::DatabaseStats) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                        📋 Main Menu                           ║".blue().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".blue());
    println!();
    
    // Show current status
    if stats.total_documents > 0 {
        println!("{}", "📊 Current Status:".bold().green());
        println!("   📄 Documents: {}", stats.total_documents.to_string().yellow());
        println!("   📝 Chunks: {}", stats.total_chunks.to_string().yellow());
        println!("   💾 Database: {:.2} MB", stats.db_size_mb.to_string().yellow());
        println!();
    } else {
        println!("{}", "📊 Current Status:".bold().yellow());
        println!("   ❌ No documents indexed yet");
        println!();
    }
    
    println!("{}", "🚀 Actions:".bold());
    println!("   1. 📁 Index Directory     - Add files to search");
    println!("   2. 🔍 Search Content      - Find relevant content");
    println!("   3. ❓ Ask Questions       - Get AI-powered answers");
    println!("   4. 📊 View Statistics     - See database info");
    println!("   5. 🧹 Clear Database      - Remove all data");
    println!("   6. ⚙️  Settings           - Configure TLDR");
    println!("   7. ❌ Exit                - Close TLDR");
    println!();
    println!("{}", "💡 Tip: Type 'q', 'quit', or 'exit' to leave".dimmed());
    println!("{}", "─".repeat(60));
    Ok(())
}

fn get_user_choice() -> Result<String> {
    print!("{} ", "Enter your choice:".bold().cyan());
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    Ok(choice.trim().to_lowercase())
}

async fn handle_index_directory(app: &mut TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".green());
    println!("{}", "║                    📁 Index Directory                         ║".green().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".green());
    println!();
    
    // Get directory path
    let path = get_directory_path()?;
    if path.is_none() {
        return Ok(());
    }
    let path = path.unwrap();
    
    // Get file patterns
    let patterns = get_file_patterns()?;
    
    // Confirm indexing
    if !confirm_indexing(&path, &patterns)? {
        println!("{} Indexing cancelled", "❌".yellow());
        wait_for_enter()?;
        return Ok(());
    }
    
    // Perform indexing
    println!("\n{}", "⏳ Indexing directory...".bold().yellow());
    println!("{}", "This may take a moment depending on the number of files.".dimmed());
    println!();
    
    use crate::search::Indexer;
    let patterns_vec: Vec<String> = patterns.split(',').map(|s| s.trim().to_string()).collect();
    let indexer = Indexer::new(patterns_vec);
    
    match indexer.index_directory(&path, app).await {
        Ok(_) => {
            println!("{}", "✅ Indexing completed successfully!".bold().green());
            println!("{}", "You can now search and ask questions about your content.".cyan());
        }
        Err(e) => {
            println!("{} Indexing failed: {}", "❌".red(), e);
        }
    }
    
    wait_for_enter()?;
    Ok(())
}

fn get_directory_path() -> Result<Option<PathBuf>> {
    println!("{}", "📂 Directory Selection:".bold());
    println!("   Enter the path to the directory you want to index.");
    println!("   Examples: ./src, /home/user/project, . (current directory)");
    println!();
    
    loop {
        print!("{} ", "Directory path:".bold().cyan());
        io::stdout().flush()?;
        
        let mut path = String::new();
        io::stdin().read_line(&mut path)?;
        let path = path.trim();
        
        if path.is_empty() {
            println!("{} Please enter a valid path", "❌".red());
            continue;
        }
        
        if path == "back" || path == "b" {
            return Ok(None);
        }
        
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            println!("{} Directory does not exist: {}", "❌".red(), path);
            println!("   Type 'back' to return to main menu");
            continue;
        }
        
        if !path_buf.is_dir() {
            println!("{} Path is not a directory: {}", "❌".red(), path);
            continue;
        }
        
        return Ok(Some(path_buf));
    }
}

fn get_file_patterns() -> Result<String> {
    println!("\n{}", "📄 File Patterns:".bold());
    println!("   Enter file patterns to include (comma-separated)");
    println!("   Examples: *.py,*.md,*.txt or *.rs,*.toml,*.md");
    println!();
    
    print!("{} ", "File patterns (press Enter for default):".bold().cyan());
    io::stdout().flush()?;
    
    let mut patterns = String::new();
    io::stdin().read_line(&mut patterns)?;
    let patterns = patterns.trim();
    
    let default_patterns = "*.rs,*.md,*.txt,*.py,*.js,*.ts,*.json,*.yaml,*.yml,*.toml";
    
    if patterns.is_empty() {
        println!("{} Using default patterns: {}", "💡".yellow(), default_patterns);
        Ok(default_patterns.to_string())
    } else {
        Ok(patterns.to_string())
    }
}

fn confirm_indexing(path: &PathBuf, patterns: &str) -> Result<bool> {
    println!("\n{}", "📋 Indexing Summary:".bold());
    println!("   Directory: {}", path.display().to_string().yellow());
    println!("   Patterns: {}", patterns.yellow());
    println!();
    
    print!("{} ", "Proceed with indexing? (y/N):".bold().cyan());
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    
    Ok(confirm == "y" || confirm == "yes")
}

async fn handle_search_flow(app: &TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                    🔍 Search Content                          ║".blue().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".blue());
    println!();
    
    loop {
        println!("{}", "💭 What would you like to search for?".bold());
        println!("   Examples: 'authentication function', 'database connection', 'API endpoints'");
        println!("   Type 'back' to return to main menu");
        println!();
        
        print!("{} ", "Search query:".bold().cyan());
        io::stdout().flush()?;
        
        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        let query = query.trim();
        
        if query.is_empty() {
            println!("{} Please enter a search query", "❌".red());
            continue;
        }
        
        if query == "back" || query == "b" {
            break;
        }
        
        // Get search parameters
        let limit = get_search_limit()?;
        let _threshold = get_search_threshold()?;
        
        // Perform search
        println!("\n{}", "🔍 Searching...".bold().yellow());
        
        match app.search_similar(query, limit).await {
            Ok(results) => {
                display_search_results(query, &results)?;
            }
            Err(e) => {
                println!("{} Search failed: {}", "❌".red(), e);
                wait_for_enter()?;
            }
        }
        
        // Ask if user wants to search again
        print!("\n{} ", "Search again? (y/N):".bold().cyan());
        io::stdout().flush()?;
        
        let mut again = String::new();
        io::stdin().read_line(&mut again)?;
        let again = again.trim().to_lowercase();
        
        if again != "y" && again != "yes" {
            break;
        }
        
        println!();
    }
    
    Ok(())
}

fn get_search_limit() -> Result<usize> {
    print!("{} ", "Number of results (default: 5):".bold().cyan());
    io::stdout().flush()?;
    
    let mut limit_str = String::new();
    io::stdin().read_line(&mut limit_str)?;
    let limit = limit_str.trim().parse::<usize>().unwrap_or(5);
    
    Ok(limit)
}

fn get_search_threshold() -> Result<f32> {
    print!("{} ", "Similarity threshold 0.0-1.0 (default: 0.3):".bold().cyan());
    io::stdout().flush()?;
    
    let mut threshold_str = String::new();
    io::stdin().read_line(&mut threshold_str)?;
    let threshold = threshold_str.trim().parse::<f32>().unwrap_or(0.3);
    
    Ok(threshold)
}

fn display_search_results(query: &str, results: &[crate::core::types::SearchResult]) -> Result<()> {
    if results.is_empty() {
        println!("{}", "❌ No results found".bold().yellow());
        println!("   Try adjusting your search terms or similarity threshold.");
        return Ok(());
    }
    
    println!("\n{}", "📋 Search Results:".bold().green());
    println!("{} {}", "Query:".bold(), query.cyan());
    println!("{}", "─".repeat(60));
    
    for (i, result) in results.iter().enumerate() {
        println!("{}", format!("{}. 📄 {}", i + 1, result.file_path).bold());
        println!("   {} Score: {:.3}", "🎯".yellow(), result.score);
        
        // Show first line of content
        let first_line = result.text.lines().next().unwrap_or("").trim();
        if !first_line.is_empty() {
            println!("   {} {}", "📝".blue(), first_line);
        }
        
        // Show file size if available
        if let Ok(metadata) = std::fs::metadata(&result.file_path) {
            let size_kb = metadata.len() as f64 / 1024.0;
            println!("   {} Size: {:.1} KB", "💾".dimmed(), size_kb);
        }
        
        println!();
    }
    
    // Offer to view full content
    if results.len() == 1 {
        print!("{} ", "View full content? (y/N):".bold().cyan());
        io::stdout().flush()?;
        
        let mut view = String::new();
        io::stdin().read_line(&mut view)?;
        let view = view.trim().to_lowercase();
        
        if view == "y" || view == "yes" {
            println!("\n{}", "📄 Full Content:".bold());
            println!("{}", "─".repeat(60));
            println!("{}", results[0].text);
            println!("{}", "─".repeat(60));
        }
    }
    
    Ok(())
}

async fn handle_ask_flow(app: &TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".purple());
    println!("{}", "║                    ❓ Ask Questions (RAG)                     ║".purple().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".purple());
    println!();
    
    let mut last_question: String = String::new();
    let mut last_search_results: Option<Vec<crate::core::types::SearchResult>> = None;
    
    loop {
        println!("{}", "🤖 Ask me anything about your indexed content!".bold());
        println!("   Examples: 'How does authentication work?', 'What are the main features?'");
        println!("   Type 'back' to return to main menu");
        println!("   Type 'analyze' to analyze search results");
        println!("   Type 'help' for troubleshooting tips");
        println!();
        
        print!("{} ", "Your question:".bold().cyan());
        io::stdout().flush()?;
        
        let mut question = String::new();
        io::stdin().read_line(&mut question)?;
        let question = question.trim();
        
        if question.is_empty() {
            println!("{} Please enter a question", "❌".red());
            continue;
        }
        
        if question == "back" || question == "b" {
            break;
        }
        
        if question == "help" {
            println!("\n{}", "🔧 Troubleshooting Tips:".bold().yellow());
            println!("   • If you get database errors, try: cargo run -- recreate-schema");
            println!("   • If search quality is poor, try: cargo run -- clear");
            println!("   • Make sure you have indexed some content first");
            println!("   • Check that your OpenAI API key is set");
            println!("   • For schema issues, the system will auto-validate");
            continue;
        }
        
        if question == "analyze" {
            // Show analysis of previous results if available
            if let Some(last_results) = &last_search_results {
                println!("\n{}", "🔍 Analyzing previous search results...".bold().yellow());
                match app.analyze_search_results(&last_question, last_results).await {
                    Ok(analysis) => {
                        println!("{}", analysis);
                    }
                    Err(e) => {
                        println!("{} Failed to analyze results: {}", "❌".red(), e);
                    }
                }
            } else {
                println!("{} No previous search results to analyze", "⚠️".yellow());
            }
            continue;
        }
        
        // Store question for analysis
        last_question = question.to_string();
        
        // Get context chunks
        let chunks = get_context_chunks()?;
        
        // Generate answer
        println!("\n{}", "🤖 Generating answer...".bold().yellow());
        println!("{}", "This may take a moment.".dimmed());
        
        match app.ask_question(question, chunks).await {
            Ok(answer) => {
                println!("\n{}", "🤖 Answer:".bold().green());
                println!("{}", answer.text);
                
                // Store results for analysis
                last_search_results = Some(answer.sources.clone());
                
                if !answer.sources.is_empty() {
                    println!("\n{}", "📚 Sources:".bold().blue());
                    for (i, source) in answer.sources.iter().enumerate() {
                        println!("   {}: {} (similarity: {:.3})", 
                            i + 1, 
                            source.file_path, 
                            source.score
                        );
                    }
                }
                
                println!("\n{} Confidence: {:.1}%", "🎯".blue(), answer.confidence * 100.0);
            }
            Err(e) => {
                println!("{} Failed to generate answer: {}", "❌".red(), e);
            }
        }
        
        // Ask if user wants to ask another question
        print!("\n{} ", "Ask another question? (y/N):".bold().cyan());
        io::stdout().flush()?;
        
        let mut again = String::new();
        io::stdin().read_line(&mut again)?;
        let again = again.trim().to_lowercase();
        
        if again != "y" && again != "yes" {
            break;
        }
        
        println!();
    }
    
    Ok(())
}

fn get_context_chunks() -> Result<usize> {
    print!("{} ", "Number of context chunks (default: 3):".bold().cyan());
    io::stdout().flush()?;
    
    let mut chunks_str = String::new();
    io::stdin().read_line(&mut chunks_str)?;
    let chunks = chunks_str.trim().parse::<usize>().unwrap_or(3);
    
    Ok(chunks)
}

async fn handle_show_stats(app: &TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                    📊 Database Statistics                     ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    println!();
    
    let stats = app.get_stats().await?;
    
    println!("{}", "📈 Overview:".bold());
    println!("   📄 Documents indexed: {}", stats.total_documents.to_string().yellow());
    println!("   📝 Text chunks created: {}", stats.total_chunks.to_string().yellow());
    println!("   🧠 Vector embeddings: {}", stats.total_embeddings.to_string().yellow());
    println!("   💾 Database size: {:.2} MB", stats.db_size_mb.to_string().yellow());
    println!();
    
    if stats.total_documents > 0 {
        println!("{}", "📊 Averages:".bold());
        let chunks_per_doc = if stats.total_documents > 0 { stats.total_chunks as f64 / stats.total_documents as f64 } else { 0.0 };
        println!("   📝 Chunks per document: {:.1}", chunks_per_doc.to_string().yellow());
        let size_per_doc = if stats.total_documents > 0 { stats.db_size_mb / stats.total_documents as f64 } else { 0.0 };
        println!("   💾 Size per document: {:.2} MB", size_per_doc.to_string().yellow());
    }
    
    println!();
    wait_for_enter()?;
    Ok(())
}

async fn handle_clear_database(app: &mut TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".red());
    println!("{}", "║                    🧹 Clear Database                          ║".red().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".red());
    println!();
    
    println!("{}", "⚠️  Warning: This will permanently delete all indexed data!".bold().red());
    println!("   This action cannot be undone.");
    println!();
    
    let stats = app.get_stats().await?;
    if stats.total_documents > 0 {
        println!("{}", "📊 Data to be deleted:".bold());
        println!("   📄 Documents: {}", stats.total_documents.to_string().yellow());
        println!("   📝 Chunks: {}", stats.total_chunks.to_string().yellow());
        println!("   💾 Database size: {:.2} MB", stats.db_size_mb.to_string().yellow());
        println!();
    }
    
    print!("{} ", "Are you absolutely sure? Type 'DELETE' to confirm:".bold().red());
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim();
    
    if confirm == "DELETE" {
        println!("\n{}", "🧹 Clearing database...".bold().yellow());
        app.clear_database().await?;
        println!("{}", "✅ Database cleared successfully!".bold().green());
    } else {
        println!("{}", "❌ Operation cancelled".bold().yellow());
    }
    
    wait_for_enter()?;
    Ok(())
}

async fn handle_settings(_app: &mut TldrApp) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".magenta());
    println!("{}", "║                        ⚙️  Settings                           ║".magenta().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".magenta());
    println!();
    
    println!("{}", "🔧 Configuration Options:".bold());
    println!("   1. 📁 Default file patterns");
    println!("   2. 🔍 Default search settings");
    println!("   3. ❓ Default RAG settings");
    println!("   4. 📊 Database location");
    println!("   5. 🔙 Back to main menu");
    println!();
    
    print!("{} ", "Choose an option:".bold().cyan());
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();
    
    match choice {
        "1" => {
            println!("{}", "📁 Default file patterns: *.rs,*.md,*.txt,*.py,*.js,*.ts,*.json,*.yaml,*.yml,*.toml".yellow());
            println!("   This feature is coming soon!");
        }
        "2" => {
            println!("{}", "🔍 Default search settings:".yellow());
            println!("   Results limit: 5");
            println!("   Similarity threshold: 0.3");
            println!("   This feature is coming soon!");
        }
        "3" => {
            println!("{}", "❓ Default RAG settings:".yellow());
            println!("   Context chunks: 3");
            println!("   This feature is coming soon!");
        }
        "4" => {
            println!("{}", "📊 Database location: tldr.db".yellow());
            println!("   This feature is coming soon!");
        }
        "5" | "back" | "b" => {
            return Ok(());
        }
        _ => {
            println!("{} Invalid option", "❌".red());
        }
    }
    
    wait_for_enter()?;
    Ok(())
}

fn show_error(title: &str, error: &anyhow::Error) -> Result<()> {
    println!("{} {}: {}", "❌".red(), title.bold().red(), error);
    wait_for_enter()?;
    Ok(())
}

fn show_invalid_choice() -> Result<()> {
    println!("{} Invalid choice. Please try again.", "⚠️".yellow());
    wait_for_enter()?;
    Ok(())
}

fn show_exit_message() -> Result<()> {
    println!("\n{}", "👋 Thanks for using TLDR!".bold().green());
    println!("{}", "Happy searching! 🚀".cyan());
    Ok(())
}

fn wait_for_enter() -> Result<()> {
    print!("{} ", "Press Enter to continue...".dimmed());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(())
} 