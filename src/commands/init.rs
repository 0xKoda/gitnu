use crate::errors::*;
use crate::models::*;
use crate::storage::Storage;
use crate::utils::*;
use std::fs;
use std::path::Path;
use colored::Colorize;
use chrono::Utc;

// Leaner skill template focusing on quick reference
const SKILL_TEMPLATE: &str = r#"---
name: gitnu
description: |
  Version-controlled context management for persistent AI memory.
  Load with `gnu summary` at session start. Commit milestones with `gnu commit`.
  See references/ for detailed documentation.

allowed-tools:
  - Bash(gnu *)
  - Read
  - Write

context: auto
---

# gitnu: Quick Reference

## Essential Commands

```bash
# Session start (always run first)
gnu summary              # Load context and recent history
gnu status              # See what's changed

# Save progress
gnu commit "message"    # Checkpoint your work

# Load context
gnu load domains/<domain>/
gnu load [[filename]]

# Branch for exploration
gnu branch feature-name
gnu checkout feature-name

# View history
gnu log --oneline
gnu diff
```

## When to Commit

Commit after:
- Completing a feature or milestone
- Making an important decision (update decisions.md first)
- Learning something valuable (update learnings.md first)
- Before exploring an alternative approach

## File Ownership

- **You write**: learnings.md, decisions.md, patterns.md
- **Human writes**: spec.md, prd.md, requirements.md
- **Both write**: todos.md, topic files

## Need More Details?

See `references/` folder for:
- Complete command reference with all flags
- Step-by-step workflows
- Real-world examples
- Troubleshooting guide
"#;

// AGENTS.md template - cross-platform system prompt
const AGENTS_MD_TEMPLATE: &str = r#"# gitnu Project Configuration

## Required Initialization

**CRITICAL**: At the start of EVERY session, you MUST execute:

```bash
gnu summary
gnu status
```

This loads the project context and shows what has changed since your last commit.

## Continuous Monitoring

After ANY significant work (completing a feature, fixing a bug, making a decision), you MUST:

1. **Check if learnings should be captured**: Did you discover a pattern, gotcha, or best practice? Update `domains/<domain>/learnings.md`

2. **Check if decisions should be documented**: Did you make an architectural or technical choice? Update `domains/<domain>/decisions.md` with the decision and rationale

3. **Commit your progress**: Run `gnu commit "clear description of what was accomplished"`

## Decision Documentation Protocol

When making ANY architectural, technical, or design decision:

1. Document it in `domains/<domain>/decisions.md` following this format:
   ```markdown
   ## [Date] Decision: [Brief Title]
   
   **Context**: What problem are we solving?
   
   **Decision**: What did we decide to do?
   
   **Rationale**: Why did we choose this approach?
   
   **Alternatives Considered**: What other options did we evaluate?
   
   **Consequences**: What are the implications of this decision?
   ```

2. Commit with message format: `"Decision: <brief summary>"`

## File Ownership Conventions

Respect these ownership rules:

- **You (AI) write**: learnings.md, decisions.md, patterns.md
- **Human writes**: spec.md, prd.md, requirements.md
- **Both can write**: todos.md, topic-specific files

Never modify spec.md or prd.md without explicit human approval.

## Branching for Exploration

When exploring alternative approaches or risky changes:

```bash
gnu branch explore-<approach-name>
gnu checkout explore-<approach-name>
# ... do exploration ...
gnu commit "findings from <approach>"
gnu checkout main
# Merge if successful, or leave branch if abandoned
```

## Using the gitnu Skill

For detailed command reference and workflows, use the gitnu skill:
- Type `/gitnu` to explicitly load it
- Or mention "gitnu" in your context for automatic loading
- See `.claude/skills/gitnu/references/` for progressive documentation

## Project Context

This is a gitnu-managed project. All domain knowledge lives in `domains/`, and you should commit progress regularly to maintain context across sessions.
"#;

