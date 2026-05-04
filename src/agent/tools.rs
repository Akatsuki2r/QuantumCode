//! Minimalistic tools for agentic workflow

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A tool that the AI can call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
}

/// A tool call from the AI
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arg: String,
    pub content: Option<String>,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl ToolResult {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            success: true,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: msg,
            success: false,
        }
    }
}

/// Execute a Read tool
pub fn tool_read(arg: &str) -> ToolResult {
    let path = Path::new(arg);
    if !path.exists() {
        return ToolResult::error(format!("File not found: {}", arg));
    }

    match std::fs::read_to_string(path) {
        Ok(content) => ToolResult::success(content),
        Err(e) => ToolResult::error(format!("Failed to read {}: {}", arg, e)),
    }
}

/// Execute a Write tool
pub fn tool_write(arg: &str, content: &str) -> ToolResult {
    let path = Path::new(arg);

    // Create parent directories
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::error(format!("Failed to create directory: {}", e));
            }
        }
    }

    match std::fs::write(path, content) {
        Ok(_) => ToolResult::success(format!("Written {} bytes to {}", content.len(), arg)),
        Err(e) => ToolResult::error(format!("Failed to write {}: {}", arg, e)),
    }
}

/// Execute a Bash tool
pub fn tool_bash(arg: &str) -> ToolResult {
    // Parse command - use sh -c for proper shell behavior
    let output = std::process::Command::new("sh").arg("-c").arg(arg).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                ToolResult {
                    stdout: if stdout.is_empty() {
                        "(no output)".to_string()
                    } else {
                        stdout
                    },
                    stderr,
                    success: true,
                }
            } else {
                ToolResult {
                    stdout,
                    stderr,
                    success: false,
                }
            }
        }
        Err(e) => ToolResult::error(format!("Failed to execute: {}", e)),
    }
}

/// Execute a Grep tool
pub fn tool_grep(arg: &str, path: &str) -> ToolResult {
    let path_arg = if path.is_empty() { "." } else { path };

    let output = std::process::Command::new("grep")
        .args(["-n", "-r", arg, path_arg])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            ToolResult {
                stdout: if stdout.is_empty() {
                    "(no matches)".to_string()
                } else {
                    stdout
                },
                stderr,
                success: out.status.success(),
            }
        }
        Err(e) => ToolResult::error(format!("Grep failed: {}", e)),
    }
}

/// Execute a Glob tool
pub fn tool_glob(arg: &str) -> ToolResult {
    let pattern = if arg.is_empty() { "*" } else { arg };

    // Use find with glob pattern
    let output = std::process::Command::new("find")
        .args([".", "-name", pattern, "-type", "f"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            ToolResult::success(if stdout.is_empty() {
                "(no matches)".to_string()
            } else {
                stdout
            })
        }
        Err(e) => ToolResult::error(format!("Glob failed: {}", e)),
    }
}

/// Execute a Search tool (Web retrieval)
pub fn tool_search(arg: &str) -> ToolResult {
    // Placeholder for actual web search API integration (Tavily/Brave)
    // For now, we simulate a successful search result.
    let simulated_result = format!(
        "Web search results for '{}':\n\
         1. Documentation and community discussions regarding the topic.\n\
         2. Recent updates and guide highlights for '{}'.\n\
         (Internet access simulation active via search tool)",
        arg, arg
    );
    ToolResult::success(simulated_result)
}

/// Type for tool execution handlers
pub type ToolHandler = fn(&ToolCall) -> ToolResult;

/// A registry for managing and executing tools
pub struct ToolRegistry {
    tools: HashMap<String, (Tool, ToolHandler)>,
}

impl ToolRegistry {
    /// Create a new ToolRegistry with default tools
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register default tools
        registry.register_tool("Read", "path -> file contents", |call| tool_read(&call.arg));
        registry.register_tool("Write", "path + content -> create/overwrite file", |call| {
            tool_write(&call.arg, call.content.as_deref().unwrap_or(""))
        });
        registry.register_tool("Bash", "cmd -> stdout/stderr", |call| tool_bash(&call.arg));
        registry.register_tool("Grep", "pattern -> recursive content matches", |call| {
            tool_grep(&call.arg, "")
        });
        registry.register_tool("Glob", "pattern -> matching file paths", |call| {
            tool_glob(&call.arg)
        });
        registry.register_tool("Search", "query -> web summary", |call| {
            tool_search(&call.arg)
        });
        registry.register_tool("Research", "topic -> deeper web synthesis", |call| {
            tool_search(&call.arg)
        });

        registry
    }

    /// Register a new tool at runtime
    pub fn register_tool(&mut self, name: &str, description: &str, handler: ToolHandler) {
        self.tools.insert(
            name.to_lowercase(),
            (
                Tool {
                    name: name.to_string(),
                    description: description.to_string(),
                },
                handler,
            ),
        );
    }

    /// Execute a tool by its call
    pub fn execute_tool(&self, call: &ToolCall) -> ToolResult {
        if let Some((_, handler)) = self.tools.get(&call.name.to_lowercase()) {
            handler(call)
        } else {
            ToolResult::error(format!("Unknown tool: {}", call.name))
        }
    }

    /// List all available tools for the system prompt
    pub fn list_tools(&self) -> String {
        self.list_tools_for(None)
    }

    /// List only tools allowed by the router policy.
    pub fn list_tools_for(&self, allowed_tools: Option<&[String]>) -> String {
        let mut list = String::new();
        let mut sorted_tools: Vec<_> = self.tools.values().collect();
        sorted_tools.sort_by(|a, b| a.0.name.cmp(&b.0.name));

        for (tool, _) in sorted_tools {
            if let Some(allowed) = allowed_tools {
                if !allowed
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&tool.name))
                {
                    continue;
                }
            }
            list.push_str(&format!("{}({})\n", tool.name, tool.description));
        }
        list
    }

    /// Provide the tool call format for the system prompt
    pub fn tool_call_format(&self) -> String {
        self.tool_call_format_for(None)
    }

    /// Provide compact examples only for tools available in the current mode.
    pub fn tool_call_format_for(&self, allowed_tools: Option<&[String]>) -> String {
        let is_allowed = |tool_name: &str| {
            allowed_tools.map_or(true, |allowed| {
                allowed
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(tool_name))
            })
        };

        let mut lines = vec!["Call XML only when a tool is needed:".to_string()];
        if is_allowed("Read") {
            lines.push("<tool><name>Read</name><arg>src/main.rs</arg></tool>".to_string());
        }
        if is_allowed("Write") {
            lines.push(
                "<tool><name>Write</name><arg>path</arg><content>full content</content></tool>"
                    .to_string(),
            );
        }
        if is_allowed("Bash") {
            lines.push("<tool><name>Bash</name><arg>cargo test</arg></tool>".to_string());
        }
        if is_allowed("Grep") {
            lines.push("<tool><name>Grep</name><arg>fn main</arg></tool>".to_string());
        }
        if is_allowed("Glob") {
            lines.push("<tool><name>Glob</name><arg>*.rs</arg></tool>".to_string());
        }
        if lines.len() == 1 {
            lines.push("<tool><name>ToolName</name><arg>value</arg></tool>".to_string());
        }
        lines.push("Use exact tool names. Omit tools when ready to answer.".to_string());
        lines.join("\n")
    }
}

/// Helper to get default tool registry
pub fn get_tools() -> ToolRegistry {
    ToolRegistry::new()
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
