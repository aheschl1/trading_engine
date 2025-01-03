use std::{self, path::PathBuf, env};

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(home_dir) = env::var_os("HOME") {
        PathBuf::from(path.replacen("~", &home_dir.to_string_lossy(), 1))
    } else {
        PathBuf::from(path) // Fallback to original path if HOME isn't set
    }
}