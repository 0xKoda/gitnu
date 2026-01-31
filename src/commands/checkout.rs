use crate::errors::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use colored::Colorize;

pub fn checkout(target: &str, force: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(Storage::new(vault_root.clone()));

    // Check for uncommitted changes
    if !force && context_mgr.has_uncommitted_changes()? {
        return Err(GitnuError::UncommittedChanges);
    }

    // Determine if target is a branch or commit
    let (commit_hash, is_branch, branch_name) = if let Some(hash) = storage.read_branch_ref(target)? {
        // It's a branch
        (hash, true, target.to_string())
    } else {
        // Try to find as commit hash
        let commit = storage.find_commit(target)?;
        match commit {
            Some(c) => (c.hash, false, String::new()),
            None => {
                return Err(GitnuError::CommitNotFound(target.to_string()));
            }
        }
    };

    // Restore snapshot
    storage.restore_snapshot(&commit_hash)?;

    // Update HEAD
    if is_branch {
        storage.write_head(&branch_name)?;
        println!("Switched to branch '{}'", branch_name.green());
    } else {
        // Detached HEAD state
        let head_path = storage.gitnu_dir().join("HEAD");
        std::fs::write(head_path, &commit_hash)?;
        println!("HEAD is now at {}", commit_hash[..7].yellow());
        println!("{}", "Note: You are in 'detached HEAD' state.".yellow());
    }

    // Show what changed
    let commit = storage.find_commit(&commit_hash)?.unwrap();
    println!("Restored context from commit {}", commit_hash[..7].yellow());
    println!("  \"{} \"", commit.message.dimmed());
    
    let summary = &commit.context_summary;
    println!("  {} domains, ~{} tokens", summary.domains_loaded.len(), summary.token_estimate);

    Ok(())
}
