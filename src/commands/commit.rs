use crate::errors::*;
use crate::models::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use chrono::Utc;
use colored::Colorize;

pub fn commit(message: &str, author_type: &str, model: Option<String>) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(Storage::new(vault_root.clone()));

    // Get current branch
    let current_branch = storage.read_head()?;

    // Get previous commit
    let parent_commit = storage.get_head_commit()?;
    let parent_hash = parent_commit.as_ref().map(|c| c.hash.clone());

    // Calculate context summary
    let summary = context_mgr.calculate_context_summary(parent_commit.as_ref())?;

    // Check if there are changes
    if parent_commit.is_some() 
        && summary.files_added.is_empty() 
        && summary.files_modified.is_empty() 
        && summary.files_removed.is_empty() {
        println!("{}", "No changes to commit".yellow());
        return Ok(());
    }

    // Create author
    let author = match author_type {
        "human" => Author::Human {
            name: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
        },
        "agent" => Author::Agent {
            model: model.unwrap_or_else(|| "claude-3-5-sonnet".to_string()),
            session_id: None,
        },
        _ => {
            return Err(GitnuError::Other(format!(
                "Invalid author type: {}. Use 'human' or 'agent'",
                author_type
            )));
        }
    };

    // Create commit hash
    let mut commit_data = Vec::new();
    if let Some(ref parent) = parent_hash {
        commit_data.extend_from_slice(b"parent ");
        commit_data.extend_from_slice(parent.as_bytes());
        commit_data.extend_from_slice(b"\n");
    }
    commit_data.extend_from_slice(message.as_bytes());
    commit_data.extend_from_slice(b"\n");
    commit_data.extend_from_slice(Utc::now().to_rfc3339().as_bytes());
    
    let hash = compute_hash(&commit_data);
    let short_hash = &hash[..7];

    // Create snapshot
    let snapshot_path = storage.create_snapshot(&hash)?;

    // Create commit object
    let commit = Commit {
        hash: hash.clone(),
        parent: parent_hash,
        timestamp: Utc::now(),
        author: author.clone(),
        message: message.to_string(),
        context_summary: summary.clone(),
        snapshot_path: relative_path(&vault_root, &snapshot_path),
    };

    // Append to commit log
    storage.append_commit(&current_branch, &commit)?;

    // Update branch reference
    storage.write_branch_ref(&current_branch, &hash)?;

    // Print summary
    println!(
        "{} {}",
        format!("[{} {}]", current_branch, short_hash).green(),
        message
    );
    println!("  Author: {}", author.display());
    
    let changes = summary.files_added.len() + summary.files_modified.len() + summary.files_removed.len();
    println!(
        "  {} files changed, {} insertions, {} deletions",
        changes,
        summary.files_added.len() + summary.files_modified.len(),
        summary.files_removed.len()
    );
    println!(
        "  Context: {} domains, ~{} tokens",
        summary.domains_loaded.len(),
        summary.token_estimate
    );

    Ok(())
}
