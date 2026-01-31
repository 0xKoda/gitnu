use crate::errors::*;
use crate::models::*;
use crate::storage::Storage;
use crate::utils::*;
use std::fs;
use std::path::Path;
use colored::Colorize;
use chrono::Utc;

const SKILL_TEMPLATE: &str = r#"---
name: gitnu
description: |
  Version-controlled context management for persistent AI memory.
  Use this skill to checkpoint progress, explore alternatives via branches,
  and maintain structured knowledge across sessions.
  
  Automatically invoked when:
  - Starting work on a project (load relevant context)
  - Completing a significant milestone (commit)
  - Exploring alternative approaches (branch)
  - Learning something important (update learnings.md, then commit)

allowed-tools:
  - Bash(gnu *)
  - Read
  - Write

context: auto
---

# gitnu: Your Memory System

You have access to a git-like context management system via the `gnu` command.
This system persists your knowledge and learnings across sessions.

## Orientation (run at session start)

When starting a new session or task:

```bash
gnu summary
gnu status
```

This tells you:
- What branch you're on and recent commits
- What context is currently loaded
- What has changed since last commit

## Key Commands

### Checkpointing Progress

After completing a significant milestone:

```bash
gnu commit "description of what was accomplished"
```

Write clear, descriptive messages. These help you (and humans) understand the progression.

### Loading Context

To load specific knowledge:

```bash
gnu load domains/<domain>/
gnu load [[filename]]  # Wikilink syntax
```

### Safe Exploration

When exploring an alternative approach:

```bash
gnu branch explore-<approach-name>
gnu checkout explore-<approach-name>
# ... do your exploration ...
gnu commit "findings from exploration"
gnu checkout main
gnu merge explore-<approach-name>  # if approach was good
# OR just leave it if approach was abandoned
```

### Recording Learnings

When you discover something important:

1. Update the relevant learnings.md file with your findings

2. Commit the learning:
```bash
gnu commit "Learned: <brief description>"
```

### Rolling Back

If you've gone down the wrong path:

```bash
gnu log --oneline  # Find the good commit
gnu rewind <commit-hash>
```

## File Ownership Convention

- **You write**: learnings.md, decisions.md, patterns.md
- **Human writes**: spec.md, prd.md, requirements.md
- **Both can write**: todos.md, topic files

Always update and commit learnings when you discover important patterns.

## Wikilink Resolution

When you see [[filename]] in documents, resolve with:

```bash
gnu resolve "[[filename]]"
```

This returns the full path to the file.
"#;

const AGENT_MD_TEMPLATE: &str = r#"# Agent Configuration

## Identity
You are an AI development assistant working in this vault.

## Capabilities
- Read and write files in the domains/ directory
- Use gnu commands for context management
- Follow the conventions in [[conventions]]

## Behavior Guidelines
1. Always run `gnu summary` when starting a new session
2. Commit after completing significant milestones
3. Use branches when exploring alternative approaches
4. Update learnings.md when discovering important patterns
5. Respect file ownership: don't modify spec.md or prd.md without asking

## Current Focus
[To be updated based on active work]
"#;

const CONVENTIONS_MD_TEMPLATE: &str = r#"# Cross-Project Conventions

## File Naming
- Use lowercase with hyphens: `my-feature.md`
- Use descriptive names that indicate content

## Markdown Style
- Use `##` for main sections
- Use `###` for subsections
- Include a blank line before and after headings

## Wikilinks
- Use `[[filename]]` for same-domain references
- Use `[[domain/filename]]` for cross-domain references

## Commit Messages
- Start with a verb: "Add", "Update", "Fix", "Refactor"
- Be concise but descriptive
- Reference key files or decisions when relevant
"#;

const SPEC_MD_TEMPLATE: &str = r#"# Project Specification

## Overview
[Describe what the project does]

## Goals
- [ ] Goal 1
- [ ] Goal 2

## Non-Goals
- Not doing X
- Not doing Y

## Technical Requirements
[Technical constraints and requirements]

## User Stories
As a [user type], I want to [action] so that [benefit].
"#;

const LEARNINGS_MD_TEMPLATE: &str = r#"# Learnings

This file is maintained by the AI agent to record important discoveries,
patterns, and insights gained during development.

---

[Learnings will be appended here]
"#;

const DECISIONS_MD_TEMPLATE: &str = r#"# Architecture Decision Records

This file tracks key technical decisions and their rationale.

---

[Decisions will be appended here]
"#;

