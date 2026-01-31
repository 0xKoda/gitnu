use crate::errors::*;
use crate::storage::Storage;
use crate::utils::*;
use crate::wikilink::resolve_wikilink;
use colored::Colorize;

pub fn load(path_or_link: &str, pin: bool, list: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let mut index = storage.load_index()?;

    if list {
        // Show what's currently loaded
        println!("{}", "Currently loaded:".bold());
        if index.loaded.is_empty() {
            println!("  {}", "Nothing loaded".dimmed());
        } else {
            for path in &index.loaded {
                let is_pinned = index.pinned.contains(path);
                let marker = if is_pinned { "[pinned]" } else { "" };
                println!("  - {} {}", path.display(), marker.yellow());
            }
        }
        return Ok(());
    }

    // Resolve path (could be wikilink)
    let path = if path_or_link.starts_with("[[") {
        resolve_wikilink(&vault_root, path_or_link)?
    } else {
        vault_root.join(path_or_link)
    };

    if !path.exists() {
        return Err(GitnuError::FileNotFound(path));
    }

    let rel_path = relative_path(&vault_root, &path);

    // Add to loaded
    if !index.loaded.contains(&rel_path) {
        index.loaded.push(rel_path.clone());
    }

    // Add to pinned if requested
    if pin && !index.pinned.contains(&rel_path) {
        index.pinned.push(rel_path.clone());
    }

    storage.save_index(&index)?;

    // Calculate tokens
    let content = if path.is_file() {
        std::fs::read_to_string(&path)?
    } else {
        // Load all files in directory
        let mut total = String::new();
        for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    total.push_str(&content);
                    total.push('\n');
                }
            }
        }
        total
    };

    let tokens = estimate_tokens(&content);

    println!(
        "{} {} (+{} tokens)",
        "Loaded:".green(),
        rel_path.display(),
        tokens
    );

    if pin {
        println!("  {}", "Pinned (will always be included)".yellow());
    }

    Ok(())
}

pub fn unload(path_or_link: Option<String>, all: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let mut index = storage.load_index()?;

    if all {
        // Unload everything except pinned
        let pinned_set: std::collections::HashSet<_> = index.pinned.iter().collect();
        index.loaded.retain(|p| pinned_set.contains(p));
        storage.save_index(&index)?;
        println!("{}", "Unloaded all non-pinned files".green());
        return Ok(());
    }

    let path_str = path_or_link.ok_or_else(|| {
        GitnuError::Other("Must specify path or use --all".to_string())
    })?;

    // Resolve path
    let path = if path_str.starts_with("[[") {
        resolve_wikilink(&vault_root, &path_str)?
    } else {
        vault_root.join(&path_str)
    };

    let rel_path = relative_path(&vault_root, &path);

    // Remove from loaded
    index.loaded.retain(|p| p != &rel_path);
    
    storage.save_index(&index)?;

    println!("{} {}", "Unloaded:".yellow(), rel_path.display());

    Ok(())
}

pub fn pin(path_or_link: &str, exclude: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let mut index = storage.load_index()?;

    // Resolve path
    let path = if path_or_link.starts_with("[[") {
        resolve_wikilink(&vault_root, path_or_link)?
    } else {
        vault_root.join(path_or_link)
    };

    let rel_path = relative_path(&vault_root, &path);

    if exclude {
        // Add to excluded list
        if !index.excluded.contains(&rel_path) {
            index.excluded.push(rel_path.clone());
        }
        storage.save_index(&index)?;
        println!("{} {} (will never be included)", "Excluded:".red(), rel_path.display());
    } else {
        // Add to pinned list
        if !index.pinned.contains(&rel_path) {
            index.pinned.push(rel_path.clone());
        }
        // Also add to loaded
        if !index.loaded.contains(&rel_path) {
            index.loaded.push(rel_path.clone());
        }
        storage.save_index(&index)?;
        println!("{} {} (will always be included)", "Pinned:".green(), rel_path.display());
    }

    Ok(())
}

pub fn unpin(path_or_link: &str) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let mut index = storage.load_index()?;

    // Resolve path
    let path = if path_or_link.starts_with("[[") {
        resolve_wikilink(&vault_root, path_or_link)?
    } else {
        vault_root.join(path_or_link)
    };

    let rel_path = relative_path(&vault_root, &path);

    // Remove from pinned and excluded
    index.pinned.retain(|p| p != &rel_path);
    index.excluded.retain(|p| p != &rel_path);
    
    storage.save_index(&index)?;

    println!("{} {}", "Unpinned:".yellow(), rel_path.display());

    Ok(())
}
