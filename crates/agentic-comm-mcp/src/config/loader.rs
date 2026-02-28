//! Config and path resolution for the .acomm store file.
//!
//! Priority:
//! 1. CLI argument (passed to SessionManager::new)
//! 2. ACOMM_STORE environment variable
//! 3. .acomm/store.acomm in the current directory
//! 4. ~/.store.acomm fallback

use std::path::PathBuf;

use sha2::{Digest, Sha256};

/// Resolve the store path using the priority chain.
///
/// Priority:
/// 1. `ACOMM_STORE` environment variable
/// 2. `.acomm/store.acomm` in the current working directory
/// 3. `~/.store.acomm` home-directory fallback
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

/// Resolve the project-local store path using a deterministic resolution chain.
///
/// Priority:
/// 1. CLI argument (if `cli_path` is `Some`)
/// 2. `ACOMM_STORE` environment variable
/// 3. `.acomm/store.acomm` relative to the current working directory
/// 4. `~/.store.acomm` home-directory fallback
pub fn project_store_path(cli_path: Option<PathBuf>) -> PathBuf {
    // 1. CLI arg takes top priority
    if let Some(p) = cli_path {
        return p;
    }

    // 2. Environment variable
    if let Ok(p) = std::env::var("ACOMM_STORE") {
        return PathBuf::from(p);
    }

    // 3. Project-local .acomm directory (relative to cwd)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let local = cwd.join(".acomm").join("store.acomm");
    if local.exists() || cwd.join(".acomm").exists() {
        return local;
    }

    // 4. Home directory fallback
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".store.acomm")
}

/// Compute a deterministic SHA-256 identity for the current project.
///
/// Uses the canonical current working directory path as input, producing
/// a hex-encoded hash that uniquely identifies the project. This prevents
/// cross-project data contamination when multiple projects use agentic-comm.
pub fn project_identity() -> String {
    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .canonicalize()
        .unwrap_or_else(|_| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

    let mut hasher = Sha256::new();
    hasher.update(cwd.to_string_lossy().as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_identity_is_deterministic() {
        let id1 = project_identity();
        let id2 = project_identity();
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_project_store_path_cli_overrides() {
        let cli = Some(PathBuf::from("/tmp/custom.acomm"));
        let path = project_store_path(cli);
        assert_eq!(path, PathBuf::from("/tmp/custom.acomm"));
    }

    #[test]
    fn test_project_store_path_fallback() {
        // Without CLI or env var, should fall back to home or local
        let path = project_store_path(None);
        // Just verify it returns something reasonable
        assert!(!path.to_string_lossy().is_empty());
    }
}
