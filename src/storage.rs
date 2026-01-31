use crate::errors::*;
use crate::models::*;
use crate::utils::*;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{Write, BufRead, BufReader};
use std::path::PathBuf;
use chrono::Utc;
use tar::{Archive, Builder};
use walkdir::WalkDir;

pub struct Storage {
    pub vault_root: PathBuf,
}

impl Storage {
    pub fn new(vault_root: PathBuf) -> Self {
        Storage { vault_root }
    }

    pub fn gitnu_dir(&self) -> PathBuf {
        self.vault_root.join(".gitnu")
    }

    pub fn domains_dir(&self) -> PathBuf {
        self.vault_root.join("domains")
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.gitnu_dir().join("objects")
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.gitnu_dir().join("refs/heads")
    }

    pub fn commits_dir(&self) -> PathBuf {
        self.gitnu_dir().join("commits")
    }

    /// Initialize vault structure
    pub fn init(&self, vault_name: &str) -> Result<()> {
        let gitnu = self.gitnu_dir();
        if gitnu.exists() {
            return Err(GitnuError::AlreadyInitialized(self.vault_root.clone()));
        }

        // Create directory structure
        ensure_dir(&gitnu)?;
        ensure_dir(&self.objects_dir())?;
        ensure_dir(&self.refs_dir())?;
        ensure_dir(&self.commits_dir())?;
        ensure_dir(&self.domains_dir())?;
        
        // Create config.toml
        let mut config = Config::default();
        config.core.vault_name = vault_name.to_string();
        self.save_config(&config)?;

        // Create initial HEAD pointing to main
        self.write_head("main")?;

        // Create empty index
        self.save_index(&Index::default())?;

        Ok(())
    }

    /// Save configuration
    pub fn save_config(&self, config: &Config) -> Result<()> {
        let path = self.gitnu_dir().join("config.toml");
        let content = toml::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load configuration
    pub fn load_config(&self) -> Result<Config> {
        let path = self.gitnu_dir().join("config.toml");
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save index
    pub fn save_index(&self, index: &Index) -> Result<()> {
        let path = self.gitnu_dir().join("index.json");
        let content = serde_json::to_string_pretty(index)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load index
    pub fn load_index(&self) -> Result<Index> {
        let path = self.gitnu_dir().join("index.json");
        if !path.exists() {
            return Ok(Index::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Write HEAD reference
    pub fn write_head(&self, branch: &str) -> Result<()> {
        let path = self.gitnu_dir().join("HEAD");
        fs::write(path, format!("ref: refs/heads/{}", branch))?;
        Ok(())
    }

    /// Read HEAD reference (returns branch name)
    pub fn read_head(&self) -> Result<String> {
        let path = self.gitnu_dir().join("HEAD");
        let content = fs::read_to_string(path)?;
        
        if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
            Ok(branch.trim().to_string())
        } else {
            Ok(content.trim().to_string())
        }
    }

    /// Write branch reference
    pub fn write_branch_ref(&self, branch: &str, commit_hash: &str) -> Result<()> {
        let path = self.refs_dir().join(branch);
        fs::write(path, commit_hash)?;
        Ok(())
    }

    /// Read branch reference
    pub fn read_branch_ref(&self, branch: &str) -> Result<Option<String>> {
        let path = self.refs_dir().join(branch);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        Ok(Some(content.trim().to_string()))
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let refs_dir = self.refs_dir();
        if !refs_dir.exists() {
            return Ok(vec![]);
        }

        let mut branches = Vec::new();
        for entry in fs::read_dir(refs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    branches.push(name.to_string());
                }
            }
        }
        Ok(branches)
    }

    /// Delete branch
    pub fn delete_branch(&self, branch: &str) -> Result<()> {
        let path = self.refs_dir().join(branch);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Create snapshot of domains directory
    pub fn create_snapshot(&self, commit_hash: &str) -> Result<PathBuf> {
        let object_dir = self.objects_dir().join(commit_hash);
        ensure_dir(&object_dir)?;

        let snapshot_path = object_dir.join("snapshot.tar.gz");
        let tar_gz = File::create(&snapshot_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        let domains_dir = self.domains_dir();
        if domains_dir.exists() {
            for entry in WalkDir::new(&domains_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let rel_path = relative_path(&self.vault_root, path);
                    tar.append_path_with_name(path, &rel_path)?;
                }
            }
        }

        tar.finish()?;

        // Create manifest
        self.create_manifest(commit_hash)?;

        Ok(snapshot_path)
    }

    /// Create manifest for snapshot
    fn create_manifest(&self, commit_hash: &str) -> Result<()> {
        let domains_dir = self.domains_dir();
        let mut files = Vec::new();
        let mut total_size = 0u64;

        if domains_dir.exists() {
            for entry in WalkDir::new(&domains_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let metadata = fs::metadata(path)?;
                    let size = metadata.len();
                    total_size += size;
                    
                    let hash = hash_file(path)?;
                    let rel_path = relative_path(&self.vault_root, path);
                    
                    files.push(FileInfo {
                        path: rel_path,
                        hash,
                        size,
                    });
                }
            }
        }

        let manifest = Manifest {
            total_files: files.len(),
            total_size,
            created_at: Utc::now(),
            files,
        };

        let manifest_path = self.objects_dir().join(commit_hash).join("manifest.json");
        let content = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, content)?;

        Ok(())
    }

    /// Restore snapshot
    pub fn restore_snapshot(&self, commit_hash: &str) -> Result<()> {
        let snapshot_path = self.objects_dir()
            .join(commit_hash)
            .join("snapshot.tar.gz");

        if !snapshot_path.exists() {
            return Err(GitnuError::CommitNotFound(commit_hash.to_string()));
        }

        // Clear domains directory first
        let domains_dir = self.domains_dir();
        if domains_dir.exists() {
            fs::remove_dir_all(&domains_dir)?;
        }
        ensure_dir(&domains_dir)?;

        // Extract snapshot
        let tar_gz = File::open(snapshot_path)?;
        let dec = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(dec);
        archive.unpack(&self.vault_root)?;

        Ok(())
    }

    /// Append commit to branch log
    pub fn append_commit(&self, branch: &str, commit: &Commit) -> Result<()> {
        let log_path = self.commits_dir().join(format!("{}.jsonl", branch));
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        let line = serde_json::to_string(commit)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read commits from branch log
    pub fn read_commits(&self, branch: &str) -> Result<Vec<Commit>> {
        let log_path = self.commits_dir().join(format!("{}.jsonl", branch));
        if !log_path.exists() {
            return Ok(vec![]);
        }

        let file = File::open(log_path)?;
        let reader = BufReader::new(file);
        let mut commits = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let commit: Commit = serde_json::from_str(&line)?;
                commits.push(commit);
            }
        }

        Ok(commits)
    }

    /// Find commit by hash (searches all branches)
    pub fn find_commit(&self, hash: &str) -> Result<Option<Commit>> {
        for branch in self.list_branches()? {
            let commits = self.read_commits(&branch)?;
            for commit in commits {
                if commit.hash.starts_with(hash) {
                    return Ok(Some(commit));
                }
            }
        }
        Ok(None)
    }

    /// Get current HEAD commit
    pub fn get_head_commit(&self) -> Result<Option<Commit>> {
        let branch = self.read_head()?;
        let commit_hash = match self.read_branch_ref(&branch)? {
            Some(hash) => hash,
            None => return Ok(None),
        };
        self.find_commit(&commit_hash)
    }
}