// Claude Code config template
const CLAUDE_CONFIG_TEMPLATE: &str = r#"{
  "skills": {
    "enabled": ["gitnu"],
    "directories": [".claude/skills"]
  }
}
"#;

// OpenCode config template
const OPENCODE_CONFIG_TEMPLATE: &str = r#"{
  "rules": [
    "AGENTS.md"
  ],
  "permission": {
    "skill": {
      "*": "allow",
      "gitnu": "allow"
    }
  }
}
"#;

// Reference documentation templates
const COMMANDS_REFERENCE: &str = r##"# Complete Command Reference

## Core Commands

### gnu summary
Shows recent commit history and current context state.

```bash
gnu summary [--lines N]  # Show last N commits (default: 5)
```

**When to use**: At the start of every session to load context.

### gnu status
Shows what has changed in domains/ since last commit.

```bash
gnu status
```

**Output**: Lists modified, added, and deleted files.

### gnu commit
Creates a checkpoint of current state.

```bash
gnu commit "message"
gnu commit -m "message"   # Short form
```

**Best practices**:
- Use clear, descriptive messages
- Start with a verb: "Add", "Update", "Fix", "Refactor", "Document"
- Reference key files or decisions
- Commit after significant milestones

### gnu log
View commit history.

```bash
gnu log                  # Full history
gnu log --oneline        # Compact one-line format
gnu log --graph          # Show branch structure
gnu log -n 10            # Limit to 10 commits
```

### gnu diff
See what has changed.

```bash
gnu diff                 # Show all changes
gnu diff <file>          # Changes in specific file
gnu diff <commit>        # Changes since commit
```

## Branching Commands

### gnu branch
List or create branches.

```bash
gnu branch                      # List all branches
gnu branch <name>               # Create new branch
gnu branch -d <name>            # Delete branch
```

**Naming conventions**:
- `feature-<name>` - New features
- `fix-<name>` - Bug fixes
- `explore-<name>` - Experimental approaches
- `refactor-<name>` - Code refactoring

### gnu checkout
Switch between branches.

```bash
gnu checkout <branch>           # Switch to branch
gnu checkout -b <branch>        # Create and switch
gnu checkout main               # Return to main
```

### gnu merge
Merge changes from another branch.

```bash
gnu merge <branch>              # Merge branch into current
```

**Note**: Conflicts must be resolved manually.

## Context Loading

### gnu load
Load specific context into your working knowledge.

```bash
gnu load domains/<domain>/      # Load entire domain
gnu load [[filename]]           # Load by wikilink
gnu load <path>                 # Load specific file
```

**When to use**: When you need to work on a specific area of the project.

### gnu resolve
Resolve wikilinks to full paths.

```bash
gnu resolve "[[filename]]"
```

**Returns**: Full path to the referenced file.

## History Navigation

### gnu rewind
Go back to a previous state.

```bash
gnu rewind <commit-hash>        # Reset to specific commit
```

**WARNING**: This discards uncommitted changes. Commit or branch first!

## Advanced Commands

### gnu snapshot
Manually create a snapshot (commits do this automatically).

```bash
gnu snapshot
```

### gnu gc
Clean up old snapshots to save disk space.

```bash
gnu gc --older-than 30d        # Remove snapshots older than 30 days
```
"##;

const WORKFLOWS_REFERENCE: &str = r##"# Common Workflows

## Daily Workflow

### Starting Your Day

1. **Load context**:
   ```bash
   gnu summary
   gnu status
   ```

2. **Review what you were working on**: The summary shows recent commits and current branch

3. **Check for uncommitted changes**: Status shows any pending work

4. **Continue or commit**: If there are changes, decide to commit or continue

### During Development

1. **Make changes** to files in domains/

2. **Update learnings** as you discover patterns:
   ```bash
   # Edit domains/<domain>/learnings.md
   gnu commit "Learned: <insight>"
   ```

3. **Document decisions** when making choices:
   ```bash
   # Edit domains/<domain>/decisions.md
   gnu commit "Decision: <choice made>"
   ```

