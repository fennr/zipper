use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "zipper")]
#[command(about = "CLI tool for archiving git repositories inside a project")]
pub struct Config {
    /// Root directory to scan
    #[arg(short = 'r', long = "root", value_name = "PATH")]
    pub root: PathBuf,

    /// Output archive name (defaults to the root directory name if omitted)
    #[arg(short = 'n', long = "name", value_name = "NAME")]
    pub name: Option<String>,

    /// Archive format (zip, tar.gz, tar.xz)
    #[arg(
        short = 'f',
        long = "format",
        value_name = "FORMAT",
        default_value = "zip"
    )]
    pub format: ArchiveFormat,

    /// Verbosity level for logging
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
}

impl std::str::FromStr for ArchiveFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zip" => Ok(ArchiveFormat::Zip),
            "tar.gz" | "targz" => Ok(ArchiveFormat::TarGz),
            "tar.xz" | "tarxz" => Ok(ArchiveFormat::TarXz),
            _ => Err(format!(
                "Unknown archive format: {s}. Supported formats: zip, tar.gz, tar.xz"
            )),
        }
    }
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::TarXz => "tar.xz",
        }
    }
}
