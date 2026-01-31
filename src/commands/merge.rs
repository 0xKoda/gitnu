use crate::errors::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use colored::Colorize;

pub fn merge(source_branch: &str, into_branch: Option<String>, squash: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(Storage::new(vault_root.clone()));

    // Get target branch (current if not specified)
    let target_branch = match into_branch {
        Some(b) => b,
        None => storage.read_head()?,
    };

    // Get commits
    let source_commit_hash = storage.read_branch_ref(source_branch)?
        .ok_or_else(|| GitnuError::BranchNotFound(source_branch.to_string()))?;
    let target_commit_hash = storage.read_branch_ref(&target_branch)?
        .ok_or_else(|| GitnuError::BranchNotFound(target_branch.clone()))?;

    let source_commit = storage.find_commit(&source_commit_hash)?
        .ok_or_else(|| GitnuError::CommitNotFound(source_commit_hash.clone()))?;
    let target_commit = storage.find_commit(&target_commit_hash)?
        .ok_or_else(|| GitnuError::CommitNotFound(target_commit_hash.clone()))?;

    println!(
        "Merging {} into {}",
        source_branch.green(),
        target_branch.green()
    );

    // Check if we're on the target branch
    let current_branch = storage.read_head()?;
    if current_branch != target_branch {
        println!("Switching to branch '{}'...", target_branch.green());
        // Restore target branch state
        storage.restore_snapshot(&target_commit.hash)?;
        storage.write_head(&target_branch)?;
    }

    // Perform merge (simplified - just copy source files over target)
    // In a real implementation, this would do smart merging of markdown files
    storage.restore_snapshot(&source_commit.hash)?;

    // Calculate new context summary
    let summary = context_mgr.calculate_context_summary(Some(&target_commit))?;

    // Create merge commit
    let merge_message = if squash {
        format!("Merge {}: {} (squashed)", source_branch, source_commit.message)
    } else {
        format!("Merge {}: {}", source_branch, source_commit.message)
    };

    let mut commit_data = Vec::new();
    commit_data.extend_from_slice(b"parent ");
    commit_data.extend_from_slice(target_commit.hash.as_bytes());
    commit_data.extend_from_slice(b"\nparent ");
    commit_data.extend_from_slice(source_commit.hash.as_bytes());
    commit_data.extend_from_slice(b"\n");
    commit_data.extend_from_slice(merge_message.as_bytes());
    commit_data.extend_from_slice(b"\n");
    commit_data.extend_from_slice(chrono::Utc::now().to_rfc3339().as_bytes());
    
    let hash = compute_hash(&commit_data);
    let snapshot_path = storage.create_snapshot(&hash)?;

    let merge_commit = crate::models::Commit {
        hash: hash.clone(),
        parent: Some(target_commit.hash.clone()),
        timestamp: chrono::Utc::now(),
        author: crate::models::Author::Agent {
            model: "gitnu-merge".to_string(),
            session_id: None,
        },
        message: merge_message.clone(),
        context_summary: summary,
        snapshot_path: relative_path(&vault_root, &snapshot_path),
    };

    // Save merge commit
    storage.append_commit(&target_branch, &merge_commit)?;
    storage.write_branch_ref(&target_branch, &hash)?;

    println!();
    println!("{}", "Merge successful!".green().bold());
    println!("  Auto-merged files from {}", source_branch);
    println!();
    println!("Created merge commit {}", hash[..7].yellow());
    println!("  \"{}\"", merge_message);

    Ok(())
}