4. **Regular checkpoints** after completing tasks:
   ```bash
   gnu commit "Completed <feature/fix/task>"
   ```

### Ending Your Session

1. **Review changes**:
   ```bash
   gnu status
   gnu diff
   ```

2. **Commit any pending work**:
   ```bash
   gnu commit "WIP: <current task> - <what's left to do>"
   ```

3. **Optional**: Update todos.md with next steps

## Feature Development Workflow

### Starting a New Feature

1. **Create a feature branch**:
   ```bash
   gnu branch feature-<name>
   gnu checkout feature-<name>
   ```

2. **Update spec** (human does this, or you with approval):
   ```bash
   # Edit domains/<domain>/spec.md
   ```

3. **Create todos**:
   ```bash
   # Edit domains/<domain>/todos.md
   gnu commit "Planning: <feature name>"
   ```

### Developing the Feature

1. **Work in iterations**, committing frequently:
   ```bash
   # Make changes
   gnu commit "Implement <component>"
   
   # More changes
   gnu commit "Add <functionality>"
   ```

2. **Document as you go**:
   ```bash
   # Update learnings.md with discoveries
   # Update decisions.md with choices
   gnu commit "Document <aspect>"
   ```

3. **Keep todos updated** to track progress

### Completing the Feature

1. **Final review**:
   ```bash
   gnu log --oneline
   gnu diff main
   ```

2. **Merge to main**:
   ```bash
   gnu checkout main
   gnu merge feature-<name>
   ```

3. **Clean up**:
   ```bash
   gnu branch -d feature-<name>
   ```

## Exploration Workflow

### Trying an Alternative Approach

1. **Create exploration branch**:
   ```bash
   gnu branch explore-<approach>
   gnu checkout explore-<approach>
   ```

2. **Experiment freely**: Make changes, try things out

3. **Document findings**:
   ```bash
   # Add findings to learnings.md
   gnu commit "Exploration: <what was learned>"
   ```

4. **Decide**:
   - **If successful**: Merge to main
   - **If unsuccessful**: Just checkout main, leave branch

### Comparing Approaches

1. **Create multiple exploration branches**:
   ```bash
   gnu branch explore-approach-a
   gnu branch explore-approach-b
   ```

2. **Try each approach** in its own branch

3. **Compare results**:
   ```bash
   gnu checkout main
   gnu diff explore-approach-a
   gnu diff explore-approach-b
   ```

4. **Merge the winner** and document the decision

## Debugging Workflow

### When Something Breaks

1. **Check recent changes**:
   ```bash
   gnu status
   gnu diff
   ```

2. **Review recent commits**:
   ```bash
   gnu log --oneline -n 10
   ```

3. **Create a fix branch**:
   ```bash
   gnu branch fix-<issue>
   gnu checkout fix-<issue>
   ```

4. **Fix the issue** and test

5. **Document the fix**:
   ```bash
   # Update learnings.md with root cause
   gnu commit "Fix: <issue> - <solution>"
   ```

### Finding When a Bug Was Introduced

1. **View history**:
   ```bash
   gnu log --oneline
   ```

2. **Check specific commits**:
   ```bash
   gnu diff <commit-hash>
   ```

3. **If needed, rewind** to a known good state:
   ```bash
   gnu rewind <good-commit>
   ```

## Cross-Domain Work

### Working Across Multiple Domains

1. **Load multiple contexts**:
   ```bash
   gnu load domains/domain-a/
   gnu load domains/domain-b/
   ```

2. **Make changes** in both domains

3. **Commit with context**:
   ```bash
   gnu commit "Cross-domain: <what connects them>"
   ```

4. **Update learnings** in relevant domains about the connection

## Recovery Workflow

### Undoing Recent Changes

1. **If not yet committed**:
   ```bash
   # Just discard changes manually or:
   gnu rewind <last-good-commit>
   ```

2. **If already committed**:
   ```bash
   gnu log --oneline
   gnu rewind <commit-before-mistake>
   ```

