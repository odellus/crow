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
    /// 5. Dynamic reminders (context-aware)
    pub fn build(&self) -> String {
        let mut prompt = String::new();

        // Layer 1: Header
        prompt.push_str(&self.header());
        prompt.push_str("\n\n");

        // Layer 2: Agent prompt or provider default
        prompt.push_str(&self.agent_or_provider_prompt());
        prompt.push_str("\n\n");

        // Layer 3: Environment context
        prompt.push_str(&self.environment_context());
        prompt.push_str("\n\n");

        // Layer 4: Custom instructions (if any)
        if let Some(instructions) = self.load_custom_instructions() {
            prompt.push_str(&instructions);
            prompt.push_str("\n\n");
        }

        // Layer 5: Dynamic reminders
        prompt.push_str(&self.dynamic_reminders());

        prompt
    }

    /// Layer 1: Provider-specific header
    fn header(&self) -> String {
        match self.provider_id.as_str() {
            "anthropic" | "claude" => {
                "You are Claude, a large language model trained by Anthropic.".to_string()
            }
            "openai" | "gpt" => {
                "You are ChatGPT, a large language model trained by OpenAI.".to_string()
            }
            "moonshot" | "moonshotai" => {
                "You are an AI coding assistant powered by Moonshot AI.".to_string()
            }
            _ => "You are an AI coding assistant.".to_string(),
        }
    }

    /// Layer 2: Agent-specific prompt OR provider default
    fn agent_or_provider_prompt(&self) -> String {
        // If agent has custom prompt, use it
        if let Some(ref prompt) = self.agent.prompt {
            return prompt.clone();
        }

        // Otherwise use provider default
        self.provider_default_prompt()
    }

    /// Provider default prompts (used when agent has no custom prompt)
    fn provider_default_prompt(&self) -> String {
        match self.provider_id.as_str() {
            "anthropic" | "claude" | "moonshot" | "moonshotai" => {
                // Claude-style default
                r#"You are a helpful AI coding assistant. Your goal is to help users with their coding tasks.

When working on tasks:
- Be systematic and thorough
- Break complex problems into steps
- Use tools to read files, write code, and run commands
- Verify your work before finishing
- If you're unsure, ask for clarification

You have access to various tools for file operations, running commands, and more. Use them effectively to accomplish the user's goals."#.to_string()
            }
            _ => "You are a helpful AI coding assistant.".to_string(),
        }
    }

    /// Layer 3: Environment context (working directory, platform, project tree)
    fn environment_context(&self) -> String {
        let mut context = String::from("# Environment\n\n");

        // Working directory
        context.push_str(&format!(
            "Working directory: {}\n",
            self.working_dir.display()
        ));

        // Platform
        context.push_str(&format!("Platform: {}\n", std::env::consts::OS));

        // Git information (if in git repo)
        if let Some(git_info) = self.get_git_info() {
            context.push_str(&git_info);
        }

        // Project structure
        context.push_str("\n## Project Structure\n\n");
        context.push_str(&self.generate_project_tree());

        context
    }

    /// Get git information (branch, status)
    fn get_git_info(&self) -> Option<String> {
        // Try to get git branch
        let branch_output = std::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.working_dir)
            .output()
            .ok()?;

        if !branch_output.status.success() {
            return None;
        }

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        Some(format!("Git branch: {}\n", branch))
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
    fn load_custom_instructions(&self) -> Option<String> {
        // Priority order:
        // 1. PROJECT_ROOT/AGENTS.md
        // 2. PROJECT_ROOT/CLAUDE.md
        // 3. ~/.opencode/AGENTS.md (not implementing global for now)

        let locations = vec![
            self.working_dir.join("AGENTS.md"),
            self.working_dir.join("CLAUDE.md"),
        ];

        for path in locations {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                return Some(format!("# Custom Instructions\n\n{}", contents));
            }
        }

        None
    }

    /// Layer 5: Dynamic reminders (context-aware additions)
    fn dynamic_reminders(&self) -> String {
        let mut reminders = String::from("# Important Reminders\n\n");

        // Agent-specific reminders
        match self.agent.name.as_str() {
            "discriminator" => {
                reminders.push_str(
                    "- You are a DISCRIMINATOR agent - your role is to VERIFY work, not do it\n",
                );
                reminders.push_str("- You can ONLY use the task_done tool\n");
                reminders.push_str(
                    "- Review the executor's work carefully and provide specific feedback\n",
                );
                reminders.push_str("- Call task_done(complete=true) when satisfied, or task_done(complete=false) with issues\n");
            }
            "supervisor" => {
                reminders.push_str("- Break work into 3-7 clear tasks\n");
                reminders.push_str("- Complete each task fully before moving to the next\n");
                reminders.push_str("- Use TodoWrite to track progress\n");
                reminders.push_str("- Verify completion with tests or validation\n");
            }
            "architect" => {
                reminders.push_str("- Think in phases (2-4 major phases)\n");
                reminders.push_str("- Delegate to Supervisor agents via Task tool\n");
                reminders.push_str("- Monitor progress and adapt strategy\n");
                reminders.push_str("- Operate autonomously without asking permission\n");
            }
            "plan" => {
                reminders.push_str("- You are in READ-ONLY mode\n");
                reminders.push_str("- You cannot edit files or run destructive commands\n");
                reminders.push_str("- Focus on analysis and planning\n");
            }
            _ => {
                reminders.push_str("- Use tools effectively to accomplish tasks\n");
                reminders.push_str("- Verify your work before finishing\n");
            }
        }

        reminders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{AgentInfo, AgentMode};

    #[test]
    fn test_build_prompt_with_custom_agent_prompt() {
        let mut agent = AgentInfo::new("test");
        agent.prompt = Some("Custom agent prompt here".to_string());

        let builder =
            SystemPromptBuilder::new(agent, PathBuf::from("/tmp/test"), "moonshot".to_string());

        let prompt = builder.build();

        assert!(prompt.contains("Custom agent prompt here"));
        assert!(prompt.contains("Working directory: /tmp/test"));
        assert!(prompt.contains("Platform:"));
    }

    #[test]
    fn test_discriminator_reminders() {
        let mut agent = AgentInfo::new("discriminator");
        agent.mode = AgentMode::Subagent;

        let builder = SystemPromptBuilder::new(agent, PathBuf::from("."), "moonshot".to_string());

        let prompt = builder.build();

        assert!(prompt.contains("DISCRIMINATOR"));
        assert!(prompt.contains("task_done"));
        assert!(prompt.contains("VERIFY work"));
    }

    #[test]
    fn test_provider_header() {
        let agent = AgentInfo::new("test");

        let builder =
            SystemPromptBuilder::new(agent.clone(), PathBuf::from("."), "moonshot".to_string());

        let prompt = builder.build();
        assert!(prompt.contains("Moonshot AI"));

        let builder2 = SystemPromptBuilder::new(agent, PathBuf::from("."), "anthropic".to_string());

        let prompt2 = builder2.build();
        assert!(prompt2.contains("Claude"));
    }
}
