//! System prompt construction for agents
//! Based on OpenCode's 5-layer architecture from session/system.ts

use crate::agent::types::AgentInfo;
use std::path::{Path, PathBuf};

/// System prompt builder - constructs the complete system prompt for an agent
pub struct SystemPromptBuilder {
    pub agent: AgentInfo,
    pub working_dir: PathBuf,
    pub provider_id: String,
}

impl SystemPromptBuilder {
    pub fn new(agent: AgentInfo, working_dir: PathBuf, provider_id: String) -> Self {
        Self {
            agent,
            working_dir,
            provider_id,
        }
    }

    /// Build the complete system prompt as a single string
    ///
    /// Layers (in order):
    /// 1. Header (provider-specific)
    /// 2. Base/Agent prompt (agent-specific or provider default)
    /// 3. Environment context (cwd, platform, project tree)
    /// 4. Custom instructions (from AGENTS.md, CLAUDE.md)
    ///
    /// NOTE: Dynamic reminders are NOT in system prompt!
    /// They are injected into user messages via insert_reminders() in executor
    pub fn build(&self, model_id: &str) -> String {
        let mut prompt = String::new();

        // Layer 1: Header
        let header = self.header();
        if !header.is_empty() {
            prompt.push_str(&header);
            prompt.push_str("\n\n");
        }

        // Layer 2: Agent prompt or provider default
        prompt.push_str(&self.agent_or_provider_prompt(model_id));
        prompt.push_str("\n\n");

        // Layer 3: Environment context
        prompt.push_str(&self.environment_context());

        // Layer 4: Custom instructions (if any)
        if let Some(instructions) = self.load_custom_instructions() {
            prompt.push_str("\n\n");
            prompt.push_str(&instructions);
        }

        prompt
    }

    /// Layer 1: Provider-specific header (shameless copy from OpenCode)
    fn header(&self) -> String {
        if self.provider_id.contains("anthropic") {
            include_str!("../prompts/anthropic_spoof.txt")
                .trim()
                .to_string()
        } else {
            String::new()
        }
    }

    /// Layer 2: Agent-specific prompt OR provider default
    fn agent_or_provider_prompt(&self, model_id: &str) -> String {
        // If agent has custom prompt, use it
        if let Some(ref prompt) = self.agent.prompt {
            return prompt.clone();
        }

        // Otherwise use provider default based on MODEL ID (not provider)
        self.provider_default_prompt(model_id)
    }

    /// Provider default prompts (shameless copy from OpenCode)
    /// NOTE: OpenCode matches on modelID, not providerID!
    fn provider_default_prompt(&self, model_id: &str) -> String {
        // Match OpenCode's logic exactly from session/system.ts:provider()
        if model_id.contains("gpt-5") {
            include_str!("../prompts/codex.txt").to_string()
        } else if model_id.contains("gpt-") || model_id.contains("o1") || model_id.contains("o3") {
            include_str!("../prompts/beast.txt").to_string()
        } else if model_id.contains("gemini-") {
            include_str!("../prompts/gemini.txt").to_string()
        } else if model_id.contains("claude") {
            include_str!("../prompts/anthropic.txt").to_string()
        } else if model_id.contains("polaris-alpha") {
            include_str!("../prompts/polaris.txt").to_string()
        } else {
            // Default: PROMPT_ANTHROPIC_WITHOUT_TODO = qwen.txt
            include_str!("../prompts/qwen.txt").to_string()
        }
    }

    /// Layer 3: Environment context (working directory, platform, project tree)
    /// Matches OpenCode's format exactly with <env> and <project> XML tags
    fn environment_context(&self) -> String {
        let mut parts = vec![
            "Here is some useful information about the environment you are running in:".to_string(),
            "<env>".to_string(),
        ];

        // Working directory
        parts.push(format!(
            "  Working directory: {}",
            self.working_dir.display()
        ));

        // Git repo check
        let is_git = std::process::Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .current_dir(&self.working_dir)
            .output()
            .ok()
            .map(|o| o.status.success())
            .unwrap_or(false);

        parts.push(format!(
            "  Is directory a git repo: {}",
            if is_git { "yes" } else { "no" }
        ));

        // Platform
        parts.push(format!("  Platform: {}", std::env::consts::OS));

        // Today's date (matching OpenCode format)
        let date = chrono::Local::now().format("%a %b %d %Y").to_string();
        parts.push(format!("  Today's date: {}", date));

        parts.push("</env>".to_string());

        // File tree (matches OpenCode's <files> tag)
        parts.push("<files>".to_string());
        let tree = self.generate_project_tree();
        if !tree.is_empty() {
            // Indent the tree content
            for line in tree.lines() {
                parts.push(format!("  {}", line));
            }
        }
        parts.push("</files>".to_string());

        parts.join("\n")
    }

    /// Generate project tree (ls output)
    /// Uses simple directory traversal with limits
    fn generate_project_tree(&self) -> String {
        const MAX_ITEMS: usize = 200;
        const MAX_DEPTH: usize = 5;

        let mut items = Vec::new();
        let mut total_items = 0;

        self.collect_tree_items(
            &self.working_dir,
            0,
            MAX_DEPTH,
            &mut items,
            &mut total_items,
            MAX_ITEMS,
        );

        let mut tree = String::new();
        for (depth, name, is_dir) in &items {
            let indent = "  ".repeat(*depth);
            let suffix = if *is_dir { "/" } else { "" };
            tree.push_str(&format!("{}{}{}\n", indent, name, suffix));
        }

        if total_items > MAX_ITEMS {
            tree.push_str(&format!(
                "\n[{} more items truncated]\n",
                total_items - MAX_ITEMS
            ));
        }

        tree
    }

