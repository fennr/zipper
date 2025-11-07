use crate::error::{Result, ZipperError};
use std::path::{Path, PathBuf};

/// Check whether the given path is a git repository root
pub fn is_git_repository(path: &Path) -> bool {
    let git_dir = path.join(".git");
    git_dir.exists() && git_dir.is_dir()
}

/// Ascend until the repository root is found
pub fn get_git_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();

    loop {
        if is_git_repository(&current) {
            return Some(current);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    None
}

/// Generate a repository identifier relative to the root path
pub fn get_repo_name(repo_path: &Path, root_path: &Path) -> Result<String> {
    let relative = repo_path.strip_prefix(root_path).map_err(|_| {
        ZipperError::git(format!(
            "Repository {} is not located inside the root directory {}",
            repo_path.display(),
            root_path.display()
        ))
    })?;

    let name = if relative.as_os_str().is_empty() {
        "root".to_string()
    } else {
        relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "_")
    };

    Ok(name)
}