const TODOS_MD_TEMPLATE: &str = r#"# Tasks and TODOs

## In Progress
- [ ] 

## Planned
- [ ] 

## Completed
- [x] Initial project setup
"#;

pub fn init(name: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let vault_name = name.unwrap_or_else(|| {
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string()
    });

    // Check if already initialized
    if vault_exists(&current_dir) {
        return Err(GitnuError::AlreadyInitialized(current_dir));
    }

    let storage = Storage::new(current_dir.clone());
    
    // Initialize vault structure
    storage.init(&vault_name)?;

    // Create domains/_global directory
    let global_dir = current_dir.join("domains/_global");
    ensure_dir(&global_dir)?;
    
    // Create agent.md
    let agent_md = global_dir.join("agent.md");
    fs::write(&agent_md, AGENT_MD_TEMPLATE)?;
    
    // Create conventions.md
    let conventions_md = global_dir.join("conventions.md");
    fs::write(&conventions_md, CONVENTIONS_MD_TEMPLATE)?;

    // Create skills directory
    let skills_dir = global_dir.join("skills");
    ensure_dir(&skills_dir)?;

    // Create project domain if name provided
    if vault_name != "unnamed" {
        create_project_domain(&current_dir, &vault_name)?;
    }

    // Create .claude/skills/gitnu/ directory and SKILL.md
    create_claude_skill(&current_dir)?;

    // Create initial commit
    create_initial_commit(&storage, &vault_name)?;

    // Print success message
    println!("{}", "Initialized gitnu vault".green().bold());
    println!("  Vault root: {}", current_dir.display());
    println!("  Vault name: {}", vault_name);
    println!();
    println!("{}", "Created:".bold());
    println!("  {}", ".gitnu/".dimmed());
    println!("  {}", "domains/_global/agent.md".dimmed());
    println!("  {}", "domains/_global/conventions.md".dimmed());
    if vault_name != "unnamed" {
        println!("  {}", format!("domains/{}/", vault_name).dimmed());
    }
    println!("  {}", ".claude/skills/gitnu/SKILL.md".dimmed());
    println!();
    println!("{} Run 'gnu status' to see current state.", "✓".green().bold());

    Ok(())
}

fn create_project_domain(vault_root: &Path, project_name: &str) -> Result<()> {
    let project_dir = vault_root.join("domains").join(project_name);
    ensure_dir(&project_dir)?;

    // Create template files
    fs::write(project_dir.join("spec.md"), SPEC_MD_TEMPLATE)?;
    fs::write(project_dir.join("learnings.md"), LEARNINGS_MD_TEMPLATE)?;
    fs::write(project_dir.join("decisions.md"), DECISIONS_MD_TEMPLATE)?;
    fs::write(project_dir.join("todos.md"), TODOS_MD_TEMPLATE)?;

    Ok(())
}

fn create_claude_skill(vault_root: &Path) -> Result<()> {
    let skill_dir = vault_root.join(".claude/skills/gitnu");
    ensure_dir(&skill_dir)?;

    let skill_md = skill_dir.join("SKILL.md");
    fs::write(skill_md, SKILL_TEMPLATE)?;

    Ok(())
}

fn create_initial_commit(storage: &Storage, vault_name: &str) -> Result<()> {
    use crate::context::ContextManager;
    
    let context_mgr = ContextManager::new(Storage::new(storage.vault_root.clone()));
    let summary = context_mgr.calculate_context_summary(None)?;

    // Create commit object
    let mut commit_data = Vec::new();
    commit_data.extend_from_slice(b"tree ");
    commit_data.extend_from_slice(vault_name.as_bytes());
    commit_data.extend_from_slice(b"\n");
    commit_data.extend_from_slice(Utc::now().to_rfc3339().as_bytes());
    
    let hash = compute_hash(&commit_data);
    let short_hash = &hash[..7];

    // Create snapshot
    let snapshot_path = storage.create_snapshot(&hash)?;

    let commit = Commit {
        hash: hash.clone(),
        parent: None,
        timestamp: Utc::now(),
        author: Author::Human {
            name: "user".to_string(),
        },
        message: "Initial commit".to_string(),
        context_summary: summary,
        snapshot_path: relative_path(&storage.vault_root, &snapshot_path),
    };

    // Write to commit log
    storage.append_commit("main", &commit)?;
    
    // Update main branch ref
    storage.write_branch_ref("main", &hash)?;

    println!("{}", format!("[main {}] Initial commit", short_hash).dimmed());

    Ok(())
}
