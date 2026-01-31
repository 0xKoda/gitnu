# gitnu Demo - Complete Walkthrough

This document demonstrates all the features of gitnu with a practical example.

## Setup

```bash
# Initialize a new vault
gnu init --name my-saas-app
```

Output:
```
Initialized gitnu vault in /path/to/vault
Created: .gitnu/
Created: domains/_global/agent.md
Created: .claude/skills/gitnu/SKILL.md
✓ Ready. Run 'gnu status' to see current state.
```

## Check Initial State

```bash
gnu status
```

Output:
```
On branch: main
Last commit: abc1234 "Initial commit" (2 seconds ago)

Active Context (estimated 464 tokens):
  Loaded:
    - domains/_global/agent.md
    - domains/_global/conventions.md
    - domains/my-saas-app/spec.md
    - domains/my-saas-app/learnings.md
    - domains/my-saas-app/decisions.md
    - domains/my-saas-app/todos.md
```

## Add Project Specification

Edit `domains/my-saas-app/spec.md`:

```markdown
# SaaS Application Specification

## Overview
Building a multi-tenant SaaS platform for team collaboration.

## Core Features
- User authentication (JWT)
- Team management
- Real-time collaboration
- API access

## Technical Requirements
- REST API
- PostgreSQL database
- Redis for caching
- WebSocket support
```

Commit the change:

```bash
gnu commit "Added initial project specification" --author human
```

Output:
```
[main def5678] Added initial project specification
  Author: Human (user)
  1 files changed, 1 insertions, 0 deletions
  Context: 2 domains, ~520 tokens
```

## Explore Alternative Approaches

Create a branch to explore GraphQL as an alternative to REST:

```bash
gnu branch explore-graphql --describe "Evaluating GraphQL vs REST"
gnu checkout explore-graphql
```

Make changes in the exploration branch:

```bash
echo "## GraphQL Decision

Pros:
- Single endpoint
- Type safety
- Efficient data fetching

Cons:
- More complex caching
- Learning curve for team" >> domains/my-saas-app/decisions.md

gnu commit "Explored GraphQL approach" --author agent --model "claude-3-5-sonnet"
```

## View Commit History

```bash
gnu log --oneline
```

Output:
```
abc123f (HEAD -> explore-graphql) Explored GraphQL approach
def5678 Added initial project specification
9876fed Initial commit
```

## Compare Branches

```bash
gnu diff main explore-graphql
```

Output:
```
Comparing main..explore-graphql

Context Changes:
~ Modified: domains/my-saas-app/decisions.md
  + "GraphQL Decision"
  + "Pros: Single endpoint, Type safety..."

Token delta: +85 tokens
```

## Merge Learnings Back

After deciding GraphQL is the right choice:

```bash
gnu checkout main
gnu merge explore-graphql
```

Output:
```
Merging explore-graphql into main
  Auto-merged: domains/my-saas-app/decisions.md (added 1 decision)

Created merge commit ghi7890
  "Merge explore-graphql: Explored GraphQL approach"
```

## Wikilink Resolution

```bash
gnu resolve "[[spec]]"
```

Output:
```
/path/to/vault/domains/my-saas-app/spec.md
```

## Load and Pin Context

```bash
gnu load domains/my-saas-app/
gnu pin "[[conventions]]"
gnu load --list
```

Output:
```
Currently loaded:
  - domains/_global/agent.md
  - domains/_global/conventions.md [pinned]
  - domains/my-saas-app/spec.md
  - domains/my-saas-app/decisions.md
```

## Export Context

```bash
gnu context --compress > context-export.txt
```

This creates a single file with all active context, compressed for token efficiency.

## Generate Summary

```bash
gnu summary
```

Output:
```
# gitnu Summary

## Current State
- Branch: main
- Last commit: ghi7890 "Merge explore-graphql" (5m ago)
- Active domains: 2
- Estimated tokens: ~620

## What You Know
- Project: my-saas-app (see [[spec]], [[decisions]])
- Key decisions: GraphQL API, PostgreSQL, JWT auth
- Recent learnings: GraphQL caching considerations

## What's Changed
- Modified today: decisions.md
- Open questions: None

## Available Branches
- main (current)
- explore-graphql (merged)
```

## Rollback Mistake

If you realize the GraphQL decision was wrong:

```bash
gnu log --oneline
# Find the commit before the merge

gnu rewind def5678
```

Output:
```
Rewound main to commit def5678 "Added initial project specification"
  Restored context from def5678
⚠️  This is a destructive operation. Original commits still exist in .gitnu/objects/
```

## Branch Management

```bash
# List all branches
gnu branch

# Create new branch
gnu branch experiment-nosql --describe "Trying NoSQL database"

# Delete branch
gnu branch -d explore-graphql
```

## Advanced: Context Management

```bash
# Unload all non-pinned files
gnu unload --all

# Exclude certain files
gnu pin --exclude "domains/archive/*"

# Unpin a file
gnu unpin "[[conventions]]"
```

## Integration with Claude Code

When Claude Code starts in a vault with gitnu initialized, it automatically:

1. Runs `gnu summary` to orient itself
2. Loads relevant context with `gnu load`
3. Commits after completing milestones
4. Uses branches when exploring alternatives
5. Records learnings in `learnings.md` and commits them

The SKILL.md file teaches Claude all these patterns automatically!

## Tips and Best Practices

1. **Commit Often**: After each significant milestone or decision
2. **Use Branches**: For any alternative approach or experiment
3. **Pin Core Files**: Keep essential context always loaded
4. **Descriptive Messages**: Make commit messages clear and searchable
5. **Summary First**: Start each session with `gnu summary`

## File Ownership Convention

- **Human writes**: `spec.md`, `prd.md`, `requirements.md`
- **Agent writes**: `learnings.md`, `decisions.md`, `patterns.md`
- **Both write**: `todos.md`, other topic files

This keeps responsibilities clear and prevents conflicts.