### Recovering from Wrong Branch

1. **Check current branch**:
   ```bash
   gnu summary
   ```

2. **Switch to correct branch**:
   ```bash
   gnu checkout <correct-branch>
   ```

3. **If changes made on wrong branch**:
   ```bash
   # Commit them first
   gnu commit "WIP: on wrong branch"
   
   # Switch to correct branch
   gnu checkout <correct-branch>
   
   # Cherry-pick if needed (manual merge)
   ```
"##;

const EXAMPLES_REFERENCE: &str = r###"# Real-World Examples

## Example 1: Starting a New Project

```bash
# Initialize gitnu
gnu init --name "my-awesome-project"

# First session - see what was created
gnu summary
# Output: Shows initial commit

gnu status
# Output: Clean working directory

# Start working
# Edit domains/my-awesome-project/spec.md
# Edit domains/my-awesome-project/todos.md

# Save your planning
gnu commit "Initial project planning"

# Start developing
# Create files, write code, etc.

# Capture a learning
# Add to domains/my-awesome-project/learnings.md:
# "## Python Environment
# - Use venv for isolation
# - requirements.txt should pin versions"

gnu commit "Learned: Python environment best practices"
```

## Example 2: Feature Branch Development

```bash
# Day 1: Start feature
gnu summary  # Loads context from last session
gnu branch feature-user-auth
gnu checkout feature-user-auth

# Work on authentication
# Edit relevant files

gnu commit "Add user model and database schema"
gnu commit "Implement password hashing"
gnu commit "Add login endpoint"

# End of day
gnu status  # Check for uncommitted work

# Day 2: Continue feature
gnu summary  # See yesterday's progress
gnu commit "Add session management"
gnu commit "Implement logout"

# Feature complete - merge to main
gnu checkout main
gnu merge feature-user-auth

# Clean up
gnu branch -d feature-user-auth
```

## Example 3: Exploring Alternative Solutions

```bash
# Problem: Need to choose between two database approaches

# Current state
gnu commit "Current implementation using SQLite"

# Try approach A
gnu branch explore-postgres
gnu checkout explore-postgres
# Implement PostgreSQL version
gnu commit "Exploration: PostgreSQL implementation"

# Try approach B
gnu checkout main
gnu branch explore-mongodb  
gnu checkout explore-mongodb
# Implement MongoDB version
gnu commit "Exploration: MongoDB implementation"

# Compare approaches
gnu checkout main
gnu diff explore-postgres
gnu diff explore-mongodb

# Document decision
# Add to decisions.md:
# "## Database Choice
# Decided: PostgreSQL
# Rationale: Better support for complex queries, ACID compliance
# Alternatives: MongoDB (too schemaless), SQLite (limited concurrency)"

# Merge chosen approach
gnu merge explore-postgres
gnu commit "Decision: Use PostgreSQL for database"

# Keep exploration branches for reference
# (or delete if you prefer: gnu branch -d explore-mongodb)
```

## Example 4: Debugging Session

```bash
# Something broke - investigate
gnu summary  # What changed recently?

gnu diff  # See current changes

# Check last few commits
gnu log --oneline -n 5
# Output:
# a7f3c21 Refactor user service
# b2e4d18 Add caching layer  
# c9f1a32 Update dependencies
# d3e8b45 Fix validation bug

# Suspect the caching layer
gnu diff b2e4d18

# Problem found in cache implementation
# Fix the issue
# Edit the problematic file

# Document the bug
# Add to learnings.md:
# "## Cache Invalidation Bug
# - Cache wasn't being invalidated on updates
# - Fixed by adding invalidation hooks
# - Lesson: Always test cache invalidation paths"

gnu commit "Fix: Cache invalidation on updates"
```

## Example 5: Multi-Domain Project

