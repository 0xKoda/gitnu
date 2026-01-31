use crate::errors::*;
use crate::models::*;
use crate::storage::Storage;
use crate::utils::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ContextManager {
    storage: Storage,
}

impl ContextManager {
    pub fn new(storage: Storage) -> Self {
        ContextManager { storage }
    }

    /// Calculate context summary for current state
    pub fn calculate_context_summary(&self, previous_commit: Option<&Commit>) -> Result<ContextSummary> {
        let domains_dir = self.storage.domains_dir();
        let mut domains_loaded = Vec::new();
        let mut files_modified = Vec::new();
        let mut files_added = Vec::new();
        let mut files_removed = Vec::new();
        let mut total_content = String::new();

        // Collect current files
        let mut current_files = std::collections::HashMap::new();
        if domains_dir.exists() {
            for entry in WalkDir::new(&domains_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let rel_path = relative_path(&self.storage.vault_root, path);
                    
                    // Track domains
                    if let Some(domain) = self.extract_domain(&rel_path) {
                        if !domains_loaded.contains(&domain) {
                            domains_loaded.push(domain);
                        }
                    }

                    // Read content for token estimation
                    if let Ok(content) = fs::read_to_string(path) {
                        total_content.push_str(&content);
                        total_content.push('\n');
                    }

                    current_files.insert(rel_path.clone(), hash_file(path)?);
                }
            }
        }

        // Compare with previous commit if available
        if let Some(prev) = previous_commit {
            let manifest_path = self.storage.objects_dir()
                .join(&prev.hash)
                .join("manifest.json");
            
            if manifest_path.exists() {
                let manifest_content = fs::read_to_string(manifest_path)?;
                let manifest: Manifest = serde_json::from_str(&manifest_content)?;
                
                let mut previous_files = std::collections::HashMap::new();
                for file_info in manifest.files {
                    previous_files.insert(file_info.path.clone(), file_info.hash);
                }

                // Find added, modified, removed
                for (path, hash) in &current_files {
                    match previous_files.get(path) {
                        Some(prev_hash) if prev_hash != hash => {
                            files_modified.push(path.clone());
                        }
                        None => {
                            files_added.push(path.clone());
                        }
                        _ => {}
                    }
                }

                for path in previous_files.keys() {
                    if !current_files.contains_key(path) {
                        files_removed.push(path.clone());
                    }
                }
            }
        } else {
            // First commit - everything is added
            files_added = current_files.keys().cloned().collect();
        }

        let token_estimate = estimate_tokens(&total_content);

        Ok(ContextSummary {
            domains_loaded,
            files_modified,
            files_added,
            files_removed,
            token_estimate,
        })
    }

    /// Extract domain name from path (e.g., "domains/myproject/spec.md" -> "myproject")
    fn extract_domain(&self, path: &Path) -> Option<String> {
        let components: Vec<_> = path.components()
            .map(|c| c.as_os_str().to_str().unwrap_or(""))
            .collect();
        
        if components.len() >= 2 && components[0] == "domains" {
            Some(components[1].to_string())
        } else {
            None
        }
    }

    /// Get list of modified files since last commit
    pub fn get_modified_files(&self) -> Result<Vec<PathBuf>> {
        let head_commit = self.storage.get_head_commit()?;
        let summary = self.calculate_context_summary(head_commit.as_ref())?;
        
        let mut modified = Vec::new();
        modified.extend(summary.files_modified);
        modified.extend(summary.files_added);
        
        Ok(modified)
    }

    /// Check if there are uncommitted changes
    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let modified = self.get_modified_files()?;
        Ok(!modified.is_empty())
    }

    /// Get all files in context
    pub fn get_all_files(&self) -> Result<Vec<PathBuf>> {
        let domains_dir = self.storage.domains_dir();
        let mut files = Vec::new();
        
        if domains_dir.exists() {
            for entry in WalkDir::new(&domains_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let rel_path = relative_path(&self.storage.vault_root, path);
                    files.push(rel_path);
                }
            }
        }
        
        Ok(files)
    }

    /// Load context as single document
    pub fn load_context(&self, compress: bool) -> Result<String> {
        let domains_dir = self.storage.domains_dir();
        let mut content = String::new();
        
        if domains_dir.exists() {
            for entry in WalkDir::new(&domains_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let rel_path = relative_path(&self.storage.vault_root, path);
                    content.push_str(&format!("\n# File: {}\n\n", rel_path.display()));
                    
                    if let Ok(file_content) = fs::read_to_string(path) {
                        content.push_str(&file_content);
                        content.push_str("\n\n");
                    }
                }
            }
        }
        
        if compress {
            content = self.compress_markdown(&content);
        }
        
        Ok(content)
    }

    /// Simple markdown compression
    fn compress_markdown(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .split("\n\n\n")
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
