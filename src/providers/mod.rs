pub mod base;
pub mod claude;
pub mod coco;
pub mod codex;
pub mod gemini;

use crate::error::{Result, WaylogError};
use std::sync::Arc;

/// Get a provider by name
pub fn get_provider(name: &str) -> Result<Arc<dyn base::Provider>> {
    match name.to_lowercase().as_str() {
        "codex" => Ok(Arc::new(codex::CodexProvider::new())),
        "claude" | "claude-code" => Ok(Arc::new(claude::ClaudeProvider::new())),
        "gemini" => Ok(Arc::new(gemini::GeminiProvider::new())),
        "coco" => Ok(Arc::new(coco::CocoProvider::new())),
        _ => Err(WaylogError::ProviderNotFound(name.to_string())),
    }
}

/// Get all available providers
#[allow(dead_code)]
pub fn all_providers() -> Vec<Arc<dyn base::Provider>> {
    vec![
        Arc::new(codex::CodexProvider::new()),
        Arc::new(claude::ClaudeProvider::new()),
        Arc::new(gemini::GeminiProvider::new()),
        Arc::new(coco::CocoProvider::new()),
    ]
}
/// Get a list of supported provider names
pub fn list_providers() -> Vec<&'static str> {
    vec!["claude", "gemini", "codex", "coco"]
}
