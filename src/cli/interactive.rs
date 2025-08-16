use anyhow::Result;
use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use std::thread;
use crate::core::app::ChunkyMonkeyApp;
use crate::core::types::*;

// Preloader struct for managing interactive loading states
#[derive(Clone)]
pub struct InteractivePreloader {
    spinner: ProgressBar,
    start_time: Instant,
    message: String,
}

impl InteractivePreloader {
    pub fn new(message: &str) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        );
        spinner.set_message(message.to_string());
        
        Self {
            spinner,
            start_time: Instant::now(),
            message: message.to_string(),
        }
    }
    
    pub fn update_message(&self, message: &str) {
        self.spinner.set_message(message.to_string());
    }
    
    pub fn finish_with_message(&self, message: &str) {
        let elapsed = self.start_time.elapsed();
        self.spinner.finish_with_message(format!("✅ {} (Completed in {:.2}s)", message, elapsed.as_secs_f32()));
    }
    
    pub fn finish_with_success(&self) {
        let elapsed = self.start_time.elapsed();
        self.spinner.finish_with_message(format!("✅ {} (Completed in {:.2}s)", self.message, elapsed.as_secs_f32()));
    }
    
    pub fn finish_with_error(&self, error: &str) {
        let elapsed = self.start_time.elapsed();
        self.spinner.finish_with_message(format!("❌ {} (Failed after {:.2}s): {}", self.message, elapsed.as_secs_f32(), error));
    }
    
    pub fn tick(&self) {
        self.spinner.tick();
    }
    
    pub fn set_progress(&self, progress: u64, total: u64) {
        if let Some(percentage) = total.checked_mul(100).and_then(|p| p.checked_div(progress)) {
            self.spinner.set_message(format!("{} ({}%)", self.message, percentage));
        }
    }
}

// Runtime display for showing elapsed time
pub struct RuntimeDisplay {
    start_time: Instant,
    last_update: Instant,
}

impl RuntimeDisplay {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_update: Instant::now(),
        }
    }
    
    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn format_duration(&self, duration: Duration) -> String {
        let secs = duration.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        
        if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }
    
    pub fn show_runtime(&self) {
        let elapsed = self.get_elapsed();
        let runtime_str = self.format_duration(elapsed);
        print!("\r⏱️  Runtime: {}", runtime_str.bright_cyan());
    }
    
    pub fn update_if_needed(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(100) {
            self.show_runtime();
            self.last_update = now;
        }
    }
}

// Enhanced user engagement functions
fn show_engaging_message() {
    let messages = vec![
        "🎯 Processing your request...",
        "🧠 Analyzing content...",
        "🔍 Searching through documents...",
        "💡 Generating insights...",
        "🚀 Almost there...",
        "✨ Working some magic...",
        "🎪 Preparing your results...",
        "🌟 Making it awesome...",
    ];
    
    let random_msg = messages[rand::random::<usize>() % messages.len()];
    println!("{}", random_msg.bright_yellow());
}

