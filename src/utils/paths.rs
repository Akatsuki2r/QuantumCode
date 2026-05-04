use std::path::PathBuf;
use directories::ProjectDirs;

/// Get the base directory for session storage
pub fn get_sessions_dir() -> PathBuf {
    ProjectDirs::from("com", "quantumn", "code")
        .map(|dirs| dirs.config_dir().join("sessions"))
        .unwrap_or_else(|| {
            // Fallback to a local .quantumn/sessions directory if system paths aren't available
            PathBuf::from(".quantumn").join("sessions")
        })
}

/// Get the full path for a specific session file
pub fn get_session_path(id: &str) -> PathBuf {
    get_sessions_dir().join(format!("{}.json", id))
}