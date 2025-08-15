use crate::pinecone::PineconeConfig;
use anyhow::Result;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub openai: OpenAIConfig,
    pub pinecone: PineconeConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tldr");

        let config = Config::builder()
            // Start with default values
            .add_source(File::from(config_dir.join("config.toml")).required(false))
            // Add environment variables with prefix
            .add_source(Environment::with_prefix("TLDR"))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub fn from_env() -> Result<Self> {
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        
        // Make Pinecone optional for local-only operation
        let pinecone_api_key = std::env::var("PINECONE_API_KEY").ok();
        let pinecone_environment = std::env::var("PINECONE_ENVIRONMENT").ok();
        let pinecone_index = std::env::var("PINECONE_INDEX").ok();

        Ok(Self {
            openai: OpenAIConfig {
                api_key: openai_api_key.unwrap_or_default(),
            },
            pinecone: PineconeConfig {
                api_key: pinecone_api_key.unwrap_or_default(),
                environment: pinecone_environment.unwrap_or_default(),
                index_name: pinecone_index.unwrap_or_default(),
            },
            database: DatabaseConfig {
                path: "tldr.db".to_string(),
            },
        })
    }
} 