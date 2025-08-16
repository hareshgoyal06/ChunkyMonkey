use anyhow::Result;
use colored::*;
use console::Term;
use crate::core::app::ChunkyMonkeyApp;
use crate::core::types::*;

pub async fn run_interactive(app: &mut ChunkyMonkeyApp) -> Result<()> {
    let _term = Term::stdout();
    
    // Show welcome screen
    show_welcome_screen();
    
    // Check if this is first time setup
    let mut stats = app.get_stats().await?;
    if stats.document_count == 0 {
        show_first_time_setup();
        handle_first_time_indexing(app).await?;
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
                handle_show_rag_stats(app).await?;
            }
            "6" => {
                handle_clear_database(app).await?;
                stats = DatabaseStats {
                    document_count: 0,
                    chunk_count: 0,
                    database_size_mb: 0.0,
                };
            }
            "7" => {
                handle_settings();
            }
            "8" | "q" | "quit" | "exit" => {
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
    println!("\n{}", " ________  ___  ___  ___  ___  ________   ___  __        ___    ___      ".bright_purple());
    println!("{}", "|\\   ____\\|\\  \\|\\  \\|\\  \\|\\  \\|\\   ___  \\|\\  \\|\\  \\     |\\  \\  /  /|     ".bright_purple());
    println!("{}", "\\ \\  \\___|\\ \\  \\\\\\  \\ \\  \\\\\\  \\ \\  \\\\ \\  \\ \\  \\/  /|_   \\ \\  \\/  / /     ".bright_yellow());
    println!("{}", " \\ \\  \\    \\ \\   __  \\ \\  \\\\\\  \\ \\  \\\\ \\  \\ \\   ___  \\   \\ \\    / /      ".bright_green());
    println!("{}", "  \\ \\  \\____\\ \\  \\ \\  \\ \\  \\\\\\  \\ \\  \\\\ \\  \\ \\  \\\\ \\  \\   \\/  /  /       ".bright_yellow());
    println!("{}", "   \\ \\_______\\ \\__\\ \\__\\ \\_______\\ \\__\\\\ \\__\\ \\__\\\\ \\__\\__/  / /         ".bright_purple());
    println!("{}", "    \\|_______|\\|__|\\|__|\\|_______|\\|__| \\|__|\\|__| \\|__|\\___/ /          ".bright_yellow());
    println!("{}", " _____ ______   ________  ________   ___  __    _______\\|___|/___    ___ ".bright_purple());
    println!("{}", "|\\   _ \\  _   \\|\\   __  \\|\\   ___  \\|\\  \\|\\  \\ |\\  ___ \\     |\\  \\  /  /|".bright_yellow());
    println!("{}", "\\ \\  \\\\\\__\\ \\  \\ \\  \\|\\  \\ \\  \\\\ \\  \\ \\  \\/  /|\\ \\   __/|    \\ \\  \\/  / / ".bright_green());
    println!("{}", " \\ \\  \\\\|__| \\  \\ \\  \\\\\\  \\ \\  \\\\ \\  \\ \\   ___  \\ \\  \\_|/__   \\ \\    / / ".bright_yellow());
    println!("{}", "  \\ \\  \\    \\ \\  \\ \\  \\\\\\  \\ \\  \\\\ \\  \\ \\  \\\\ \\  \\ \\  \\_|\\ \\   \\/  /  /  ".bright_purple());
    println!("{}", "   \\ \\__\\    \\ \\__\\ \\_______\\ \\__\\\\ \\__\\ \\__\\\\ \\__\\ \\_______\\__/  / /    ".bright_yellow());
    println!("{}", "    \\|__|     \\|__|\\|_______|\\|__| \\|__|\\|__| \\|__|\\|_______|\\___/ /     ".bright_purple());
    println!("{}", "                                                            \\|___|/    ".bright_green());
    
    println!("\n{}", "                                _".bright_yellow());
    println!("{}", "                               //\\".bright_yellow());
    println!("{}", "                              V  \\".bright_yellow());
    println!("{}", "                               \\  \\_".bright_yellow());
    println!("{}", "                                \\,'.`-.".bright_yellow());
    println!("{}", "                                 |\\ `. `.".bright_yellow());
    println!("{}", "                                 ( \\  `. `-.                        _,.-:\\".bright_yellow());
    println!("{}", "                                  \\ \\   `.  `-._             __..--' ,-';/".bright_yellow());
    println!("{}", "                                   \\ `.   `-.   `-..___..---'   _.--' ,'/".bright_yellow());
    println!("{}", "                                    `. `.    `-._        __..--'    ,' /".bright_yellow());
    println!("{}", "                                      `. `-_     ``--..''       _.-' ,'".bright_yellow());
    println!("{}", "                                        `-_ `-.___        __,--'   ,'".bright_yellow());
    println!("{}", "                                           `-.__  `----\"\"\"    __.-'".bright_yellow());
    println!("{}", "                                              `--..____..--'".bright_yellow());
}

fn show_first_time_setup() {
    println!("\n🎉 {}! Let's get you started.", "Welcome to ChunkyMonkey".bright_green().bold());
    println!("First, you'll need to index some documents to search through.");
    println!("This will create embeddings and make your content searchable.\n");
}

async fn show_main_menu(stats: &DatabaseStats) -> Result<()> {
    println!("\n{}", "╔══════════════════════════════════════════════════════════════╗".white());
    println!("{}", "║                    🐒 Main Menu 🍌                           ║".white());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝\n".white());
    
    // Status section with professional white design and subtle color accents
    println!("{}", "📊 Current Status:".white().bold());
    println!("   🗂️  {}: {}", "Documents".white(), stats.document_count.to_string().bright_purple());
    println!("   💾 {}: {:.2} MB", "Database".white(), stats.database_size_mb.to_string().bright_yellow());
    println!("   🔍 {}: {}", "Chunks".white(), stats.chunk_count.to_string().bright_green());
    
    println!("\n{}", "🚀 Actions:".white().bold());
    println!("   1. 📁 {}       - Add files to search", "Index Directory".white());
    println!("   2. 🔍 {}        - Find relevant content", "Search Content".white());
    println!("   3. ❓ {}         - Get AI-powered answers", "Ask Questions".white());
    println!("   4. 📊 {}         - See database info", "View Statistics".white());
    println!("   5. 🤖 {}         - See RAG system status", "RAG Pipeline Stats".white());
    println!("   6. 🧹 {}         - Remove all data", "Clear Database".white());
    println!("   7. ⚙️  {}             - Configure ChunkyMonkey", "Settings".white());
    println!("   8. ❌ {}                  - Close ChunkyMonkey", "Exit".white());
    
    println!("\n💡 {}: Type 'q', 'quit', or 'exit' to leave", "Tip".bright_purple());
    Ok(())
}

fn get_user_choice() -> Result<String> {
    let term = Term::stdout();
    term.write_str("\n🎯 Enter your choice: ")?;
    let choice = term.read_line()?;
    Ok(choice.trim().to_lowercase())
}

async fn handle_first_time_indexing(app: &mut ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "🎯 First Time Setup".bright_green().bold());
    println!("{}", "─".repeat(50));
    
    println!("Let's start by indexing some documents to make them searchable!");
    println!("You can index any directory containing text files, code, or documentation.\n");
    
    let directory_path = get_directory_path()?;
    let file_patterns = get_file_patterns()?;
    
    if confirm_indexing(&directory_path, &file_patterns)? {
        println!("\n🚀 Starting indexing process...");
        
        let indexer = crate::search::Indexer::new();
        indexer.index_directory(&directory_path, Some(&file_patterns), app).await?;
        
        println!("✅ Indexing completed successfully!");
        println!("🎉 You're all set! You can now search through your documents and ask questions.");
    } else {
        println!("❌ Indexing cancelled. You'll need to index documents to use ChunkyMonkey.");
    }
    
    Ok(())
}

