//! Model supervisor for managing llama.cpp server processes
//!
//! Handles starting, stopping, and health checking llama.cpp server instances.

use color_eyre::eyre::{eyre, Result, WrapErr};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Default port for llama.cpp server
const DEFAULT_PORT: u16 = 8080;

/// Maximum time to wait for server startup (seconds)
const STARTUP_TIMEOUT_SECS: u64 = 60;

/// Health check interval (milliseconds)
const HEALTH_CHECK_INTERVAL_MS: u64 = 500;

/// Model supervisor for managing llama.cpp server processes
pub struct ModelSupervisor {
    /// Currently active model name
    active_model: Option<String>,
    /// Currently running server process
    process: Option<Child>,
    /// Port the server is running on
    port: u16,
    /// Mapping from model names to GGUF file paths
    model_paths: HashMap<String, PathBuf>,
    /// Optional draft model used for llama.cpp speculative decoding
    draft_model_path: Option<PathBuf>,
    /// Speculative decoding draft batch size
    draft_max: u16,
    /// Speculative decoding minimum accepted draft length
    draft_min: u16,
    /// Speculative decoding probability threshold
    draft_p_min: f32,
}

impl ModelSupervisor {
    /// Create a new model supervisor with default port
    pub fn new() -> Self {
        Self {
            active_model: None,
            process: None,
            port: DEFAULT_PORT,
            model_paths: HashMap::new(),
            draft_model_path: None,
            draft_max: 16,
            draft_min: 0,
            draft_p_min: 0.75,
        }
    }

    /// Create a new supervisor with custom port
    pub fn with_port(port: u16) -> Self {
        Self {
            active_model: None,
            process: None,
            port,
            model_paths: HashMap::new(),
            draft_model_path: None,
            draft_max: 16,
            draft_min: 0,
            draft_p_min: 0.75,
        }
    }

    /// Add a model path mapping
    pub fn add_model_path(&mut self, model_name: String, path: PathBuf) {
        self.model_paths.insert(model_name, path);
    }

    /// Set the port for the server
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    /// Configure llama.cpp speculative decoding for auto-started servers.
    pub fn set_speculative_decoding(
        &mut self,
        draft_model_path: Option<PathBuf>,
        draft_max: u16,
        draft_min: u16,
        draft_p_min: f32,
    ) {
        self.draft_model_path = draft_model_path;
        self.draft_max = draft_max;
        self.draft_min = draft_min;
        self.draft_p_min = draft_p_min;
    }

    /// Get the base URL for the server
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Get the currently active model
    pub fn active_model(&self) -> Option<&str> {
        self.active_model.as_deref()
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        // We can't check with try_wait() since it requires mutable access
        // Instead, we just check if we have a process handle
        self.process.is_some()
    }

    /// Ensure a model is loaded and server is running
    ///
    /// If the same model is already running, does nothing.
    /// If a different model is running, stops it and starts the new one.
    /// If no model is running, starts the server.
    pub fn ensure(&mut self, model: &str) -> Result<()> {
        // Same model already running
        if self.active_model.as_deref() == Some(model) && self.is_running() {
            return Ok(());
        }

        // Stop existing server if running
        self.stop()?;

        // Start new server
        self.start(model)?;

        // Wait for health check
        self.wait_for_ready()?;

        Ok(())
    }

    /// Start the llama.cpp server for a given model
    fn start(&mut self, model: &str) -> Result<()> {
        let model_path = self
            .model_paths
            .get(model)
            .ok_or_else(|| eyre!("Model path not configured for: {}", model))?;

        // Verify the model file exists
        if !model_path.exists() {
            return Err(eyre!("Model file not found: {:?}", model_path));
        }

        let port_arg = format!("--port={}", self.port);
        let model_arg = format!("--model={}", model_path.display());
        let ctx_arg = "--ctx-size=8192".to_string();

        let mut cmd = Command::new("llama-server");
        cmd.arg(&model_arg).arg(&port_arg).arg(&ctx_arg);

        if let Some(draft_model_path) = &self.draft_model_path {
            if !draft_model_path.exists() {
                return Err(eyre!("Draft model file not found: {:?}", draft_model_path));
            }

            cmd.arg(format!("--spec-draft-model={}", draft_model_path.display()))
                .arg(format!("--spec-draft-n-max={}", self.draft_max))
                .arg(format!("--spec-draft-n-min={}", self.draft_min))
                .arg(format!("--spec-draft-p-min={}", self.draft_p_min));
        }

        // Start the process
        let child = cmd
            .spawn()
            .wrap_err_with(|| format!("Failed to start llama-server for model: {}", model))?;

        self.process = Some(child);
        self.active_model = Some(model.to_string());

        tracing::info!(
            "Started llama.cpp server for model '{}' on port {}",
            model,
            self.port
        );

        Ok(())
    }

    /// Stop the currently running server
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            // Try graceful shutdown first
            process
                .kill()
                .wrap_err("Failed to kill llama-server process")?;

            // Wait for process to exit
            process
                .wait()
                .wrap_err("Failed to wait for llama-server process")?;

            tracing::info!("Stopped llama.cpp server");
        }

        self.active_model = None;
        Ok(())
    }

    /// Wait for the server to become ready (health check)
    fn wait_for_ready(&mut self) -> Result<()> {
        let url = format!("{}/health", self.base_url());

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(STARTUP_TIMEOUT_SECS);

        while start.elapsed() < timeout {
            // Check if process is still alive
            if let Some(ref mut process) = self.process {
                match process.try_wait() {
                    Ok(Some(status)) => {
                        // Process has exited
                        return Err(eyre!(
                            "llama-server process exited unexpectedly with status: {}",
                            status
                        ));
                    }
                    Ok(None) => {
                        // Process is still running, check health using async runtime
                        // Use a simple TCP connection check instead of blocking HTTP
                        let addr = format!("127.0.0.1:{}", self.port);
                        if std::net::TcpStream::connect(&addr).is_ok() {
                            tracing::info!("llama.cpp server is ready");
                            return Ok(());
                        }
                        // Not ready yet, wait and retry
                        std::thread::sleep(Duration::from_millis(HEALTH_CHECK_INTERVAL_MS));
                    }
                    Err(e) => {
                        return Err(eyre!("Failed to check process status: {}", e));
                    }
                }
            } else {
                return Err(eyre!("No process running"));
            }
        }

        Err(eyre!(
            "Timeout waiting for llama.cpp server to become ready"
        ))
    }

    /// Health check using async API
    pub async fn health_check(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/health", self.base_url());

        match client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

impl Default for ModelSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ModelSupervisor {
    fn drop(&mut self) {
        // Ensure process is killed when supervisor is dropped
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}

/// Thread-safe wrapper for ModelSupervisor
pub type SharedSupervisor = Arc<Mutex<ModelSupervisor>>;

/// Create a new shared supervisor
pub fn shared_supervisor() -> SharedSupervisor {
    Arc::new(Mutex::new(ModelSupervisor::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_supervisor() {
        let supervisor = ModelSupervisor::new();
        assert!(supervisor.active_model().is_none());
        assert!(!supervisor.is_running());
        assert_eq!(supervisor.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_custom_port() {
        let supervisor = ModelSupervisor::with_port(9090);
        assert_eq!(supervisor.base_url(), "http://localhost:9090");
    }

    #[test]
    fn test_add_model_path() {
        let mut supervisor = ModelSupervisor::new();
        supervisor.add_model_path(
            "llama3.2".to_string(),
            PathBuf::from("/models/llama3.2.gguf"),
        );
        assert!(supervisor.model_paths.contains_key("llama3.2"));
    }
}
