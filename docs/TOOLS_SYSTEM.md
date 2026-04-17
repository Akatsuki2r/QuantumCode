# Tool System

## Overview

The Tool System defines the actions that the AI agent can execute. Tools are controlled by the Router's Tool Policy, which determines which tools are allowed based on intent and mode.

**Status**: 85% Implemented

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Tool System                             │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Tool Trait  │  │   Registry  │  │   Tool Policy       │  │
│  │             │  │             │  │   (from Router)     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         └────────────────┼─────────────────────┘             │
│                          │                                   │
│         ┌────────────────┼────────────────┐                  │
│         │                │                │                  │
│    ┌────▼────┐    ┌─────▼─────┐   ┌──────▼──────┐           │
│    │  Read   │    │   Write   │   │    Bash     │           │
│    └─────────┘    └───────────┘   └─────────────┘           │
│         │                │                │                  │
│    ┌────▼────┐    ┌─────▼─────┐                              │
│    │  Grep   │    │   Glob    │                              │
│    └─────────┘    └───────────┘                              │
└─────────────────────────────────────────────────────────────┘
```

## Tool Trait

**File**: `src/tools/mod.rs`

```rust
pub trait Tool: Send + Sync {
    /// Tool name (for AI to reference)
    fn name(&self) -> &str;
    
    /// Tool description (for AI to understand capabilities)
    fn description(&self) -> &str;
    
    /// JSON schema for parameters
    fn parameters(&self) -> serde_json::Value;
    