```bash
# Working on a fullstack app with multiple domains

# Initialize with multiple domains
gnu init --name "webapp"

# Create additional domains manually
mkdir -p domains/frontend
mkdir -p domains/backend  
mkdir -p domains/infrastructure

# Create standard files in each
# (spec.md, learnings.md, decisions.md, todos.md)

# Working session - frontend focus
gnu load domains/frontend/
# Make frontend changes
gnu commit "Frontend: Add user dashboard"

# Switch to backend work
gnu load domains/backend/
# Make backend changes
gnu commit "Backend: Add analytics endpoint"

# Cross-domain work
gnu load domains/frontend/
gnu load domains/backend/
# Changes that touch both
gnu commit "Cross-domain: Connect dashboard to analytics API"

# Document the integration in both learnings.md files
```

## Example 6: Recovering from a Mistake

```bash
# Made changes that broke everything
gnu status
# Shows many modified files

# Check what changed
gnu diff

# Realize the changes are wrong
gnu log --oneline
# Output:
# Current state (uncommitted changes)
# e5f7a23 Working: Feature X
# f6g8b34 Add feature Y

# Rewind to before we broke things
gnu rewind e5f7a23

# Clean slate - start over
gnu status
# Output: Clean working directory

# Try again, more carefully this time
```

## Example 7: Managing Learnings

```bash
# Throughout development, capture insights

# Discovered a Python gotcha
# Edit domains/backend/learnings.md:
# "## Python Imports
# - Circular imports break at runtime, not import time
# - Solution: Use late imports or restructure modules
# - Spent 2 hours debugging this"

gnu commit "Learned: Python circular import gotchas"

# Found a useful pattern
# Edit domains/frontend/learnings.md:
# "## React Patterns  
# - Custom hooks for data fetching reduce duplication
# - Pattern: useResource(url) -> {data, loading, error}
# - Used in UserList, PostList, CommentList"

gnu commit "Learned: Custom hook pattern for data fetching"

# Later, when working on similar feature
gnu load [[learnings]]
# Loads the relevant learnings.md to refresh memory
```

## Example 8: Collaborative Workflow

```bash
# Human adds a spec
# File: domains/webapp/spec.md created by human

# You review it
gnu status  # Shows spec.md as new file

# Create implementation plan
# Edit domains/webapp/todos.md based on spec

gnu commit "Planning: Implementation todos from spec"

# Implement first part
# Make changes
gnu commit "Implement authentication (from spec)"

# Discover spec needs clarification
# Update todos.md with question for human
gnu commit "Question: Spec unclear on password reset flow"

# Human updates spec
# You continue implementation
gnu load [[spec]]  # Reload updated spec
gnu commit "Implement password reset (per updated spec)"
```
"###;

const TROUBLESHOOTING_REFERENCE: &str = r##"# Troubleshooting Guide

## Common Issues

### "Already initialized" error

**Problem**: Running `gnu init` in an already initialized directory.

**Solution**: 
```bash
# Check if already initialized
ls .gitnu

# If you want to start fresh
rm -rf .gitnu domains .claude
gnu init
```

### Changes not showing in status

**Problem**: Made changes but `gnu status` shows clean.

**Cause**: Changes are outside the `domains/` directory.

**Solution**: gitnu only tracks files in `domains/`. Move your files there:
```bash
mkdir -p domains/my-project
mv my-files/* domains/my-project/
```

### Can't find wikilink

**Problem**: `gnu resolve "[[filename]]"` fails to find file.

**Cause**: File doesn't exist or is named differently.

**Solution**:
```bash
# Search for the file
find domains -name "*filename*"

# Use the full path or correct name
gnu load domains/project/actual-filename.md
```

### Merge conflicts

**Problem**: `gnu merge` shows conflicts.

**Cause**: Same file was modified in both branches.

**Solution**:
1. Open the conflicting files
2. Manually resolve the conflicts
3. Commit the resolution:
   ```bash
   gnu commit "Merge feature-x - resolved conflicts"
   ```

### Lost uncommitted work

**Problem**: Used `gnu rewind` and lost uncommitted changes.

