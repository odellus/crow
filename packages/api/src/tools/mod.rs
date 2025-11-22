//! Tool system for agent execution
//! Mirrors OpenCode's tool execution model

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

pub mod bash;
pub mod batch;
pub mod edit;
pub mod glob;
pub mod grep;
pub mod invalid;
pub mod list;
pub mod lsp_diagnostics;
pub mod lsp_hover;
pub mod multiedit;
pub mod patch;
pub mod read;
pub mod task;
pub mod todoread;
pub mod todowrite;
pub mod webfetch;
pub mod websearch;
pub mod work_completed;
pub mod write;

pub use bash::BashTool;
pub use batch::BatchTool;
pub use edit::EditTool;
pub use glob::GlobTool;
pub use grep::GrepTool;
pub use invalid::InvalidTool;
pub use list::ListTool;
pub use lsp_diagnostics::LspDiagnosticsTool;
pub use lsp_hover::LspHoverTool;
pub use multiedit::MultiEditTool;
pub use patch::PatchTool;
pub use read::ReadTool;
pub use task::TaskTool;
pub use todoread::TodoReadTool;
pub use todowrite::TodoWriteTool;
pub use webfetch::WebFetchTool;
pub use websearch::WebSearchTool;
pub use work_completed::WorkCompletedTool;
pub use write::WriteTool;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub status: ToolStatus,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub metadata: Value,
}

/// Tool execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Pending,
    Running,
    Completed,
    Error,
}

/// Tool execution context - provides session and environment info
#[derive(Clone)]
pub struct ToolContext {
    // Required fields
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub working_dir: std::path::PathBuf,

    // Additional context matching OpenCode
    pub project_root: std::path::PathBuf, // Git root or working_dir
    pub call_id: Option<String>,          // Unique tool call ID
    pub provider_id: Option<String>,      // LLM provider
    pub model_id: Option<String>,         // LLM model

    // Cancellation support
    pub abort: Option<CancellationToken>, // Cancellation token for aborting

    // LSP support
    pub lsp: Option<std::sync::Arc<crate::lsp::Lsp>>,
}

impl ToolContext {
    /// Get project root, defaulting to working_dir
    pub fn root(&self) -> &std::path::Path {
        &self.project_root
    }

    /// Check if tool should abort
    pub fn should_abort(&self) -> bool {
        self.abort
            .as_ref()
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// Create a basic context (for backwards compatibility)
    pub fn new(
        session_id: String,
        message_id: String,
        agent: String,
        working_dir: std::path::PathBuf,
    ) -> Self {
        let project_root = find_project_root(&working_dir);
        Self {
            session_id,
            message_id,
            agent,
            working_dir,
            project_root,
            call_id: None,
            provider_id: None,
            model_id: None,
            abort: None,
            lsp: None,
        }
    }
}

/// Find project root (git root or current directory)
pub fn find_project_root(working_dir: &std::path::Path) -> std::path::PathBuf {
    // Try git root first
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(working_dir)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return std::path::PathBuf::from(root);
        }
    }

    // Fall back to working directory
    working_dir.to_path_buf()
}

/// Tool trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (e.g., "bash", "read", "write")
    fn name(&self) -> &str;

    /// Tool description for LLM
    fn description(&self) -> &str;

    /// JSON schema for tool parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with given input and context
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult;
}

/// Registry of all available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry with standard tools (no Task tool)
    /// For full functionality including Task tool, use new_with_deps()
    pub fn new() -> Self {
        // Create shared TodoWrite tool for TodoRead to reference
        let todo_write_shared = std::sync::Arc::new(TodoWriteTool::new());

        let tools: Vec<Box<dyn Tool>> = vec![
            // File operations
            Box::new(BashTool),
            Box::new(EditTool),
            Box::new(GlobTool),
            Box::new(GrepTool),
            Box::new(ListTool),
            Box::new(ReadTool),
            Box::new(WriteTool),
            // Todo management (critical for planning!)
            Box::new((*todo_write_shared).clone()),
            Box::new(TodoReadTool::new(todo_write_shared)),
            // Web tools
            Box::new(WebFetchTool),
            Box::new(WebSearchTool::new()),
            // LSP tools
            Box::new(LspHoverTool),
            Box::new(LspDiagnosticsTool),
            // Note: TaskTool requires dependencies, use new_with_deps() instead
            // Error handling
            Box::new(InvalidTool),
            // Note: WorkCompletedTool hidden for now
        ];

        Self { tools }
    }

    /// Create a new tool registry with all tools including Task tool
    pub async fn new_with_deps(
        session_store: std::sync::Arc<crate::session::SessionStore>,
        agent_registry: std::sync::Arc<crate::agent::AgentRegistry>,
        lock_manager: std::sync::Arc<crate::session::SessionLockManager>,
        provider_config: crate::providers::ProviderConfig,
    ) -> std::sync::Arc<Self> {
        // Create shared TodoWrite tool for TodoRead to reference
        let todo_write_shared = std::sync::Arc::new(TodoWriteTool::new());

        // Create registry Arc that we'll populate (using parking_lot to avoid poisoning)
        let registry = std::sync::Arc::new(parking_lot::RwLock::new(None::<std::sync::Arc<Self>>));
        let registry_for_task = registry.clone();

        // Create tools including Task tool (it will get the registry later via Arc)
        let tools: Vec<Box<dyn Tool>> = vec![
            // File operations
            Box::new(BashTool),
            Box::new(EditTool),
            Box::new(GlobTool),
            Box::new(GrepTool),
            Box::new(ListTool),
            Box::new(ReadTool),
            Box::new(WriteTool),
            // Todo management (critical for planning!)
            Box::new((*todo_write_shared).clone()),
            Box::new(TodoReadTool::new(todo_write_shared)),
            // Web tools
            Box::new(WebFetchTool),
            Box::new(WebSearchTool::new()),
            // LSP tools
            Box::new(LspHoverTool),
            Box::new(LspDiagnosticsTool),
            // Subagent spawning with dependencies
            Box::new(
                TaskTool::new(
                    session_store,
                    agent_registry,
                    registry_for_task,
                    lock_manager,
                    provider_config,
                )
                .await,
            ), // NOW ASYNC!
            // Error handling
            Box::new(InvalidTool),
            // Note: WorkCompletedTool hidden for now
        ];

        let tool_registry = std::sync::Arc::new(Self { tools });

        // Store the registry reference for TaskTool to use
        *registry.write() = Some(tool_registry.clone());

        tool_registry
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|b| b.as_ref())
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        input: Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, String> {
        let tool = self
            .get(name)
            .ok_or_else(|| format!("Tool not found: {}", name))?;

        Ok(tool.execute(input, ctx).await)
    }

    /// Convert tools to OpenAI ChatCompletionTool format
    #[cfg(feature = "server")]
    pub fn to_openai_tools(&self) -> Vec<async_openai::types::ChatCompletionTool> {
        use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};

        self.tools
            .iter()
            .map(|tool| ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: tool.name().to_string(),
                    description: Some(tool.description().to_string()),
                    parameters: Some(tool.parameters_schema()),
                    strict: None,
                },
            })
            .collect()
    }

    /// List all tool IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name().to_string()).collect()
    }

    /// List all tools with their schemas
    pub fn list_tools(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "id": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters_schema()
                })
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
