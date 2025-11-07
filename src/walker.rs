use crate::error::{Result, ZipperError};
use crate::git;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Locate all git repositories under the provided root directory
pub fn find_git_repositories(root: &Path) -> Result<Vec<PathBuf>> {
    info!("Scanning for git repositories in: {}", root.display());

    if !root.exists() {
        return Err(ZipperError::config(format!(
            "Directory does not exist: {}",
            root.display()
        )));
    }

    if !root.is_dir() {
        return Err(ZipperError::config(format!(
            "Path is not a directory: {}",
            root.display()
        )));
    }

    let mut repositories = Vec::new();
    let mut visited_git_dirs = std::collections::HashSet::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(false)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.to_string_lossy().contains(".git") {
                    if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                        if let Some(parent) = path.parent() {
                            if !visited_git_dirs.contains(parent) {
                                visited_git_dirs.insert(parent.to_path_buf());
                                repositories.push(parent.to_path_buf());
                                info!("Detected git repository: {}", parent.display());
                            }
                        }
                    }
                    continue;
                }

                if path.is_dir() && git::is_git_repository(path) && !visited_git_dirs.contains(path)
                {
                    visited_git_dirs.insert(path.to_path_buf());
                    repositories.push(path.to_path_buf());
                    info!("Detected git repository: {}", path.display());
                }
            }
            Err(err) => {
                warn!("Filesystem traversal error: {}", err);
            }
        }
    }

    if git::is_git_repository(root) && !visited_git_dirs.contains(root) {
        repositories.push(root.to_path_buf());
        info!("Root directory is a git repository: {}", root.display());
    }

    info!("Found {} git repositories", repositories.len());
    Ok(repositories)
}

/// Walk files inside a git repository honoring ignore rules
pub fn walk_repository_files(repo_path: &Path) -> Result<Vec<PathBuf>> {
    debug!("Walking repository: {}", repo_path.display());

    let mut files = Vec::new();

    let walker = WalkBuilder::new(repo_path)
        .git_ignore(true)
        .git_exclude(true)
        .hidden(false)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.to_string_lossy().contains(".git") {
                    continue;
                }

                if path.is_file() {
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                warn!(
                    "Failed to walk repository files {}: {}",
                    repo_path.display(),
                    err
                );
            }
        }
    }

    debug!(
        "Collected {} files in repository {}",
        files.len(),
        repo_path.display()
    );
    Ok(files)
}