**Prevention**: Always commit or check status before rewinding:
```bash
gnu status
gnu commit "WIP: current state" 
gnu rewind <commit>
```

**Recovery**: If just rewound, look for snapshot files in `.gitnu/snapshots/`

### Branch confusion

**Problem**: Not sure what branch you're on.

**Solution**:
```bash
gnu summary  # Shows current branch at top
gnu branch   # Lists all branches, * marks current
```

### Too many old snapshots

**Problem**: `.gitnu/snapshots/` directory is huge.

**Solution**:
```bash
# Clean up old snapshots
gnu gc --older-than 30d

# Or manually
rm -rf .gitnu/snapshots/<old-hash>
```

### Can't checkout branch

**Problem**: `gnu checkout` fails with uncommitted changes.

**Cause**: Changes in working directory conflict with branch.

**Solution**:
```bash
# Commit current work
gnu commit "WIP: switching branches"

# Then checkout
gnu checkout other-branch
```

## Error Messages Explained

### "Vault not initialized"

You're trying to use gnu commands outside a gitnu vault.

Solution: `gnu init` or `cd` to an initialized directory.

### "No commits yet"

The vault is initialized but has no commits.

Solution: Make some changes and `gnu commit`.

### "Branch already exists"

Trying to create a branch that already exists.

Solution: Use a different name or checkout existing:
```bash
gnu checkout existing-branch
```

### "Cannot rewind: uncommitted changes"

Trying to rewind with unsaved work.

Solution: Commit or discard changes first.

## Performance Issues

### Slow operations

**Cause**: Very large domains/ directory.

**Solution**:
- Split into multiple domains
- Use .gitignore patterns (if implemented)
- Clean up generated files

### Status takes too long

**Cause**: Many files in domains/.

**Solution**:
- Reduce number of tracked files  
- Move large binary files elsewhere
- Commit more frequently (smaller diffs)

## Best Practices to Avoid Issues

1. **Always run `gnu summary` at session start**: Loads necessary context

2. **Commit frequently**: Small, focused commits are easier to manage

3. **Use descriptive branch names**: Makes navigation clearer

4. **Keep domains/ organized**: Use subdirectories for clarity

5. **Document as you go**: Update learnings.md and decisions.md immediately

6. **Check status before switching**: Avoids losing work

7. **Use branches for experiments**: Keep main clean and stable

## Getting Help

If you encounter an issue not covered here:

1. Check the command reference: `.claude/skills/gitnu/references/commands.md`

2. Review workflows: `.claude/skills/gitnu/references/workflows.md`

3. File an issue with:
   - What command you ran
   - Error message received
   - Output of `gnu status` and `gnu summary`
   - Your environment (OS, shell)
"##;

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

    // Create .claude/skills/gitnu/ with references
    create_claude_skill(&current_dir)?;

    // Create AGENTS.md (cross-platform system prompt)
    create_agents_md(&current_dir)?;

    // Create .claude/config.json
    create_claude_config(&current_dir)?;

    // Create opencode.json (optional but recommended)
    create_opencode_config(&current_dir)?;

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
    println!("  {}", ".claude/skills/gitnu/references/".dimmed());
    println!("  {}", "AGENTS.md".dimmed());
    println!("  {}", ".claude/config.json".dimmed());
    println!("  {}", "opencode.json".dimmed());
    println!();
    println!("{}", "Cross-platform compatible with:".bold());
    println!("  {} Claude Code, OpenCode, Cursor, VS Code, Zed", "✓".green());
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

    // Create references directory with comprehensive documentation
    let references_dir = skill_dir.join("references");
    ensure_dir(&references_dir)?;

    fs::write(references_dir.join("commands.md"), COMMANDS_REFERENCE)?;
    fs::write(references_dir.join("workflows.md"), WORKFLOWS_REFERENCE)?;
    fs::write(references_dir.join("examples.md"), EXAMPLES_REFERENCE)?;
    fs::write(references_dir.join("troubleshooting.md"), TROUBLESHOOTING_REFERENCE)?;

    Ok(())
}

