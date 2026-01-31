use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a commit in the gitnu vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub parent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub author: Author,
    pub message: String,
    pub context_summary: ContextSummary,
    pub snapshot_path: PathBuf,
}

/// Author of a commit - either human or AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Author {
    Human { name: String },
    Agent { 
        model: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String> 
    },
}

impl Author {
    pub fn display(&self) -> String {
        match self {
            Author::Human { name } => format!("Human ({})", name),
            Author::Agent { model, .. } => format!("Agent ({})", model),
        }
    }
}

/// Summary of what's in the context at commit time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub domains_loaded: Vec<String>,
    pub files_modified: Vec<PathBuf>,
    pub files_added: Vec<PathBuf>,
    pub files_removed: Vec<PathBuf>,
    pub token_estimate: usize,
}

/// Reference to a branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRef {
    pub name: String,
    pub head: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// The staging area / relevance queue
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Index {
    #[serde(default)]
    pub staged: Vec<StagedFile>,
    #[serde(default)]
    pub pinned: Vec<PathBuf>,
    #[serde(default)]
    pub excluded: Vec<PathBuf>,
    #[serde(default)]
    pub loaded: Vec<PathBuf>,
}

/// A file staged for inclusion in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedFile {
    pub path: PathBuf,
    pub reason: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn display(&self) -> &str {
        match self {
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

/// Configuration for the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub context: ContextConfig,
    pub agent: AgentConfig,
    pub pins: PinsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub vault_name: String,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_tokens: usize,
    pub auto_commit: bool,
    pub compress_snapshots: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub default_author: String,
    pub model_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinsConfig {
    pub always_load: Vec<String>,
    pub never_load: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            core: CoreConfig {
                vault_name: "unnamed".to_string(),
                default_branch: "main".to_string(),
                created_at: Utc::now(),
            },
            context: ContextConfig {
                max_tokens: 100_000,
                auto_commit: false,
                compress_snapshots: true,
            },
            agent: AgentConfig {
                default_author: "agent".to_string(),
                model_hint: "claude-3-5-sonnet".to_string(),
            },
            pins: PinsConfig {
                always_load: vec![
                    "domains/_global/agent.md".to_string(),
                ],
                never_load: vec![
                    "domains/archive/*".to_string(),
                ],
            },
        }
    }
}

/// Snapshot manifest for quick metadata access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<FileInfo>,
    pub total_files: usize,
    pub total_size: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
}