    /// Execute the tool with given parameters
    fn execute(&self, params: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
```

## Implemented Tools

### 1. Read Tool

**File**: `src/tools/read_file.rs`

**Purpose**: Read file contents

**Parameters**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to the file to read"
    }
  },
  "required": ["path"]
}
```

**Implementation**:
```rust
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str { "Read" }
    
    fn description(&self) -> &str {
        "Read the contents of a file. Use this to examine source code, config files, etc."
    }
    
    fn execute(&self, params: &Value) -> Result<Value, ToolError> {
        let path = params.get("path")
            .ok_or(ToolError::MissingParameter("path"))?
            .as_str()
            .ok_or(ToolError::InvalidParameter("path must be a string"))?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::IoError(e))?;
        
        Ok(json!({ "content": content, "path": path }))
    }
}
```

**Policy**:
- Allowed in: All modes
- Requires confirmation: No

---

### 2. Write Tool

**File**: `src/tools/write_file.rs`

**Purpose**: Create or overwrite files

**Parameters**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to the file to write"
    },
    "content": {
      "type": "string",
      "description": "Content to write to the file"
    }
  },
  "required": ["path", "content"]
}
```

**Policy**:
- Allowed in: Build mode
- Disallowed in: Plan, Review, Chat modes
- Requires confirmation: Yes (for destructive operations)

---

### 3. Bash Tool

**File**: `src/tools/bash.rs`

**Purpose**: Execute shell commands

**Parameters**:
```json
{
  "type": "object",
  "properties": {
    "command": {
      "type": "string",
      "description": "Shell command to execute"
    },
    "cwd": {
      "type": "string",
      "description": "Working directory for command"
    }
  },
  "required": ["command"]
}
```

**Policy**:
- Allowed in: Build, Debug modes
- Disallowed in: Plan, Review, Chat modes
- Requires confirmation: Yes

**Security Considerations**:
- No command sandboxing (TODO)
- Runs with user privileges
- Should validate against dangerous commands

---

### 4. Grep Tool

**File**: `src/tools/grep.rs`

**Purpose**: Search file contents using ripgrep

**Parameters**:
```json
{
  "type": "object",
  "properties": {
    "pattern": {
      "type": "string",
      "description": "Regex pattern to search for"
    },
    "path": {
      "type": "string",
      "description": "Directory or file to search in"
    }
  },
  "required": ["pattern"]
}
```

**Implementation**:
Uses `ripgrep` (rg) for fast searching.

**Policy**:
- Allowed in: All modes except Chat
- Requires confirmation: No

---

### 5. Glob Tool

**File**: `src/tools/glob.rs`

**Purpose**: Find files by pattern

**Parameters**:
```json
{
  "type": "object",
  "properties": {
    "pattern": {
      "type": "string",
      "description": "Glob pattern (e.g., **/*.rs)"
    },
    "path": {
      "type": "string",
      "description": "Base directory to search in"
    }
  },
  "required": ["pattern"]
}
```

**Policy**:
- Allowed in: All modes except Chat
- Requires confirmation: No

---

## Tool Policy

**File**: `src/router/tools.rs`

The Router determines which tools are allowed based on intent and mode.

### Policy by Mode

```rust
pub fn pick_tools(intent: Intent, mode: AgentMode) -> ToolPolicy {
    match mode {
        AgentMode::Build => build_mode_tools(intent),
        AgentMode::Review => ToolPolicy::read_only(),
        AgentMode::Debug => debug_mode_tools(),
        AgentMode::Plan => plan_mode_tools(),
        AgentMode::Chat => chat_mode_tools(),
    }
}
```

### Default Policy (Build Mode)

```rust
pub fn default_policy() -> Self {
    Self {
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
        ],
        disallowed_tools: vec![],
        require_confirmation: false,
    }
}
```

### Read-Only Policy (Review Mode)

```rust
pub fn read_only() -> Self {
    Self {
        allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        disallowed_tools: vec!["Write".to_string(), "Bash".to_string()],
        require_confirmation: false,
    }
}
```

### Plan Mode Policy

```rust
fn plan_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        disallowed_tools: vec!["Write".to_string(), "Bash".to_string()],
        require_confirmation: false,
    }
}
```

### Chat Mode Policy

```rust
fn chat_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec!["Read".to_string()],
        disallowed_tools: vec![
            "Write".to_string(),
            "Bash".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
        ],
        require_confirmation: false,
    }
}
```

### Debug Mode Policy

```rust
fn debug_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec![
            "Read".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
            "Bash".to_string(),
        ],
        disallowed_tools: vec!["Write".to_string()],
        require_confirmation: true,
    }
}
```

## Tool Registry

**File**: `src/tools/mod.rs`

The registry manages tool lookup and execution:

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register built-in tools
        registry.register("Read", Box::new(ReadFileTool));
        registry.register("Write", Box::new(WriteFileTool));
        registry.register("Bash", Box::new(BashTool));
        registry.register("Grep", Box::new(GrepTool));
        registry.register("Glob", Box::new(GlobTool));
        
        registry
    }
    
    pub fn register(&mut self, name: &str, tool: Box<dyn Tool>) {
        self.tools.insert(name.to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
    
    pub fn execute(&self, name: &str, params: &Value) -> Result<Value, ToolError> {
        let tool = self.get(name)
            .ok_or(ToolError::ToolNotFound(name))?;
        tool.execute(params)
    }
}
```

## Tool Filtering

**File**: `src/router/tools.rs`

Tools are filtered based on policy:

```rust
pub fn filter_tools_by_policy<'a>(
    tool_names: &'a [&str],
    policy: &ToolPolicy
) -> Vec<&'a str> {
    tool_names
        .iter()
        .filter(|name| policy.is_tool_allowed(name))
        .copied()
        .collect()
}
```

## Tool Error Types

```rust
pub enum ToolError {
    /// Tool not found in registry
    ToolNotFound(String),
    
    /// Missing required parameter
    MissingParameter(&'static str),
    
    /// Invalid parameter value
    InvalidParameter(String),
    
    /// I/O error
    IoError(std::io::Error),
    
    /// Permission denied
    PermissionDenied(String),
    
    /// Execution timeout
    Timeout,
    
    /// Other error
    Other(String),
}
```

## Tool Execution Flow

```
1. AI generates tool call
   │
   ▼
2. Parse tool name and parameters
   │
   ▼
3. Validate against ToolPolicy
   │
   ├── Disallowed → Reject
   │
   ▼
4. Check if confirmation required
   │
   ├── Yes → Prompt user
   │   └── User declines → Reject
   │
   ▼
5. Execute tool
   │
   ▼
6. Handle result
   │
   ├── Success → Return to AI
   │
   └── Error → Return error to AI
```

