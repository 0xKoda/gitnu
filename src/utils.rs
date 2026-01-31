use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use crate::errors::*;
use std::fs;

/// Find the vault root by looking for .gitnu directory
pub fn find_vault_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let gitnu_dir = current.join(".gitnu");
        if gitnu_dir.exists() && gitnu_dir.is_dir() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(GitnuError::NoVaultFound),
        }
    }
}

/// Check if a vault exists in the given directory
pub fn vault_exists(path: &Path) -> bool {
    path.join(".gitnu").exists()
}

/// Compute SHA256 hash of content
pub fn compute_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Compute hash of a file
pub fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    Ok(compute_hash(&content))
}

/// Format file size in human-readable form
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Estimate token count (simple approximation: ~4 chars per token)
pub fn estimate_tokens(content: &str) -> usize {
    // Simple estimation: average 4 characters per token
    // This is a rough approximation; real tokenization would be more accurate
    content.len() / 4
}

/// Parse commit reference (HEAD~N, branch name, or hash)
pub fn parse_commit_ref(reference: &str) -> Result<String> {
    // For now, return as-is; the caller will resolve it
    Ok(reference.to_string())
}

/// Get relative path from base
pub fn relative_path(base: &Path, target: &Path) -> PathBuf {
    target.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| target.to_path_buf())
}

/// Format timestamp as relative time
pub fn relative_time(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*timestamp);
    
    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_weeks() < 4 {
        format!("{} weeks ago", duration.num_weeks())
    } else {
        format!("{} months ago", duration.num_days() / 30)
    }
}

/// Ensure directory exists
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
