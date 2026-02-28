//! Config and path resolution for the .acomm store file.
//!
//! Priority:
//! 1. CLI argument (passed to SessionManager::new)
//! 2. ACOMM_STORE environment variable
//! 3. .acomm/store.acomm in the current directory
//! 4. ~/.store.acomm fallback

use std::path::PathBuf;

/// Resolve the store path using the priority chain.
pub fn resolve_comm_path() -> PathBuf {
    // Check env var
    if let Ok(p) = std::env::var("ACOMM_STORE") {
        return PathBuf::from(p);
    }

    // Check local .acomm directory
    let local = PathBuf::from(".acomm/store.acomm");
    if local.exists() {
        return local;
    }

    // Fallback to home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".store.acomm")
}
