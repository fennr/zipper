# Zipper

Zipper is a Rust CLI tool that scans a project tree, detects git repositories, and archives them while respecting their `.gitignore` rules. When the root itself is a git repo, it is archived directly; otherwise, each repo is packed individually and bundled into a final archive.

## Usage

```
cargo run --release -- \
  --root <PATH> \
  --name <NAME> \
  --format <zip|tar.gz|tar.xz> \
  --verbose
```

- `--root` (`-r`) points to the directory to crawl.
- `--name` (`-n`) sets the resulting archive name; defaults to the root directory name.
- `--format` (`-f`) chooses the archive format (default `zip`).
- `--verbose` (`-v`) increases logging (multiple flags for more detail).

Build:

```
cargo build --release
```
