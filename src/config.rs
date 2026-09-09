use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::chip8::{Platform, Quirks};

#[derive(Deserialize)]
struct RawConfig {
    version: Platform,
}

/// Loads `config.toml` and resolves it to the `Quirks` preset it selects.
pub fn load_quirks(path: &Path) -> Quirks {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read config file at {}: {err}", path.display()));
    let raw: RawConfig =
        toml::from_str(&text).unwrap_or_else(|err| panic!("Failed to parse config: {err}"));
    raw.version.quirks()
}

/// Resolves a file name relative to the running executable's directory,
/// rather than the process's current working directory.
pub fn exe_relative(file_name: &str) -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to resolve current exe path");
    path.pop();
    path.push(file_name);
    path
}