async fn handle_index_directory(app: &mut ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "📁 Directory Indexing".bright_green().bold());
    println!("{}", "─".repeat(50));
    
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
    term.write_str("📂 Enter directory path to index: ")?;
    let path = term.read_line()?;
    Ok(path.trim().to_string())
}

fn get_file_patterns() -> Result<String> {
    let term = Term::stdout();
    term.write_str("🔍 Enter file patterns (e.g., *.txt,*.md,*.py) or press Enter for all files: ")?;
    let path = term.read_line()?;
    let path = path.trim();
    if path.is_empty() {
        Ok("*".to_string())
    } else {
        Ok(path.to_string())
    }
}

fn confirm_indexing(directory: &str, patterns: &str) -> Result<bool> {
    let term = Term::stdout();
    println!("\n📋 Ready to index:");
    println!("   📂 Directory: {}", directory.bright_green());
    println!("   🔍 Patterns: {}", patterns.bright_green());
    term.write_str("\n🚀 Proceed? (y/N): ")?;
    
    let response = term.read_line()?;
    Ok(response.trim().to_lowercase() == "y")
}

async fn handle_search_flow(app: &ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "🔍 Semantic Search".bright_purple().bold());
    println!("{}", "─".repeat(50));
    
    let term = Term::stdout();
    
    loop {
        term.write_str("\n🎯 Enter search query (or 'back' to return): ")?;
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
        
        println!("\n🔍 Searching...");
        
        match app.search(query, limit, threshold).await {
            Ok(results) => {
                display_search_results(&results);
            }
            Err(e) => {
                show_error(&format!("Search failed: {}", e));
            }
        }
        
        term.write_str("\n🔄 Press Enter to search again, or type 'back' to return: ")?;
        let response = term.read_line()?;
        if response.trim().to_lowercase() == "back" {
            break;
        }
    }
    
    Ok(())
}