## Security Considerations

### Current Security Measures

1. **Mode-based restrictions**: Write/Bash disabled in read-only modes
2. **Confirmation**: Destructive operations require user confirmation
3. **Policy enforcement**: Tools filtered before AI sees them

### Missing Security (TODO)

1. **Path validation**: No protection against path traversal
2. **Command validation**: No filtering of dangerous bash commands
3. **Sandboxing**: Commands run with full user privileges
4. **Rate limiting**: No limits on tool execution frequency
5. **Audit logging**: No record of tool executions

### Recommended Security Enhancements

```rust
// Path validation
pub fn validate_path(path: &str, allowed_dirs: &[PathBuf]) -> Result<PathBuf, SecurityError> {
    let resolved = std::fs::canonicalize(path)?;
    if allowed_dirs.iter().any(|dir| resolved.starts_with(dir)) {
        Ok(resolved)
    } else {
        Err(SecurityError::PathNotAllowed)
    }
}

// Command validation
pub fn validate_command(cmd: &str) -> Result<(), SecurityError> {
    let dangerous = ["rm -rf /", "dd if=", "mkfs", ":(){:|:&};:"];
    if dangerous.iter().any(|d| cmd.contains(d)) {
        return Err(SecurityError::DangerousCommand);
    }
    Ok(())
}
```

## Testing

### Unit Tests

Currently minimal tool tests exist. Need to add:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_tool() {
        let tool = ReadFileTool;
        let params = json!({ "path": "test.txt" });
        let result = tool.execute(&params);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_tool_policy_filter() {
        let policy = ToolPolicy::read_only();
        assert!(policy.is_tool_allowed("Read"));
        assert!(!policy.is_tool_allowed("Write"));
        assert!(!policy.is_tool_allowed("Bash"));
    }
}
```

## Future Enhancements

### 1. Parallel Tool Execution

Execute independent tools in parallel:

```rust
pub async fn execute_parallel(tools: Vec<ToolCall>) -> Vec<ToolResult> {
    let futures = tools.into_iter().map(|call| {
        tokio::spawn(async move {
            execute_tool(call.name, call.params)
        })
    });
    
    join_all(futures).await
}
```

### 2. Tool Caching

Cache results of read operations:

```rust
pub struct CachedToolExecutor {
    cache: moka::future::Cache<String, Value>,
}

impl CachedToolExecutor {
    pub async fn execute(&self, name: &str, params: &Value) -> Result<Value, ToolError> {
        let key = format!("{}:{}", name, serde_json::to_string(params)?);
        
        if let Some(cached) = self.cache.get(&key).await {
            return Ok(cached);
        }
        
        let result = self.registry.execute(name, params).await?;
        self.cache.insert(key, result).await;
        Ok(result)
    }
}
```

### 3. Custom Tools (Plugins)

Allow users to define custom tools:

```rust
pub struct CustomTool {
    name: String,
    script: PathBuf,
}

impl Tool for CustomTool {
    fn execute(&self, params: &Value) -> Result<Value, ToolError> {
        Command::new(&self.script)
            .arg(serde_json::to_string(params)?)
            .output()?
            .try_into()
    }
}
```

## Tool Comparison

| Tool | Read | Write | Exec | Search |
|------|------|-------|------|--------|
| Read | ✅ | ❌ | ❌ | ❌ |
| Write | ❌ | ✅ | ❌ | ❌ |
| Bash | ❌ | ❌ | ✅ | ❌ |
| Grep | ✅ | ❌ | ❌ | ✅ |
| Glob | ✅ | ❌ | ❌ | ✅ |

## Integration with Router

The Router's Tool Policy layer determines which tools are available:

```rust
let decision = route(prompt, cwd, &config);

// Agent should use decision.tools
agent.set_allowed_tools(decision.tools.allowed_tools);
agent.set_confirmation_required(decision.tools.require_confirmation);
```

This ensures:
- Mode restrictions are enforced
- Destructive operations require confirmation
- AI only sees available tools
