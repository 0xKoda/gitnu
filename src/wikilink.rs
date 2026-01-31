use crate::errors::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Resolve wikilink to full path
pub fn resolve_wikilink(vault_root: &Path, link: &str) -> Result<PathBuf> {
    // Remove [[ and ]] brackets
    let name = link.trim_start_matches("[[").trim_end_matches("]]");
    
    let domains_dir = vault_root.join("domains");
    
    // Check if it contains a path separator (e.g., "authentication/patterns")
    if name.contains('/') {
        // Treat as relative path within domains/
        let path = domains_dir.join(format!("{}.md", name));
        if path.exists() {
            return Ok(path);
        }
        // Try without .md extension in case it's already there
        let path = domains_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // Search all domains for matching filename
    let mut matches = Vec::new();
    if domains_dir.exists() {
        for entry in WalkDir::new(&domains_dir) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            if entry.file_type().is_file() {
                if let Some(stem) = entry.path().file_stem() {
                    if stem == name {
                        matches.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    
    match matches.len() {
        0 => Err(GitnuError::WikilinkNotFound(name.to_string())),
        1 => Ok(matches[0].clone()),
        _ => Err(GitnuError::WikilinkAmbiguous(name.to_string(), matches)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_simple_wikilink() {
        let temp_dir = TempDir::new().unwrap();
        let vault_root = temp_dir.path();
        let domains_dir = vault_root.join("domains/test");
        fs::create_dir_all(&domains_dir).unwrap();
        
        let test_file = domains_dir.join("spec.md");
        fs::write(&test_file, "# Spec").unwrap();

        let result = resolve_wikilink(vault_root, "[[spec]]").unwrap();
        assert_eq!(result, test_file);
    }

    #[test]
    fn test_resolve_path_wikilink() {
        let temp_dir = TempDir::new().unwrap();
        let vault_root = temp_dir.path();
        let domains_dir = vault_root.join("domains/auth");
        fs::create_dir_all(&domains_dir).unwrap();
        
        let test_file = domains_dir.join("patterns.md");
        fs::write(&test_file, "# Patterns").unwrap();

        let result = resolve_wikilink(vault_root, "[[auth/patterns]]").unwrap();
        assert_eq!(result, test_file);
    }
}
