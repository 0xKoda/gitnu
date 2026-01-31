use crate::errors::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use colored::Colorize;

pub fn summary() -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(Storage::new(vault_root.clone()));

    println!("{}", "# gitnu Summary".bold());
    println!();

    // Current State
    println!("{}", "## Current State".bold());
    let current_branch = storage.read_head()?;
    println!("- Branch: {}", current_branch.green());

    if let Some(commit) = storage.get_head_commit()? {
        let short_hash = &commit.hash[..7];
        let time_ago = relative_time(&commit.timestamp);
        println!(
            "- Last commit: {} \"{}\" ({})",
            short_hash.yellow(),
            commit.message,
            time_ago.dimmed()
        );
        println!(
            "- Active domains: {}",
            commit.context_summary.domains_loaded.len()
        );
        println!(
            "- Estimated tokens: ~{}",
            commit.context_summary.token_estimate
        );
    } else {
        println!("- No commits yet");
    }

    println!();

    // What You Know
    println!("{}", "## What You Know".bold());
    let domains_dir = storage.domains_dir();
    if domains_dir.exists() {
        let mut domains = Vec::new();
        for entry in std::fs::read_dir(&domains_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('_') {
                    domains.push(name);
                }
            }
        }

        if !domains.is_empty() {
            println!("- Active domains:");
            for domain in &domains {
                println!("  - {} (see [[{}]])", domain, domain);
            }
        }

        // Show key files
        let global_dir = domains_dir.join("_global");
        if global_dir.exists() {
            println!("- Global context: [[agent]], [[conventions]]");
        }
    }

    println!();

    // What's Changed
    println!("{}", "## What's Changed".bold());
    let modified = context_mgr.get_modified_files()?;
    if !modified.is_empty() {
        println!("- Modified files:");
        for file in &modified {
            println!("  - {}", file.display());
        }
    } else {
        println!("- No uncommitted changes");
    }

    println!();

    // Available Branches
    println!("{}", "## Available Branches".bold());
    let current_branch = storage.read_head()?;
    let branches = storage.list_branches()?;
    for branch in branches {
        if branch == current_branch {
            println!("- {} (current)", branch.green());
        } else if let Some(hash) = storage.read_branch_ref(&branch)? {
            if let Some(_commit) = storage.find_commit(&hash)? {
                let commits = storage.read_commits(&current_branch)?;
                let branch_commits = storage.read_commits(&branch)?;
                let diverged = branch_commits.len().abs_diff(commits.len());
                println!("- {} (diverged {} commits)", branch, diverged);
            }
        }
    }

    Ok(())
}