fn create_agents_md(vault_root: &Path) -> Result<()> {
    let agents_md = vault_root.join("AGENTS.md");
    
    if agents_md.exists() {
        // File exists - check if gitnu config already present
        let existing = fs::read_to_string(&agents_md)?;
        
        if existing.contains("gitnu") || existing.contains("gnu summary") {
            println!("{} AGENTS.md already contains gitnu config, skipping...", "⚠".yellow());
            return Ok(());
        }
        
        // Append gitnu section with clear separator
        let gitnu_section = format!(
            "\n\n---\n\n{}\n\n_Added by `gnu init` on {}_\n",
            AGENTS_MD_TEMPLATE,
            chrono::Local::now().format("%Y-%m-%d")
        );
        
        fs::write(&agents_md, format!("{}{}", existing, gitnu_section))?;
        println!("{} Appended gitnu config to existing AGENTS.md", "✓".green());
        
    } else {
        // No file exists - create new one
        let header = format!(
            "{}\n\n_Created by `gnu init` on {}_\n",
            AGENTS_MD_TEMPLATE,
            chrono::Local::now().format("%Y-%m-%d")
        );
        fs::write(&agents_md, header)?;
        println!("{} Created AGENTS.md", "✓".green());
    }
    
    Ok(())
}

fn create_claude_config(vault_root: &Path) -> Result<()> {
    let config_dir = vault_root.join(".claude");
    let config_path = config_dir.join("config.json");
    
    ensure_dir(&config_dir)?;
    
    if config_path.exists() {
        // Parse existing config and merge
        let existing_content = fs::read_to_string(&config_path)?;
        
        match serde_json::from_str::<serde_json::Value>(&existing_content) {
            Ok(mut existing) => {
                // Merge gitnu skill into existing config
                if let Some(obj) = existing.as_object_mut() {
                    // Add or update skills section
                    let skills = obj.entry("skills").or_insert(serde_json::json!({
                        "enabled": [],
                        "directories": []
                    }));
                    
                    if let Some(skills_obj) = skills.as_object_mut() {
                        // Add gitnu to enabled list
                        if let Some(enabled) = skills_obj.get_mut("enabled") {
                            if let Some(arr) = enabled.as_array_mut() {
                                if !arr.iter().any(|v| v == "gitnu") {
                                    arr.push(serde_json::json!("gitnu"));
                                }
                            }
                        } else {
                            skills_obj.insert("enabled".to_string(), serde_json::json!(["gitnu"]));
                        }
                        
                        // Add .claude/skills to directories
                        if let Some(dirs) = skills_obj.get_mut("directories") {
                            if let Some(arr) = dirs.as_array_mut() {
                                if !arr.iter().any(|v| v == ".claude/skills") {
                                    arr.push(serde_json::json!(".claude/skills"));
                                }
                            }
                        } else {
                            skills_obj.insert("directories".to_string(), serde_json::json!([".claude/skills"]));
                        }
                    }
                }
                
                fs::write(&config_path, serde_json::to_string_pretty(&existing)?)?;
                println!("{} Updated .claude/config.json with gitnu skill", "✓".green());
            }
            Err(_) => {
                println!("{} Could not parse existing .claude/config.json - please add gitnu manually", "⚠".yellow());
            }
        }
    } else {
        // Create new config
        fs::write(&config_path, CLAUDE_CONFIG_TEMPLATE)?;
        println!("{} Created .claude/config.json", "✓".green());
    }
    
    Ok(())
}

fn create_opencode_config(vault_root: &Path) -> Result<()> {
    let config_path = vault_root.join("opencode.json");
    
    if config_path.exists() {
        println!("{} opencode.json already exists - you may need to manually add gitnu skill", "⚠".yellow());
        return Ok(());
    }
    
    fs::write(&config_path, OPENCODE_CONFIG_TEMPLATE)?;
    println!("{} Created opencode.json", "✓".green());
    
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
