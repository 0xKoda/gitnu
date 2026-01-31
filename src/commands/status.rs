use crate::errors::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use colored::Colorize;

pub fn status() -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(Storage::new(vault_root.clone()));

    // Get current branch
    let current_branch = storage.read_head()?;
    println!("{} {}", "On branch:".bold(), current_branch.green());

    // Get last commit
    if let Some(commit) = storage.get_head_commit()? {
        let short_hash = &commit.hash[..7];
        let time_ago = relative_time(&commit.timestamp);
        println!(
            "{} {} \"{}\" ({})",
            "Last commit:".bold(),
            short_hash.yellow(),
            commit.message,
            time_ago.dimmed()
        );
    } else {
        println!("{}", "No commits yet".dimmed());
    }

    println!();

    // Show active context
    let index = storage.load_index()?;
    let all_files = context_mgr.get_all_files()?;
    
    if !all_files.is_empty() {
        let summary = context_mgr.calculate_context_summary(storage.get_head_commit()?.as_ref())?;
        println!(
            "{} (estimated {} tokens):",
            "Active Context".bold(),
            summary.token_estimate.to_string().cyan()
        );

        // Show loaded files
        if !index.loaded.is_empty() || !all_files.is_empty() {
            println!("  {}:", "Loaded".green());
            let files_to_show: Vec<_> = if !index.loaded.is_empty() {
                index.loaded.iter().collect()
            } else {
                all_files.iter().collect()
            };
            
            for file in files_to_show.iter().take(10) {
                println!("    - {}", file.display().to_string().dimmed());
            }
            if files_to_show.len() > 10 {
                println!("    {} ({} more files)", "...".dimmed(), files_to_show.len() - 10);
            }
        }

        // Show pinned files
        if !index.pinned.is_empty() {
            println!("  {}:", "Pinned".blue());
            for file in &index.pinned {
                println!("    - {}", file.display().to_string().dimmed());
            }
        }

        println!();
    }

    // Show staged files
    if !index.staged.is_empty() {
        println!("{}", "Staged (ready to include):".bold());
        for staged in &index.staged {
            println!(
                "    - {} [{}] \"{}\"",
                staged.path.display().to_string().dimmed(),
                staged.priority.display().yellow(),
                staged.reason.dimmed()
            );
        }
        println!();
    }

    // Show modified files
    let modified = context_mgr.get_modified_files()?;
    if !modified.is_empty() {
        println!("{}", "Modified since last commit:".bold());
        for file in &modified {
            // Get line count change
            let full_path = vault_root.join(file);
            if full_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    let lines = content.lines().count();
                    println!("    - {} (+{} lines)", file.display().to_string().dimmed(), lines);
                }
            }
        }
        println!();
    }

    // Show untracked domains
    let domains_dir = storage.domains_dir();
    if domains_dir.exists() {
        let mut untracked_domains = Vec::new();
        for entry in std::fs::read_dir(&domains_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let domain_name = entry.file_name().to_string_lossy().to_string();
                
                // Check if this domain has any tracked files
                let has_files = walkdir::WalkDir::new(entry.path())
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_type().is_file());
                
                if has_files && !domain_name.starts_with('_') {
                    // Check if in index
                    let domain_tracked = all_files.iter().any(|f| {
                        f.to_str()
                            .map(|s| s.starts_with(&format!("domains/{}/", domain_name)))
                            .unwrap_or(false)
                    });
                    
                    if !domain_tracked {
                        untracked_domains.push(domain_name);
                    }
                }
            }
        }

        if !untracked_domains.is_empty() {
            println!("{}", "Untracked domains:".bold());
            for domain in untracked_domains {
                let count = walkdir::WalkDir::new(domains_dir.join(&domain))
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .count();
                println!("    - domains/{} ({} files)", domain.dimmed(), count);
            }
        }
    }

    Ok(())
}