    /// Recursively collect tree items with depth limiting
    fn collect_tree_items(
        &self,
        dir: &Path,
        depth: usize,
        max_depth: usize,
        items: &mut Vec<(usize, String, bool)>,
        total_items: &mut usize,
        max_items: usize,
    ) {
        if depth >= max_depth || items.len() >= max_items {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            if items.len() >= max_items {
                break;
            }

            *total_items += 1;

            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and common ignore patterns
            if file_name.starts_with('.')
                || file_name == "node_modules"
                || file_name == "target"
                || file_name == "__pycache__"
            {
                continue;
            }

            let is_dir = entry.path().is_dir();

            items.push((depth, file_name.clone(), is_dir));

            // Recurse into directories
            if is_dir {
                self.collect_tree_items(
                    &entry.path(),
                    depth + 1,
                    max_depth,
                    items,
                    total_items,
                    max_items,
                );
            }
        }
    }

    /// Layer 4: Load custom instructions from AGENTS.md, CLAUDE.md, etc.
    /// Shameless copy of OpenCode's search pattern from session/system.ts:custom()
    fn load_custom_instructions(&self) -> Option<String> {
        let mut instructions = Vec::new();

        // Local files to search for (in priority order)
        let local_files = vec!["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];

        // Global files to check
        let global_files = self.get_global_instruction_paths();

        // Search local files using findUp (from working_dir up to git root)
        if let Some(root) = self.find_git_root() {
            for filename in &local_files {
                if let Some(path) = self.find_up(filename, &self.working_dir, &root) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        instructions.push(format!(
                            "Instructions from: {}\n{}",
                            path.display(),
                            content
                        ));
                        break; // Only first match per search
                    }
                }
            }
        } else {
            // No git repo - just check working_dir directly
            for filename in &local_files {
                let path = self.working_dir.join(filename);
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        instructions.push(format!(
                            "Instructions from: {}\n{}",
                            path.display(),
                            content
                        ));
                        break;
                    }
                }
            }
        }

        // Check global instruction files
        for path in global_files {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    instructions.push(format!(
                        "Instructions from: {}\n{}",
                        path.display(),
                        content
                    ));
                    break; // Only first global match
                }
            }
        }

        if instructions.is_empty() {
            None
        } else {
            Some(instructions.join("\n\n"))
        }
    }

    /// Get paths to global instruction files
    fn get_global_instruction_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // XDG config directory: ~/.config/crow/AGENTS.md
        if let Ok(home) = std::env::var("HOME") {
            let xdg_config =
                std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

            paths.push(PathBuf::from(xdg_config).join("crow/AGENTS.md"));

            // Legacy ~/.claude/CLAUDE.md for compatibility
            paths.push(PathBuf::from(format!("{}/.claude/CLAUDE.md", home)));
        }

        paths
    }

    /// Find git repository root
    fn find_git_root(&self) -> Option<PathBuf> {
        let output = std::process::Command::new("git")
            .args(&["rev-parse", "--show-toplevel"])
            .current_dir(&self.working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(PathBuf::from(root))
        } else {
            None
        }
    }

    /// Find a file by searching upward from start to root
    /// This matches OpenCode's Filesystem.findUp() behavior
    fn find_up(&self, filename: &str, start: &Path, root: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();

        loop {
            let candidate = current.join(filename);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }

            // Stop at root
            if current == root {
                break;
            }

            // Move up one directory
            if !current.pop() {
                break;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::AgentInfo;

    #[test]
    fn test_build_prompt_with_custom_agent_prompt() {
        let mut agent = AgentInfo::new("test");
        agent.prompt = Some("Custom agent prompt here".to_string());

        let builder =
            SystemPromptBuilder::new(agent, PathBuf::from("/tmp/test"), "moonshot".to_string());

        let prompt = builder.build("some-model");

        assert!(prompt.contains("Custom agent prompt here"));
        assert!(prompt.contains("Working directory: /tmp/test"));
        assert!(prompt.contains("Platform:"));
    }

    #[test]
    fn test_environment_format() {
        let agent = AgentInfo::new("test");
        let builder = SystemPromptBuilder::new(agent, PathBuf::from("."), "moonshot".to_string());
        let prompt = builder.build("some-model");

        // Check for OpenCode-style XML tags
        assert!(prompt.contains("<env>"));
        assert!(prompt.contains("</env>"));
        assert!(prompt.contains("<files>"));
        assert!(prompt.contains("</files>"));
        assert!(prompt.contains("Today's date:"));
        assert!(prompt.contains("Is directory a git repo:"));
    }

    #[test]
    fn test_provider_prompts() {
        let agent = AgentInfo::new("test");

        // Test default (qwen.txt) - model doesn't match any known pattern
        let builder =
            SystemPromptBuilder::new(agent.clone(), PathBuf::from("."), "moonshot".to_string());
        let prompt = builder.build("unknown-model");
        // qwen.txt should be loaded (PROMPT_ANTHROPIC_WITHOUT_TODO)
        assert!(!prompt.is_empty());

        // Test anthropic header and claude model
        let builder2 = SystemPromptBuilder::new(agent, PathBuf::from("."), "anthropic".to_string());
        let prompt2 = builder2.build("claude-3-5-sonnet");
        assert!(prompt2.contains("Claude")); // From anthropic_spoof.txt header
    }
}
