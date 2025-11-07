use crate::error::{Result, ZipperError};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub trait ArchiveBuilder: Send {
    fn add_file(&mut self, path: &Path, archive_path: &Path) -> Result<()>;
    fn finish(self: Box<Self>) -> Result<()>;
}

pub struct ZipArchiveBuilder {
    writer: zip::ZipWriter<File>,
    output_path: PathBuf,
}

impl ZipArchiveBuilder {
    pub fn new(output_path: PathBuf) -> Result<Self> {
        let file = File::create(&output_path)
            .map_err(|e| ZipperError::archive(format!("Failed to create archive file: {e}")))?;
        let writer = zip::ZipWriter::new(file);

        info!("Creating ZIP archive: {}", output_path.display());

        Ok(Self {
            writer,
            output_path,
        })
    }
}

impl ArchiveBuilder for ZipArchiveBuilder {
    fn add_file(&mut self, path: &Path, archive_path: &Path) -> Result<()> {
        debug!(
            "Adding file to ZIP: {} -> {}",
            path.display(),
            archive_path.display()
        );

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let archive_path_str = archive_path.to_string_lossy().replace('\\', "/");

        self.writer
            .start_file(archive_path_str, options)
            .map_err(|e| ZipperError::archive(format!("Failed to add file to archive: {e}")))?;

        io::copy(&mut File::open(path)?, &mut self.writer)
            .map_err(|e| ZipperError::archive(format!("Failed to copy file into archive: {e}")))?;

        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.writer
            .finish()
            .map_err(|e| ZipperError::archive(format!("Failed to finalize archive: {e}")))?;

        info!("ZIP archive created: {}", self.output_path.display());
        Ok(())
    }
}

pub struct TarGzArchiveBuilder {
    tar: tar::Builder<flate2::write::GzEncoder<File>>,
    output_path: PathBuf,
}

impl TarGzArchiveBuilder {
    pub fn new(output_path: PathBuf) -> Result<Self> {
        let file = File::create(&output_path)
            .map_err(|e| ZipperError::archive(format!("Failed to create archive file: {e}")))?;
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let tar = tar::Builder::new(encoder);

        info!("Creating TAR.GZ archive: {}", output_path.display());

        Ok(Self { tar, output_path })
    }
}

impl ArchiveBuilder for TarGzArchiveBuilder {
    fn add_file(&mut self, path: &Path, archive_path: &Path) -> Result<()> {
        debug!(
            "Adding file to TAR.GZ: {} -> {}",
            path.display(),
            archive_path.display()
        );

        let mut file = File::open(path).map_err(|e| {
            ZipperError::archive(format!("Failed to open file {}: {e}", path.display()))
        })?;

        self.tar
            .append_file(archive_path, &mut file)
            .map_err(|e| ZipperError::archive(format!("Failed to add file to archive: {e}")))?;

        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.tar
            .finish()
            .map_err(|e| ZipperError::archive(format!("Failed to finalize archive: {e}")))?;

        info!("TAR.GZ archive created: {}", self.output_path.display());
        Ok(())
    }
}

pub struct TarXzArchiveBuilder {
    tar: tar::Builder<xz2::write::XzEncoder<File>>,
    output_path: PathBuf,
}

impl TarXzArchiveBuilder {
    pub fn new(output_path: PathBuf) -> Result<Self> {
        let file = File::create(&output_path)
            .map_err(|e| ZipperError::archive(format!("Failed to create archive file: {e}")))?;
        let encoder = xz2::write::XzEncoder::new(file, 6);
        let tar = tar::Builder::new(encoder);

        info!("Creating TAR.XZ archive: {}", output_path.display());

        Ok(Self { tar, output_path })
    }
}

impl ArchiveBuilder for TarXzArchiveBuilder {
    fn add_file(&mut self, path: &Path, archive_path: &Path) -> Result<()> {
        debug!(
            "Adding file to TAR.XZ: {} -> {}",
            path.display(),
            archive_path.display()
        );

        let mut file = File::open(path).map_err(|e| {
            ZipperError::archive(format!("Failed to open file {}: {e}", path.display()))
        })?;

        self.tar
            .append_file(archive_path, &mut file)
            .map_err(|e| ZipperError::archive(format!("Failed to add file to archive: {e}")))?;

        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.tar
            .finish()
            .map_err(|e| ZipperError::archive(format!("Failed to finalize archive: {e}")))?;

        info!("TAR.XZ archive created: {}", self.output_path.display());
        Ok(())
    }
}

pub fn create_archive_builder(
    format: crate::config::ArchiveFormat,
    output_path: PathBuf,
) -> Result<Box<dyn ArchiveBuilder>> {
    match format {
        crate::config::ArchiveFormat::Zip => Ok(Box::new(ZipArchiveBuilder::new(output_path)?)),
        crate::config::ArchiveFormat::TarGz => Ok(Box::new(TarGzArchiveBuilder::new(output_path)?)),
        crate::config::ArchiveFormat::TarXz => Ok(Box::new(TarXzArchiveBuilder::new(output_path)?)),
    }
}
