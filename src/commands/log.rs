use crate::errors::*;
use crate::storage::Storage;
use crate::utils::*;
use colored::Colorize;

pub fn log(oneline: bool, limit: Option<usize>, branch: Option<String>) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root);

    // Get branch to query
    let branch_name = match branch {
        Some(b) => b,
        None => storage.read_head()?,
    };

    // Get commits
    let mut commits = storage.read_commits(&branch_name)?;
    commits.reverse(); // Show newest first

    if commits.is_empty() {
        println!("{}", "No commits yet".dimmed());
        return Ok(());
    }

    // Apply limit
    let commits_to_show = if let Some(lim) = limit {
        &commits[..commits.len().min(lim)]
    } else {
        &commits
    };

    // Get current HEAD to mark it
    let head_commit = storage.get_head_commit()?;
    let head_hash = head_commit.as_ref().map(|c| c.hash.as_str());

    for commit in commits_to_show {
        let short_hash = &commit.hash[..7];
        
        if oneline {
            // One-line format
            let head_marker = if Some(commit.hash.as_str()) == head_hash {
                format!(" (HEAD -> {})", branch_name).yellow().to_string()
            } else {
                String::new()
            };
            
            println!(
                "{}{} {}",
                short_hash.yellow(),
                head_marker,
                commit.message
            );
        } else {
            // Full format
            let head_marker = if Some(commit.hash.as_str()) == head_hash {
                format!(" (HEAD -> {})", branch_name).yellow().to_string()
            } else {
                String::new()
            };
            
            println!("{} {}{}", "commit".yellow(), short_hash.yellow(), head_marker);
            println!("{} {}", "Author:".bold(), commit.author.display());
            println!(
                "{}   {}",
                "Date:".bold(),
                commit.timestamp.format("%a %b %d %H:%M:%S %Y")
            );
            println!();
            println!("    {}", commit.message);
            println!();
            println!(
                "    Context: {} domains loaded, ~{} tokens",
                commit.context_summary.domains_loaded.len(),
                commit.context_summary.token_estimate
            );
            
            if !commit.context_summary.files_modified.is_empty() {
                print!("    Modified: ");
                for (i, file) in commit.context_summary.files_modified.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}", file.file_name().unwrap_or_default().to_string_lossy());
                }
                println!();
            }
            
            if !commit.context_summary.files_added.is_empty() {
                print!("    Added: ");
                for (i, file) in commit.context_summary.files_added.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}", file.file_name().unwrap_or_default().to_string_lossy());
                }
                println!();
            }
            
            println!();
        }
    }

    Ok(())
}
