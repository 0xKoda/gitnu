use crate::errors::*;
use crate::storage::Storage;
use crate::utils::*;
use colored::Colorize;

pub fn rewind(target: &str, soft: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    // Find target commit
    let commit = storage.find_commit(target)?;
    let commit = match commit {
        Some(c) => c,
        None => return Err(GitnuError::CommitNotFound(target.to_string())),
    };

    // Get current branch
    let current_branch = storage.read_head()?;

    // Update branch ref to target commit
    storage.write_branch_ref(&current_branch, &commit.hash)?;

    if !soft {
        // Restore snapshot
        storage.restore_snapshot(&commit.hash)?;
        println!(
            "{} {} to commit {} \"{}\"",
            "Rewound".yellow(),
            current_branch.green(),
            commit.hash[..7].yellow(),
            commit.message
        );
        println!("  Restored context from {}", commit.hash[..7].yellow());
    } else {
        println!(
            "{} {} to commit {} \"{}\"",
            "Rewound".yellow(),
            current_branch.green(),
            commit.hash[..7].yellow(),
            commit.message
        );
        println!("  Working directory unchanged (--soft)");
    }

    println!(
        "{}  This is a destructive operation. Original commits still exist in .gitnu/objects/",
        "⚠️".yellow()
    );

    Ok(())
}
