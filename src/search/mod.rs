use crate::core::app::TldrApp;
use crate::core::types::DocumentStatus;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::path::Path;
use walkdir::WalkDir;
use console::{style, Term};
use colored::Colorize;
use std::time::Duration;

pub struct Indexer {
    patterns: Vec<String>,
}

impl Indexer {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    pub async fn index_directory(&self, directory: &Path, app: &mut TldrApp) -> Result<()> {
        // Beautiful header
        println!("\n{}", "╔══════════════════════════════════════════════════════════════╗".blue());
        println!("{}", "║                    🚀 Content Indexing Engine                    ║".blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝\n".blue());
        
        println!("{}", "📂 Scanning directory for content...".bold().cyan());
        println!("   {}", style(format!("Path: {}", directory.display())).dim());
        
        // Collect all files
        let files: Vec<_> = WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        println!("   {} files discovered", style(files.len()).yellow().bold());

        // Filter files by patterns
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|entry| self.matches_patterns(entry.path()))
            .filter(|entry| self.filter_files_by_size(entry.path()))
            .collect();

        println!("   {} files match indexing criteria", style(filtered_files.len()).green().bold());

        if filtered_files.is_empty() {
            println!("\n{}", "⚠️  No files to index".yellow().bold());
            return Ok(());
        }

        // Create multi-progress bar for better visual organization
        let multi_progress = MultiProgress::new();
        let main_progress = multi_progress.add(ProgressBar::new(filtered_files.len() as u64));
        
        // Style the main progress bar
        main_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {wide_bar:.cyan/blue} {pos}/{len} files [{elapsed_precise}]")
                .unwrap()
                .progress_chars("█░")
        );

        let mut success_count = 0;
        let mut error_count = 0;
        let mut skipped_count = 0;

        println!("\n{}", "🔄 Starting content processing...".bold().magenta());
        println!("{}", "─".repeat(60));

        // Process files one by one
        for entry in filtered_files.iter() {
            let file_name = entry.file_name().to_string_lossy();
            let file_path = entry.path();
            
            // Create individual file progress bar
            let file_progress = multi_progress.add(ProgressBar::new(100));
            file_progress.set_style(
                ProgressStyle::default_spinner()
                    .template("  {spinner:.green} {msg}")
                    .unwrap()
            );
            
            file_progress.set_message(format!("Processing {}", file_name));
            
            // Process the file
            match self.process_file(file_path, app).await {
                Ok(DocumentStatus::Added) => {
                    success_count += 1;
                    file_progress.finish_with_message(format!(
                        "{} {}",
                        style("✅").green(),
                        style(format!("{} processed successfully", file_name)).green()
                    ));
                }
                Ok(DocumentStatus::Skipped) => {
                    skipped_count += 1;
                    file_progress.finish_with_message(format!(
                        "{} {}",
                        style("⏭️").yellow(),
                        style(format!("{} already indexed, skipping", file_name)).yellow()
                    ));
                }
                Err(e) => {
                    error_count += 1;
                    file_progress.finish_with_message(format!(
                        "{} {}",
                        style("❌").red(),
                        style(format!("{} failed: {}", file_name, e)).red()
                    ));
                }
            }
            
            main_progress.inc(1);
            
            // Small delay for visual effect
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        main_progress.finish_with_message("Indexing completed!");

        // Beautiful summary
        println!("\n{}", "─".repeat(60));
        println!("{}", "📊 Indexing Summary Report".bold().cyan());
        println!("{}", "─".repeat(60));
        
        // Success section
        if success_count > 0 {
            println!("   {} {} files indexed successfully", 
                style("✅").green(), 
                style(success_count.to_string()).green().bold()
            );
        }
        
        // Skipped section
        if skipped_count > 0 {
            println!("   {} {} files skipped (already indexed)", 
                style("⏭️").yellow(), 
                style(skipped_count.to_string()).yellow().bold()
            );
        }
        
        // Error section
        if error_count > 0 {
            println!("   {} {} files failed to process", 
                style("❌").red(), 
                style(error_count.to_string()).red().bold()
            );
        }
        
        println!("   {} {} total files processed", 
            style("📁").blue(), 
            style(filtered_files.len().to_string()).blue().bold()
        );
        
        println!("{}", "─".repeat(60));
        
        // Final status
        if error_count == 0 {
            println!("{}", "🎉 All files processed successfully!".green().bold());
        } else {
            println!("{}", "⚠️  Some files had issues during processing".yellow().bold());
        }
        
        println!("\n{}", "💡 You can now search and ask questions about your indexed content!".cyan());

        Ok(())
    }

    async fn process_file(&self, file_path: &Path, app: &mut TldrApp) -> Result<DocumentStatus> {
        // Check if document already exists by trying to add it
        // The app will handle the duplicate check internally
        let content = std::fs::read_to_string(file_path)?;
        
        // Add document to the app
        app.add_document(file_path, &content).await
    }

    fn matches_patterns(&self, file_path: &Path) -> bool {
        if self.patterns.is_empty() {
            return true;
        }

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        self.patterns.iter().any(|pattern| {
            if pattern == "*" {
                return true;
            }
            
            if pattern.starts_with("*.") {
                let ext = pattern[1..].to_lowercase();
                return file_name.to_lowercase().ends_with(&ext);
            }
            
            file_name.contains(pattern)
        })
    }

    fn filter_files_by_size(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let size = metadata.len();
            // Filter out files larger than 5MB
            size <= 5 * 1024 * 1024
        } else {
            false
        }
    }

    fn generate_file_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
} 