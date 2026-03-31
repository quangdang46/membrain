//! Membrain MCP and JSON-RPC daemon implementation.
//!
//! Provides the external programmatic access to membrain-core.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use membrain_core::persistence::default_local_paths;
use tracing_subscriber::fmt::writer::MakeWriter;
use tracing_subscriber::EnvFilter;

pub mod daemon;
pub mod mcp;
pub mod preflight;
pub mod rpc;

pub fn init_file_tracing(log_root: Option<&Path>) {
    let filter = match tracing_filter() {
        Ok(filter) => filter,
        Err(error) => {
            eprintln!("warning: invalid MEMBRAIN_LOG filter: {error}");
            return;
        }
    };

    let log_path = match resolve_log_path(log_root) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("warning: membrain tracing log path unavailable: {error}");
            return;
        }
    };

    let writer = match SharedFileWriter::new(&log_path) {
        Ok(writer) => writer,
        Err(error) => {
            eprintln!(
                "warning: failed to open membrain log file {}: {error}",
                log_path.display()
            );
            return;
        }
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .compact()
        .try_init();
}

fn tracing_filter() -> Result<EnvFilter, tracing_subscriber::filter::ParseError> {
    match std::env::var("MEMBRAIN_LOG") {
        Ok(value) if !value.trim().is_empty() => EnvFilter::try_new(value),
        Ok(_) | Err(std::env::VarError::NotPresent) => EnvFilter::try_new("info"),
        Err(std::env::VarError::NotUnicode(_)) => EnvFilter::try_new("info"),
    }
}

fn resolve_log_path(log_root: Option<&Path>) -> Result<PathBuf, String> {
    match std::env::var("MEMBRAIN_LOG_PATH") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        Ok(_) | Err(std::env::VarError::NotPresent) => {
            let root_dir = match log_root {
                Some(root_dir) => root_dir.to_path_buf(),
                None => default_local_paths()
                    .map(|paths| paths.root_dir)
                    .map_err(|error| error.to_string())?,
            };
            Ok(default_log_path(&root_dir))
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            Err("MEMBRAIN_LOG_PATH is not valid unicode".to_string())
        }
    }
}

fn default_log_path(root_dir: &Path) -> PathBuf {
    root_dir.join("membrain.log")
}

#[derive(Clone)]
struct SharedFileWriter {
    file: Arc<Mutex<std::fs::File>>,
}

impl SharedFileWriter {
    fn new(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

struct SharedFileGuard<'a> {
    guard: MutexGuard<'a, std::fs::File>,
}

impl Write for SharedFileGuard<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.guard.flush()
    }
}

impl<'a> MakeWriter<'a> for SharedFileWriter {
    type Writer = SharedFileGuard<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        SharedFileGuard {
            guard: match self.file.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::default_log_path;
    use std::path::PathBuf;

    #[test]
    fn default_log_path_stays_under_membrain_root() {
        assert_eq!(
            default_log_path(PathBuf::from("/tmp/membrain-home").as_path()),
            PathBuf::from("/tmp/membrain-home/membrain.log")
        );
    }
}