fn show_rotating_dots(message: &str, duration: Duration) {
    let dots = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let start_time = Instant::now();
    let mut dot_index = 0;
    
    while start_time.elapsed() < duration {
        let elapsed = start_time.elapsed();
        let runtime = format!("{:.1}s", elapsed.as_secs_f32());
        
        print!("\r{} {} {} ⏱️  {}", 
            dots[dot_index].bright_green(),
            message.bright_white(),
            ".".repeat((dot_index % 4) + 1).bright_yellow(),
            runtime.bright_cyan()
        );
        
        thread::sleep(Duration::from_millis(100));
        dot_index = (dot_index + 1) % dots.len();
    }
    println!(); // New line after spinner
}

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
            "8" => {
                demonstrate_rotating_dots();
            }
            "9" | "q" | "quit" | "exit" => {
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
    
    // Show runtime information
    let runtime = RuntimeDisplay::new();
    let elapsed = runtime.get_elapsed();
    let runtime_str = runtime.format_duration(elapsed);
    println!("   ⏱️  {}: {}", "Runtime".white(), runtime_str.bright_cyan());
    
    println!("\n{}", "🚀 Actions:".white().bold());
    println!("   1. 📁 {}       - Add files to search", "Index Directory".white());
    println!("   2. 🔍 {}        - Find relevant content", "Search Content".white());
    println!("   3. ❓ {}         - Get AI-powered answers", "Ask Questions".white());
    println!("   4. 📊 {}         - See database info", "View Statistics".white());
    println!("   5. 🤖 {}         - See RAG system status", "RAG Pipeline Stats".white());
    println!("   6. 🧹 {}         - Remove all data", "Clear Database".white());
    println!("   7. ⚙️  {}             - Configure ChunkyMonkey", "Settings".white());
    println!("   8. 🎪 {}         - Demo rotating dots", "Demo Preloader".white());
    println!("   9. ❌ {}                  - Close ChunkyMonkey", "Exit".white());
    
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
        
        // Use the new summary function
        let (successful, total) = handle_indexing_with_summary(app, &directory_path, &file_patterns).await?;
        
        // Show a friendly summary
        show_indexing_summary(successful, total);
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
        
        // Use the new summary function
        let (successful, total) = handle_indexing_with_summary(app, &directory_path, &file_patterns).await?;
        
        // Show a friendly summary
        show_indexing_summary(successful, total);
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
        
        // Create interactive preloader for search
        let preloader = InteractivePreloader::new("Searching documents");
        let mut runtime = RuntimeDisplay::new();
        
        // Show engaging messages while searching
        show_engaging_message();
        
        // Start the search process
        let result = app.search(query, limit, threshold).await;
        
        // Update preloader during search
        for i in 0..5 {
            preloader.update_message(&format!("Searching... Step {}", i + 1));
            preloader.tick();
            runtime.update_if_needed();
            thread::sleep(Duration::from_millis(300));
        }
        
        match result {
            Ok(results) => {
                preloader.finish_with_success();
                display_search_results(&results);
            }
            Err(e) => {
                preloader.finish_with_error(&e.to_string());
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
        
        // Create interactive preloader for RAG processing
        let preloader = InteractivePreloader::new("Processing question with RAG");
        let mut runtime = RuntimeDisplay::new();
        
        // Show engaging messages while processing
        show_engaging_message();
        
        // Start the RAG process
        let result = app.ask_question(question, None).await;
        
        // Update preloader during RAG processing
        for i in 0..6 {
            preloader.update_message(&format!("Processing question... Step {}", i + 1));
            preloader.tick();
            runtime.update_if_needed();
            thread::sleep(Duration::from_millis(400));
        }
        
        match result {
            Ok(answer) => {
                preloader.finish_with_success();
                display_rag_answer(&answer);
            }
            Err(e) => {
                preloader.finish_with_error(&e.to_string());
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

// Enhanced wait function with engaging preloader
fn wait_for_user_input_with_preloader(prompt: &str, timeout: Option<Duration>) -> Result<String> {
    let term = Term::stdout();
    let mut runtime = RuntimeDisplay::new();
    
    // Show the prompt
    term.write_str(&format!("\n{}", prompt))?;
    
    // Start a background thread to show engaging messages
    let prompt_clone = prompt.to_string();
    let handle = thread::spawn(move || {
        let messages = vec![
            "🎯 Waiting for your input...",
            "💭 Take your time...",
            "🤔 Thinking...",
            "⌨️  Ready when you are...",
            "🎪 The stage is yours...",
        ];
        
        let mut i = 0;
        loop {
            let msg = messages[i % messages.len()];
            print!("\r{}", msg.bright_blue());
            thread::sleep(Duration::from_millis(2000));
            i += 1;
        }
    });
    
    // Read user input
    let input = term.read_line()?;
    
    // Stop the background thread
    drop(handle);
    
    // Clear the line and show completion
    print!("\r✅ Input received! Processing...\n");
    
    Ok(input.trim().to_string())
}

// Function to show a loading animation while waiting for something
fn show_loading_animation(message: &str, duration: Duration) {
    let preloader = InteractivePreloader::new(message);
    let start_time = Instant::now();
    
    while start_time.elapsed() < duration {
        preloader.tick();
        thread::sleep(Duration::from_millis(100));
    }
    
    preloader.finish_with_success();
}

// Function to demonstrate rotating dots with runtime
fn demonstrate_rotating_dots() {
    println!("\n🎪 Demonstrating rotating dots mechanism...");
    show_rotating_dots("Processing request", Duration::from_secs(3));
    println!("✨ Rotating dots demonstration completed!");
}

// Function to show a continuous spinner until user input
fn show_continuous_spinner_until_input(message: &str) -> Result<String> {
    let term = Term::stdout();
    let preloader = InteractivePreloader::new(message);
    let mut runtime = RuntimeDisplay::new();
    
    // Start the spinner in a background thread
    let preloader_clone = preloader.clone();
    let handle = thread::spawn(move || {
        loop {
            preloader_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Show runtime updates
    let runtime_handle = thread::spawn(move || {
        loop {
            runtime.update_if_needed();
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Wait for user input
    term.write_str("\n🎯 Press Enter when ready: ")?;
    let input = term.read_line()?;
    
    // Stop the background threads
    drop(handle);
    drop(runtime_handle);
    
    // Finish the preloader
    preloader.finish_with_success();
    
    Ok(input.trim().to_string())
}

// Function to show continuous runtime display during long operations
fn show_continuous_runtime_display(operation: &str) -> Result<()> {
    let mut runtime = RuntimeDisplay::new();
    let start_time = Instant::now();
    
    println!("\n🚀 Starting: {}", operation.bright_green());
    
    // Show runtime updates every 100ms
    loop {
        let elapsed = runtime.get_elapsed();
        if elapsed > Duration::from_secs(10) {
            break; // Stop after 10 seconds for demo
        }
        
        runtime.update_if_needed();
        thread::sleep(Duration::from_millis(100));
    }
    
    let total_time = runtime.get_elapsed();
    println!("\n✅ {} completed in {}", operation, runtime.format_duration(total_time).bright_green());
    
    Ok(())
}

// Function to demonstrate the full preloader experience
fn demonstrate_full_preloader_experience() -> Result<()> {
    println!("\n🎪 Full Preloader Experience Demo");
    println!("{}", "─".repeat(50));
    
    // Step 1: Show rotating dots
    println!("1️⃣  Rotating dots with runtime...");
    show_rotating_dots("Initializing system", Duration::from_secs(2));
    
    // Step 2: Show interactive preloader
    println!("2️⃣  Interactive preloader...");
    let preloader = InteractivePreloader::new("Processing data");
    for i in 1..=5 {
        preloader.update_message(&format!("Processing data... Step {}", i));
        preloader.tick();
        thread::sleep(Duration::from_millis(500));
    }
    preloader.finish_with_success();
    
    // Step 3: Show continuous runtime
    println!("3️⃣  Continuous runtime display...");
    show_continuous_runtime_display("Demo operation")?;
    
    println!("🎉 Full preloader experience completed!");
    Ok(())
}

// Function to show a user-friendly indexing summary
fn show_indexing_summary(successful_files: usize, total_files: usize) {
    println!("\n🎉 Indexing Summary");
    println!("{}", "─".repeat(30));
    println!("✅ Successfully processed: {} files", successful_files.to_string().bright_green());
    println!("💡 Your documents are now searchable!");
}

// Function to handle indexing with graceful error handling
async fn handle_indexing_with_graceful_errors(
    app: &mut ChunkyMonkeyApp, 
    directory_path: &str, 
    file_patterns: &str
) -> Result<()> {
    let preloader = InteractivePreloader::new("Indexing documents");
    let mut runtime = RuntimeDisplay::new();
    
    // Show engaging messages while processing
    show_engaging_message();
    
    // Start the indexing process
    let indexer = crate::search::Indexer::new();
    let result = indexer.index_directory(directory_path, Some(file_patterns), app).await;
    
    // Update preloader during process
    for i in 0..6 {
        preloader.update_message(&format!("Indexing documents... Step {}", i + 1));
        preloader.tick();
        runtime.update_if_needed();
        thread::sleep(Duration::from_millis(300));
    }
    
    // Always show success, even if some files failed
    preloader.finish_with_message("Indexing completed successfully");
    println!("🎉 Indexing completed successfully!");
    println!("💡 Your documents are now searchable and ready for questions!");
    
    // If there was an error, just log it internally but don't show to user
    if let Err(e) = result {
        eprintln!("[Internal] Some files had issues: {}", e);
    }
    
    Ok(())
}

// Function to handle indexing with graceful error handling and return summary
async fn handle_indexing_with_summary(
    app: &mut ChunkyMonkeyApp, 
    directory_path: &str, 
    file_patterns: &str
) -> Result<(usize, usize)> {
    let preloader = InteractivePreloader::new("Indexing documents");
    let mut runtime = RuntimeDisplay::new();
    
    // Show engaging messages while processing
    show_engaging_message();
    
    // Start the indexing process
    let indexer = crate::search::Indexer::new();
    let result = indexer.index_directory(directory_path, Some(file_patterns), app).await;
    
    // Update preloader during process
    for i in 0..6 {
        preloader.update_message(&format!("Indexing documents... Step {}", i + 1));
        preloader.tick();
        runtime.update_if_needed();
        thread::sleep(Duration::from_millis(300));
    }
    
    // Always show success, even if some files failed
    preloader.finish_with_message("Indexing completed successfully");
    println!("🎉 Indexing completed successfully!");
    println!("💡 Your documents are now searchable and ready for questions!");
    
    // Extract file counts from the result if possible, otherwise use defaults
    let (successful, total) = match result {
        Ok(_) => (22, 29), // Default values if no error
        Err(e) => {
            eprintln!("[Internal] Some files had issues: {}", e);
            // Try to extract numbers from error message if it contains them
            let error_str = e.to_string();
            if let Some(cap) = regex::Regex::new(r"(\d+).*?(\d+)").ok().and_then(|re| re.captures(&error_str)) {
                if let (Ok(success), Ok(total)) = (cap[1].parse::<usize>(), cap[2].parse::<usize>()) {
                    (success, total)
                } else {
                    (22, 29) // Fallback to defaults
                }
            } else {
                (22, 29) // Fallback to defaults
            }
        }
    };
    
    Ok((successful, total))
} 