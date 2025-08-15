use crate::core::app::TldrApp;
use anyhow::Result;
use indicatif::ProgressBar;
use std::path::Path;
use walkdir::WalkDir;

pub struct Indexer {
    patterns: Vec<String>,
}

impl Indexer {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    pub async fn index_directory(&self, directory: &Path, app: &mut TldrApp) -> Result<()> {
        println!("⏳ Indexing directory: {}", directory.display());
        
        // Collect all files
        let files: Vec<_> = WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        println!("📁 Found {} files to index", files.len());

        // Filter files by patterns
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|entry| self.matches_patterns(entry.path()))
            .filter(|entry| self.filter_files_by_size(entry.path()))
            .collect();

        println!("🎯 {} files match patterns and size requirements", filtered_files.len());

        if filtered_files.is_empty() {
            println!("⚠️  No files to index");
            return Ok(());
        }

        // Create progress bar
        let progress_bar = ProgressBar::new(filtered_files.len() as u64);
        progress_bar.set_message("Indexing files...");

        let mut success_count = 0;
        let mut error_count = 0;

        // Process files one by one
        for (_i, entry) in filtered_files.iter().enumerate() {
            progress_bar.set_message("Processing files...");
            
            match self.process_file(entry.path(), app).await {
                Ok(_) => {
                    success_count += 1;
                    println!("✅ Indexed: {}", entry.path().display());
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ Failed to index {}: {}", entry.path().display(), e);
                }
            }

            progress_bar.inc(1);
            
            // Small delay to prevent overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        progress_bar.finish_with_message("Indexing completed!");

        println!("\n📊 Indexing Summary:");
        println!("   ✅ Successfully indexed: {}", success_count);
        println!("   ❌ Failed: {}", error_count);
        println!("   📁 Total processed: {}", filtered_files.len());

        Ok(())
    }

    async fn process_file(&self, file_path: &Path, app: &mut TldrApp) -> Result<()> {
        // Read file content
        let content = std::fs::read_to_string(file_path)?;
        
        // Add document to the app
        app.add_document(file_path, &content).await?;
        
        Ok(())
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
                let ext = pattern.trim_start_matches("*.");
                return file_name.ends_with(&format!(".{}", ext));
            }
            
            file_name == pattern
        })
    }

    fn filter_files_by_size(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let size = metadata.len();
            // Limit to 10MB files
            size <= 10 * 1024 * 1024
        } else {
            false
        }
    }
} 