use crate::errors::*;
use crate::storage::Storage;
use crate::utils::*;
use colored::Colorize;
use std::collections::HashSet;

pub fn diff(source: Option<String>, target: Option<String>) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    let (source_commit, target_commit) = match (source, target) {
        (None, None) => {
            // Diff between HEAD and working directory
            let head = storage.get_head_commit()?;
            match head {
                Some(h) => {
                    println!("Changes since last commit:");
                    show_working_diff(&storage, &h)?;
                    return Ok(());
                }
                None => {
                    println!("{}", "No commits yet".dimmed());
                    return Ok(());
                }
            }
        }
        (Some(s), None) => {
            // Diff between commit and working directory
            let source_commit = storage.find_commit(&s)?
                .ok_or_else(|| GitnuError::CommitNotFound(s.clone()))?;
            println!("Changes between commit {} and working directory:", &source_commit.hash[..7].yellow());
            show_working_diff(&storage, &source_commit)?;
            return Ok(());
        }
        (Some(s), Some(t)) => {
            // Diff between two commits or branches
            let source_commit = resolve_target(&storage, &s)?;
            let target_commit = resolve_target(&storage, &t)?;
            (source_commit, target_commit)
        }
        (None, Some(_)) => {
            return Err(GitnuError::Other("Invalid diff arguments".to_string()));
        }
    };

    // Show diff between two commits
    println!(
        "Comparing {}..{}",
        source_commit.hash[..7].yellow(),
        target_commit.hash[..7].yellow()
    );
    println!();

    show_commit_diff(&source_commit, &target_commit)?;

    Ok(())
}

fn resolve_target(storage: &Storage, target: &str) -> Result<crate::models::Commit> {
    // Try as branch first
    if let Some(hash) = storage.read_branch_ref(target)? {
        if let Some(commit) = storage.find_commit(&hash)? {
            return Ok(commit);
        }
    }
    
    // Try as commit hash
    storage.find_commit(target)?
        .ok_or_else(|| GitnuError::CommitNotFound(target.to_string()))
}

fn show_working_diff(storage: &Storage, head_commit: &crate::models::Commit) -> Result<()> {
    use crate::context::ContextManager;
    
    let context_mgr = ContextManager::new(Storage::new(storage.vault_root.clone()));
    let summary = context_mgr.calculate_context_summary(Some(head_commit))?;

    println!("{}", "Context Changes:".bold());

    if !summary.files_added.is_empty() {
        for file in &summary.files_added {
            println!("+ Added file: {} ", file.display().to_string().green());
        }
    }

    if !summary.files_modified.is_empty() {
        for file in &summary.files_modified {
            println!("~ Modified: {}", file.display().to_string().yellow());
        }
    }

    if !summary.files_removed.is_empty() {
        for file in &summary.files_removed {
            println!("- Removed: {}", file.display().to_string().red());
        }
    }

    if summary.files_added.is_empty() && summary.files_modified.is_empty() && summary.files_removed.is_empty() {
        println!("  {}", "No changes".dimmed());
    }

    println!();
    let token_delta = summary.token_estimate as i64 - head_commit.context_summary.token_estimate as i64;
    let sign = if token_delta >= 0 { "+" } else { "" };
    println!("Token delta: {}{} tokens", sign, token_delta);

    Ok(())
}

fn show_commit_diff(
    source: &crate::models::Commit,
    target: &crate::models::Commit,
) -> Result<()> {
    println!("{}", "Context Changes:".bold());

    let source_files: HashSet<_> = source.context_summary.files_added.iter()
        .chain(source.context_summary.files_modified.iter())
        .collect();
    
    let target_files: HashSet<_> = target.context_summary.files_added.iter()
        .chain(target.context_summary.files_modified.iter())
        .collect();

    // Files in target but not in source
    for file in target_files.difference(&source_files) {
        println!("+ Added file: {}", file.display().to_string().green());
    }

    // Files in source but not in target
    for file in source_files.difference(&target_files) {
        println!("- Removed: {}", file.display().to_string().red());
    }

    // Files in both (potentially modified)
    for file in source_files.intersection(&target_files) {
        println!("~ Modified: {}", file.display().to_string().yellow());
    }

    println!();
    
    let token_delta = target.context_summary.token_estimate as i64 
        - source.context_summary.token_estimate as i64;
    let sign = if token_delta >= 0 { "+" } else { "" };
    println!("Token delta: {}{} tokens", sign, token_delta);

    // Show domain differences
    let source_domains: HashSet<_> = source.context_summary.domains_loaded.iter().collect();
    let target_domains: HashSet<_> = target.context_summary.domains_loaded.iter().collect();

    if source_domains != target_domains {
        println!();
        println!("{}", "Domain Changes:".bold());
        
        for domain in target_domains.difference(&source_domains) {
            println!("+ Added domain: domains/{}/", domain.green());
        }
        
        for domain in source_domains.difference(&target_domains) {
            println!("- Removed from context: domains/{}/", domain.red());
        }
    }

    Ok(())
}
