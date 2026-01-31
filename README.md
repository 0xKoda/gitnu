# gitnu - Version-Controlled Knowledge Operating System for AI Agents

`gitnu` (binary: `gnu`) is a Rust CLI tool that provides git-like version control for **knowledge and context**, not code. It's designed to work alongside markdown-based note-taking applications (like Obsidian) and AI coding agents (like Claude Code), enabling persistent memory across AI sessions.

## Features

- **Persistent Memory**: Checkpoint context states with commits
- **Branching/Exploration**: Safely explore alternative approaches without polluting main context
- **Structured Knowledge**: Organize knowledge in a domain/topic hierarchy
- **Dual Authorship**: Track whether changes came from humans or AI agents
- **Claude Code Integration**: Seamless integration via the Skills system

## Installation

### From Source

```bash
cargo install --path .
```

The binary will be installed as `gnu`.

### Usage

Initialize a new vault:

```bash
gnu init --name my-project
```

Check current status:

```bash
gnu status
```

Create a checkpoint:

```bash
gnu commit "Completed authentication design"
```

Create and switch branches:

```bash
gnu branch explore-graphql --describe "Exploring GraphQL approach"
gnu checkout explore-graphql
```

View commit history:

```bash
gnu log --oneline
```

Merge learnings back:

```bash
gnu checkout main
gnu merge explore-graphql
```

## Commands

- `gnu init` - Initialize a new gitnu vault
- `gnu status` - Show current context state
- `gnu commit <message>` - Create a checkpoint
- `gnu log` - Show commit history
- `gnu branch` - List, create, or delete branches
- `gnu checkout <target>` - Switch branches or restore commits
- `gnu rewind <commit>` - Roll back to a previous commit
- `gnu diff [source] [target]` - Show changes between commits/branches
- `gnu merge <source>` - Merge learnings from one branch into another
- `gnu load <path>` - Load domains/files into active context
- `gnu unload <path>` - Remove from active context
- `gnu pin <path>` - Mark files to always include
- `gnu resolve <wikilink>` - Resolve wikilinks to full paths
- `gnu context` - Output current context as single document
- `gnu summary` - Generate summary of current state

## Directory Structure

```
vault/
├── .gitnu/              # Version control metadata
│   ├── config.toml      # Configuration
│   ├── HEAD             # Current branch reference
│   ├── refs/heads/      # Branch pointers
│   ├── objects/         # Compressed snapshots
│   ├── commits/         # Commit logs
│   └── index.json       # Staging/relevance queue
├── .claude/             # Claude Code integration
│   └── skills/gitnu/
│       └── SKILL.md     # Auto-generated skill
└── domains/             # Knowledge organized by domain
    ├── _global/
    │   ├── agent.md     # Agent configuration
    │   └── conventions.md
    └── <project>/
        ├── spec.md      # Project specification
        ├── prd.md       # Product requirements
        ├── decisions.md # Architecture decisions
        ├── learnings.md # Agent discoveries
        └── todos.md     # Task tracking
```

## Claude Code Integration

When you run `gnu init`, it automatically creates a `.claude/skills/gitnu/SKILL.md` file that teaches Claude Code how to use gitnu for context management.

The skill enables Claude to:
- Load relevant context when starting a session
- Checkpoint progress after completing milestones
- Explore alternative approaches via branches
- Record important learnings and decisions
- Roll back when going down wrong paths

## Philosophy

Git tracks **what changed in code**. gitnu tracks **what the AI knows and has learned**, allowing rollback to previous cognitive states.

This enables:
- **Checkpointing**: Save context state before major decisions
- **Safe Exploration**: Try alternatives without losing main context
- **Knowledge Persistence**: Remember learnings across sessions
- **Behavioral Reset**: Roll back when agent behavior degrades
- **Context Management**: Control what's in active memory

## License

MIT

## Author

Built for AI agents and the humans who work with them.
