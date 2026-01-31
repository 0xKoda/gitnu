# gitnu Quick Start Guide

Get up and running with gitnu in 60 seconds.

## Installation

```bash
# Clone or navigate to the gitnu directory
cd /path/to/gitnu

# Build and install
cargo install --path .

# Verify installation
gnu --version
```

## Your First Vault

```bash
# Create a new directory for your project
mkdir my-project
cd my-project

# Initialize gitnu
gnu init --name my-project
```

You'll see:
```
Initialized gitnu vault
Created: .gitnu/
Created: domains/_global/agent.md
Created: .claude/skills/gitnu/SKILL.md
✓ Ready. Run 'gnu status' to see current state.
```

## Basic Workflow

### 1. Check What's There

```bash
gnu status
```

Shows your current branch, last commit, and active context.

### 2. Edit Your Knowledge

Open files in `domains/my-project/`:
- `spec.md` - Your project specification
- `decisions.md` - Architecture decisions (AI usually writes this)
- `learnings.md` - Patterns and discoveries (AI writes this)
- `todos.md` - Task tracking (both edit this)

Edit them with any text editor or Obsidian.

### 3. Checkpoint Your Progress

```bash
gnu commit "Added user authentication design" --author human
```

For AI agents:
```bash
gnu commit "Learned JWT best practices" --author agent --model "claude-3-5-sonnet"
```

### 4. View History

```bash
gnu log --oneline
```

### 5. Try Alternative Approaches

```bash
# Create exploration branch
gnu branch explore-graphql --describe "Testing GraphQL instead of REST"

# Switch to it
gnu checkout explore-graphql

# Make changes, then commit
gnu commit "Explored GraphQL approach"

# Compare with main
gnu diff main explore-graphql

# Switch back
gnu checkout main

# Merge if good
gnu merge explore-graphql
```

### 6. Roll Back If Needed

```bash
# See history
gnu log --oneline

# Rewind to earlier commit
gnu rewind abc1234
```

## Using Wikilinks

Reference files with `[[filename]]` syntax:

```bash
# Resolve to full path
gnu resolve "[[spec]]"

# Load into active context
gnu load "[[spec]]"

# Pin to always include
gnu pin "[[conventions]]"
```

## Get Context Overview

```bash
# Quick summary
gnu summary

# Full context export
gnu context > mycontext.txt
```

## Tips

1. **Commit often** - After each significant change or decision
2. **Use branches** - For any "what if" exploration
3. **Pin essential files** - Keep core context always loaded
4. **Check status** - Before and after major changes
5. **Use descriptive messages** - Future you will thank you

## With Claude Code

If using Claude Code in a gitnu vault:

1. Claude automatically reads `.claude/skills/gitnu/SKILL.md`
2. It knows to run `gnu summary` when starting
3. It commits after completing tasks
4. It uses branches for exploration
5. It records learnings automatically

You don't need to teach Claude anything - the skill file does it!

## Common Commands Reference

| Command | Purpose |
|---------|---------|
| `gnu init` | Create new vault |
| `gnu status` | Show current state |
| `gnu commit "msg"` | Checkpoint |
| `gnu log` | View history |
| `gnu branch` | List branches |
| `gnu branch name` | Create branch |
| `gnu checkout name` | Switch branch |
| `gnu diff` | Show changes |
| `gnu merge src` | Merge branch |
| `gnu resolve "[[link]]"` | Find file |
| `gnu summary` | Overview |
| `gnu rewind hash` | Roll back |

## Help

For any command:
```bash
gnu help
gnu <command> --help
```

## Next Steps

- Read [DEMO.md](DEMO.md) for detailed walkthrough
- See [README.md](README.md) for full documentation
- Check [IMPLEMENTATION.md](IMPLEMENTATION.md) for technical details

Start experimenting! gitnu is designed to be safe - you can always roll back.
