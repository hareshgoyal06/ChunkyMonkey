use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;
use glob::Pattern;
use crate::core::app::ChunkyMonkeyApp;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Indexer;

impl Indexer {
    pub fn new() -> Self {
        Self
    }

    pub async fn index_directory(&self, directory: &str, patterns: Option<&str>, app: &mut ChunkyMonkeyApp) -> Result<()> {
        let directory_path = Path::new(directory);
        if !directory_path.exists() {
            anyhow::bail!("Directory does not exist: {}", directory);
        }
        if !directory_path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", directory);
        }

        // Parse file patterns
        let patterns = if let Some(pat) = patterns {
            pat.split(',').map(|s| s.trim()).collect::<Vec<_>>()
        } else {
            vec!["*"]
        };

        // Collect files
        let files = self.collect_files(directory_path, &patterns)?;
        if files.is_empty() {
            println!("⚠️  No files found matching patterns: {}", patterns.join(", "));
            return Ok(());
        }

        // Create progress bar with better styling
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("🐒 [{spinner:.green}] [{bar:40.cyan/blue}] {pos}/{len} files [{elapsed_precise}] {msg}")
            .unwrap()
            .progress_chars("█░"));

        let mut success_count = 0;
        let mut error_count = 0;

        // Process files one by one
        for file_path in files.iter() {
            let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
            pb.set_message(format!("Processing: {}", file_name));
            
            match self.index_file(file_path, app).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    // Only show errors, not successful completions
                    pb.set_message(format!("❌ Error: {}", e));
                }
            }
            
            pb.inc(1);
            
            // Small delay to prevent overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        pb.finish_with_message("Indexing complete! 🎉");
        
        // Show summary only if there were errors
        if error_count > 0 {
            println!("\n📊 Indexing Summary:");
            println!("   ✅ Successfully indexed: {} files", success_count);
            println!("   ❌ Failed: {} files", error_count);
        }

        Ok(())
    }

    fn collect_files(&self, directory: &Path, patterns: &[&str]) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(directory)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                // Check if file matches any pattern
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let matches_pattern = patterns.iter().any(|pattern| {
                    if let Ok(pat) = Pattern::new(pattern) {
                        pat.matches(&file_name)
                    } else {
                        false
                    }
                });
                
                if matches_pattern {
                    // Filter by file size (skip files larger than 10MB)
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if metadata.len() <= 10 * 1024 * 1024 { // 10MB
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
        
        Ok(files)
    }

    async fn index_file(&self, file_path: &Path, app: &mut ChunkyMonkeyApp) -> Result<()> {
        // Add timeout to prevent hanging on problematic files
        let timeout_duration = tokio::time::Duration::from_secs(30);
        
        match tokio::time::timeout(timeout_duration, app.add_document(file_path)).await {
            Ok(result) => result.map(|_| ()), // Convert Result<u32> to Result<()>
            Err(_) => anyhow::bail!("Timeout while processing file: {}", file_path.display()),
        }
    }
} 