use crate::core::app::TldrApp;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use walkdir::WalkDir;

pub mod indexer;

pub struct Indexer {
    chunk_size: usize,
    overlap: usize,
}

impl Indexer {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }
    
    pub async fn index_directory(
        &self,
        app: &mut TldrApp,
        path: &PathBuf,
        patterns: &str,
    ) -> Result<()> {
        let patterns: Vec<String> = patterns
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if patterns.is_empty() {
            return Err(anyhow::anyhow!("No valid file patterns provided"));
        }
        
        println!("🔍 Collecting files...");
        
        // Collect files to index
        let files = self.collect_files(path, &patterns)?;
        
        if files.is_empty() {
            println!("{}", "No files found matching the patterns".yellow());
            return Ok(());
        }
        
        println!("📁 Found {} files to index", files.len());
        
        // Filter out very large files (>10MB) to prevent memory issues
        println!("🔍 Filtering files by size...");
        let (valid_files, skipped_files) = self.filter_files_by_size(&files)?;
        
        if !skipped_files.is_empty() {
            println!("⚠️  Skipped {} files larger than 10MB to prevent memory issues", skipped_files.len());
        }
        
        if valid_files.is_empty() {
            println!("{}", "No valid files to index after filtering".yellow());
            return Ok(());
        }
        
        println!("📄 Processing {} files...", valid_files.len());
        
        // Create progress bar
        let pb = ProgressBar::new(valid_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // Process files one by one with better error handling
        let mut processed = 0;
        let mut errors = 0;
        
        for (i, file_path) in valid_files.iter().enumerate() {
            pb.set_message(format!("Indexing {} ({}/{})", 
                file_path.file_name().unwrap().to_string_lossy(),
                i + 1,
                valid_files.len()
            ));
            
            // Process file with timeout and error handling
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30), // 30 second timeout per file
                self.index_file(app, file_path)
            ).await {
                Ok(Ok(_)) => {
                    processed += 1;
                }
                Ok(Err(e)) => {
                    errors += 1;
                    println!(
                        "{} Error indexing {}: {}",
                        "❌".red(),
                        file_path.display(),
                        e
                    );
                }
                Err(_) => {
                    errors += 1;
                    println!(
                        "{} Timeout indexing {} (took longer than 30 seconds)",
                        "⏰".yellow(),
                        file_path.display()
                    );
                }
            }
            
            pb.inc(1);
            
            // Small delay between files to prevent overwhelming the system
            if i < valid_files.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }
        
        pb.finish_with_message("Indexing completed!");
        
        // Summary
        println!("\n{}", "📊 Indexing Summary:".bold().green());
        println!("   ✅ Successfully processed: {}", processed);
        if errors > 0 {
            println!("   ❌ Errors encountered: {}", errors);
        }
        if !skipped_files.is_empty() {
            println!("   ⚠️  Skipped (too large): {}", skipped_files.len());
        }
        println!("   📁 Total files found: {}", files.len());
        
        Ok(())
    }
    
    fn collect_files(&self, root_path: &PathBuf, patterns: &[String]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Skip directories and hidden files
            if path.is_dir() || path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            
            // Check if file matches any pattern
            let file_name = path.file_name().unwrap().to_string_lossy();
            let matches_pattern = patterns.iter().any(|pattern| {
                if pattern.starts_with('*') {
                    // Simple glob matching
                    let suffix = &pattern[1..];
                    file_name.ends_with(suffix)
                } else {
                    file_name == *pattern
                }
            });
            
            if matches_pattern {
                files.push(path.to_path_buf());
            }
        }
        
        Ok(files)
    }
    
    fn filter_files_by_size(&self, files: &[PathBuf]) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
        
        let mut valid_files = Vec::new();
        let mut skipped_files = Vec::new();
        
        for file_path in files {
            match std::fs::metadata(file_path) {
                Ok(metadata) => {
                    if metadata.len() <= MAX_FILE_SIZE {
                        valid_files.push(file_path.clone());
                    } else {
                        skipped_files.push(file_path.clone());
                    }
                }
                Err(_) => {
                    // If we can't get metadata, skip the file
                    skipped_files.push(file_path.clone());
                }
            }
        }
        
        Ok((valid_files, skipped_files))
    }
    
    async fn index_file(&self, app: &mut TldrApp, file_path: &PathBuf) -> Result<()> {
        // Read file content
        let content = std::fs::read_to_string(file_path)?;
        
        if content.trim().is_empty() {
            return Ok(());
        }
        
        println!("   📖 Read file: {} ({} bytes)", file_path.display(), content.len());
        
        // Add document to app
        println!("   🔄 Processing document...");
        app.add_document(file_path.clone(), content).await?;
        println!("   ✅ Document processed successfully");
        
        Ok(())
    }
} 