fn get_search_limit() -> Result<usize> {
    let term = Term::stdout();
    term.write_str("📊 Maximum results (1-50): ")?;
    let input = term.read_line()?;
    let limit: usize = input.trim().parse().unwrap_or(10);
    Ok(limit.max(1).min(50))
}

fn get_search_threshold() -> Result<f32> {
    let term = Term::stdout();
    term.write_str("🎯 Similarity threshold (0.0-1.0, default 0.7): ")?;
    let input = term.read_line()?;
    let threshold: f32 = input.trim().parse().unwrap_or(0.7);
    Ok(threshold.max(0.0).min(1.0))
}

fn display_search_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("❌ No results found");
        return;
    }
    
    println!("\n🎉 Found {} results:\n", results.len().to_string().bright_green());
    
    for (i, result) in results.iter().enumerate() {
        println!("{}. 📄 {} (Similarity: {:.3})", 
            (i + 1).to_string().bright_yellow(), 
            result.document_path.bright_green(), 
            result.similarity.to_string().bright_green()
        );
        
        // Show a cleaner preview of the content
        let preview = result.chunk_text.chars().take(80).collect::<String>();
        if !preview.is_empty() {
            println!("   {}", preview.bright_white());
        }
        
        if result.chunk_text.len() > 80 {
            println!("   {}", "...".bright_white());
        }
        println!();
    }
}

async fn handle_ask_flow(app: &ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "❓ RAG Question Answering".bright_yellow().bold());
    println!("{}", "─".repeat(50));
    
    let term = Term::stdout();
    
    loop {
        term.write_str("\n🤔 Enter your question (or 'back' to return): ")?;
        let question = term.read_line()?;
        let question = question.trim();
        
        if question.to_lowercase() == "back" {
            break;
        }
        
        if question.is_empty() {
            println!("❌ Question cannot be empty");
            continue;
        }
        
        println!("\n🧠 Processing your question...");
        
        match app.ask_question(question, None).await {
            Ok(answer) => {
                display_rag_answer(&answer);
            }
            Err(e) => {
                show_error(&format!("Question answering failed: {}", e));
            }
        }
        
        term.write_str("\n🔄 Press Enter to ask another question, or type 'back' to return: ")?;
        let response = term.read_line()?;
        if response.trim().to_lowercase() == "back" {
            break;
        }
    }
    
    Ok(())
}

fn display_rag_answer(answer: &RAGAnswer) {
    println!("\n{}", "✨ Answer Generated Successfully!".bright_green().bold());
    println!("{}", "─".repeat(50));
    
    println!("❓ Question: {}", answer.question.bright_green());
    println!("\n💡 Answer:");
    println!("{}", answer.answer.bright_white());
}

