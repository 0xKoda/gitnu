use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum GitnuError {
    #[error("No gitnu vault found in current directory or parents\n  Run 'gnu init' to create a new vault\n  Or navigate to an existing vault directory")]
    NoVaultFound,

    #[error("Vault already initialized at {0}")]
    AlreadyInitialized(PathBuf),

    #[error("Branch '{0}' already exists\n  Use 'gnu checkout {0}' to switch to it\n  Or choose a different branch name")]
    BranchExists(String),

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("Commit '{0}' not found")]
    CommitNotFound(String),

    #[error("Uncommitted changes would be lost\n  Commit your changes first: gnu commit \"message\"\n  Or discard them with: gnu checkout --force")]
    UncommittedChanges,

    #[error("Merge conflict in {0}\n  Edit the file to resolve conflicts (look for <<<<<<< markers)\n  Then run: gnu commit \"Resolved merge conflict\"")]
    MergeConflict(String),

    #[error("Wikilink '{0}' not found in vault")]
    WikilinkNotFound(String),

    #[error("Wikilink '{0}' is ambiguous, matches: {}", .1.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "))]
    WikilinkAmbiguous(String, Vec<PathBuf>),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid commit reference: {0}")]
    InvalidCommitRef(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GitnuError>;
