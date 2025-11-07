use clap::Parser;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use zipper::archive;
use zipper::config::{ArchiveFormat, Config};
use zipper::error::{Result, ZipperError};
use zipper::git;
use zipper::walker;

fn main() {
    let config = Config::parse();

    // Configure logging
    let filter = match config.verbose {
        0 => EnvFilter::new("info"),
        1 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    if let Err(e) = run(config) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(config: Config) -> Result<()> {
    let root = std::fs::canonicalize(&config.root)
        .map_err(|e| ZipperError::config(format!("Failed to canonicalize root path: {e}")))?;

    let repositories = walker::find_git_repositories(&root)?;

    if repositories.is_empty() {
        return Err(ZipperError::config("No git repositories found".to_string()));
    }

    let project_name = config.name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive")
            .to_string()
    });

    info!("Starting zipper with options:");
    info!("  Root directory: {}", root.display());
    info!("  Project name: {}", project_name);
    info!("  Archive format: {:?}", config.format);

    // Detect whether the root directory is the only repository we found
    let is_root_only_repo = if repositories.len() == 1 {
        let repo_path = &repositories[0];
        let normalized_repo = std::fs::canonicalize(repo_path).map_err(|e| {
            ZipperError::config(format!(
                "Failed to canonicalize repository path {}: {}",
                repo_path.display(),
                e
            ))
        })?;
        normalized_repo == root && git::is_git_repository(&root)
    } else {
        false
    };

    if is_root_only_repo {
        info!("Root directory is the only repository; archiving it directly");

        let final_archive_name = format!("{}.{}", project_name, config.format.extension());
        let final_archive_path = PathBuf::from(&final_archive_name);

        archive_repository(&root, &final_archive_path, config.format)?;

        info!("Archive created: {}", final_archive_path.display());
        info!("Done! Repository archived");
    } else {
        let temp_dir = TempDir::new().map_err(|e| {
            ZipperError::archive(format!("Failed to create temporary directory: {e}"))
        })?;
        let temp_path = temp_dir.path();

        info!("Creating per-repository archives");
        let mut repo_archives = Vec::new();

        for repo_path in &repositories {
            let repo_name = git::get_repo_name(repo_path, &root)?;
            let archive_name = format!("{}.{}", repo_name, config.format.extension());
            let archive_path = temp_path.join(&archive_name);

            info!(
                "Archiving repository: {} -> {}",
                repo_path.display(),
                archive_name
            );

            archive_repository(repo_path, &archive_path, config.format)?;

            repo_archives.push((archive_name, archive_path));
        }

        info!("Creating final archive");
        let final_archive_name = format!("{}.{}", project_name, config.format.extension());
        let final_archive_path = PathBuf::from(&final_archive_name);

        let mut final_archive =
            archive::create_archive_builder(config.format, final_archive_path.clone())?;

        for (archive_name, archive_path) in &repo_archives {
            info!("Adding repository archive to bundle: {}", archive_name);
            final_archive.add_file(archive_path, Path::new(archive_name))?;
        }

        final_archive.finish()?;

        info!("Final archive created: {}", final_archive_path.display());
        info!(
            "Done! Found and archived {} repositories",
            repositories.len()
        );
    }

    Ok(())
}

fn archive_repository(repo_path: &Path, output_path: &Path, format: ArchiveFormat) -> Result<()> {
    let files = walker::walk_repository_files(repo_path)?;

    let mut archive = archive::create_archive_builder(format, output_path.to_path_buf())?;

    for file_path in &files {
        let relative_path = file_path
            .strip_prefix(repo_path)
            .map_err(|e| ZipperError::archive(format!("Failed to obtain relative path: {e}")))?;

        archive.add_file(file_path, relative_path)?;
    }

    archive.finish()?;

    Ok(())
}