async fn handle_show_stats(app: &ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "📊 Database Statistics".bright_green().bold());
    println!("{}", "─".repeat(50));
    
    match app.get_stats().await {
        Ok(stats) => {
            println!("🗂️  Documents indexed: {}", stats.document_count.to_string().bright_green());
            println!("🔍 Total chunks: {}", stats.chunk_count.to_string().bright_green());
            println!("💾 Database size: {:.2} MB", stats.database_size_mb.to_string().bright_green());
        }
        Err(e) => {
            show_error(&format!("Failed to get statistics: {}", e));
        }
    }
    
    Ok(())
}

async fn handle_show_rag_stats(app: &ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "🤖 RAG Pipeline Statistics".bright_yellow().bold());
    println!("{}", "─".repeat(50));
    
    match app.get_rag_stats().await {
        Ok(stats) => {
            println!("🧠 Advanced RAG: {}", if stats.config_enabled { "✅ Enabled".bright_green() } else { "❌ Disabled".bright_red() });
            println!("📊 Quality Assessment: {}", if stats.quality_assessment_enabled { "✅ Enabled".bright_green() } else { "❌ Disabled".bright_red() });
            println!("✅ Answer Validation: {}", if stats.answer_validation_enabled { "✅ Enabled".bright_green() } else { "❌ Disabled".bright_red() });
            println!("🔍 Semantic Expansion: {}", if stats.semantic_expansion_enabled { "✅ Enabled".bright_green() } else { "❌ Disabled".bright_red() });
            println!("🔄 Fallback Strategies: {}", if stats.fallback_strategies_enabled { "✅ Enabled".bright_green() } else { "❌ Disabled".bright_red() });
            println!("\n📈 Vector Index:");
            println!("   🏠 Local vectors: {}", stats.local_vector_count.to_string().bright_green());
            println!("   ☁️  Pinecone: {}", if stats.pinecone_available { "✅ Available".bright_green() } else { "❌ Not configured".bright_red() });
            println!("   🤖 Ollama: {}", if stats.ollama_available { "✅ Available".bright_green() } else { "❌ Not configured".bright_red() });
            println!("   📏 Embedding dimension: {}", stats.embedding_dimension.to_string().bright_green());
        }
        Err(e) => {
            show_error(&format!("Failed to get RAG statistics: {}", e));
        }
    }
    
    Ok(())
}

async fn handle_clear_database(app: &mut ChunkyMonkeyApp) -> Result<()> {
    println!("\n{}", "🧹 Clear Database".bright_purple().bold());
    println!("{}", "─".repeat(50));
    
    println!("⚠️  This action will permanently delete ALL indexed documents and data!");
    println!("This action cannot be undone.\n");
    
    let term = Term::stdout();
    term.write_str("🚨 Are you absolutely sure? Type 'DELETE ALL' to confirm: ")?;
    let confirmation = term.read_line()?;
    
    if confirmation.trim() == "DELETE ALL" {
        match app.clear_database().await {
            Ok(_) => {
                println!("✅ Database cleared successfully!");
                println!("All documents and embeddings have been removed.");
            }
            Err(e) => {
                show_error(&format!("Failed to clear database: {}", e));
            }
        }
    } else {
        println!("❌ Database clear cancelled.");
    }
    
    Ok(())
}

fn handle_settings() {
    println!("\n{}", "⚙️  Settings".bright_white().bold());
    println!("{}", "─".repeat(50));
    println!("Settings configuration is not yet implemented.");
    println!("Configuration is currently handled through config.toml file.");
}

fn show_error(message: &str) {
    println!("❌ {}", message.bright_red());
}

fn show_invalid_choice() {
    println!("❌ Invalid choice. Please try again.");
}

fn show_exit_message() {
    println!("\n🐒 Thanks for using ChunkyMonkey! Going bananas for chunks! 🍌👋");
}

fn wait_for_enter() {
    let term = Term::stdout();
    term.write_str("\n⏸️  Press Enter to continue...").ok();
    term.read_line().ok();
} 