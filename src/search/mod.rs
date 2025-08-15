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
        
        // Collect files to index
        let files = self.collect_files(path, &patterns)?;
        
        if files.is_empty() {
            println!("{}", "No files found matching the patterns".yellow());
            return Ok(());
        }
        
        println!("📁 Found {} files to index", files.len());
        
        // Create progress bar
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // Index each file
        for file_path in files {
            pb.set_message(format!("Indexing {}", file_path.file_name().unwrap().to_string_lossy()));
            
            match self.index_file(app, &file_path).await {
                Ok(_) => {
                    // File indexed successfully
                }
                Err(e) => {
                    println!(
                        "{} Error indexing {}: {}",
                        "❌".red(),
                        file_path.display(),
                        e
                    );
                }
            }
            
            pb.inc(1);
        }
        
        pb.finish_with_message("Indexing completed!");
        
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
    
    async fn index_file(&self, app: &mut TldrApp, file_path: &PathBuf) -> Result<()> {
        // Read file content
        let content = std::fs::read_to_string(file_path)?;
        
        if content.trim().is_empty() {
            return Ok(());
        }
        
        // Add document to app
        app.add_document(file_path.clone(), content).await?;
        
        Ok(())
    }
} 