use crate::errors::*;
use crate::storage::Storage;
use crate::utils::*;
use colored::Colorize;

pub fn branch_list() -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    let current_branch = storage.read_head()?;
    let branches = storage.list_branches()?;

    if branches.is_empty() {
        println!("{}", "No branches found".dimmed());
        return Ok(());
    }

    for branch in branches {
        let is_current = branch == current_branch;
        let marker = if is_current { "*" } else { " " };
        
        // Get branch head commit
        if let Some(commit_hash) = storage.read_branch_ref(&branch)? {
            if let Some(commit) = storage.find_commit(&commit_hash)? {
                let short_hash = &commit.hash[..7];
                let branch_display = if is_current {
                    branch.green().to_string()
                } else {
                    branch
                };
                
                println!(
                    "{} {:<20} {} \"{}\"",
                    marker.green(),
                    branch_display,
                    short_hash.yellow(),
                    commit.message
                );
            } else {
                println!("{} {}", marker.green(), branch);
            }
        } else {
            println!("{} {} (no commits)", marker.green(), branch);
        }
    }

    Ok(())
}

pub fn branch_create(name: &str, description: Option<String>) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    // Check if branch already exists
    if storage.read_branch_ref(name)?.is_some() {
        return Err(GitnuError::BranchExists(name.to_string()));
    }

    // Get current HEAD commit
    let head_commit = storage.get_head_commit()?;
    let head_hash = match head_commit {
        Some(commit) => commit.hash,
        None => {
            return Err(GitnuError::Other(
                "Cannot create branch: no commits yet".to_string(),
            ));
        }
    };

    // Create branch pointing to current HEAD
    storage.write_branch_ref(name, &head_hash)?;

    println!("{} branch '{}'", "Created".green(), name.green());
    if let Some(desc) = description {
        println!("  Description: {}", desc.dimmed());
    }
    println!("  Starting at: {}", &head_hash[..7].yellow());

    Ok(())
}

pub fn branch_delete(name: &str) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    // Check if branch exists
    if storage.read_branch_ref(name)?.is_none() {
        return Err(GitnuError::BranchNotFound(name.to_string()));
    }

    // Check if it's the current branch
    let current_branch = storage.read_head()?;
    if current_branch == name {
        return Err(GitnuError::Other(format!(
            "Cannot delete current branch '{}'. Switch to another branch first.",
            name
        )));
    }

    storage.delete_branch(name)?;
    println!("{} branch '{}'", "Deleted".red(), name);

    Ok(())
}
