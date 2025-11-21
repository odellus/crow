//! Edit tool - modifies existing files using exact and fuzzy string replacements
//! This is the primary way the LLM modifies code
//! Based on OpenCode's edit tool with advanced fuzzy matching

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use similar::{ChangeTag, TextDiff};
use std::path::Path;
use tokio::fs;

pub struct EditTool;

#[derive(Deserialize)]
struct EditInput {
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "oldString")]
    old_string: String,
    #[serde(rename = "newString")]
    new_string: String,
    #[serde(default)]
    replace_all: bool,
}

#[derive(Serialize, Deserialize)]
struct EditOutput {
    filepath: String,
    before: String,
    after: String,
    additions: usize,
    deletions: usize,
    diff: String,
}

// Similarity thresholds for block anchor fallback matching
const SINGLE_CANDIDATE_SIMILARITY_THRESHOLD: f64 = 0.0;
const MULTIPLE_CANDIDATES_SIMILARITY_THRESHOLD: f64 = 0.3;

/// Levenshtein distance algorithm implementation
fn levenshtein(a: &str, b: &str) -> usize {
    // Handle empty strings
    if a.is_empty() || b.is_empty() {
        return a.len().max(b.len());
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost,  // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Normalizes line endings to LF
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Creates a unified diff using the similar crate
fn create_two_files_patch(old_path: &str, new_path: &str, old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut output = String::new();
    
    output.push_str(&format!("--- {}\n", old_path));
    output.push_str(&format!("+++ {}\n", new_path));
    
    for (_idx, group) in diff.grouped_ops(3).iter().enumerate() {
        for op in group {
            let tag = op.tag();
            let old_range = op.old_range();
            let new_range = op.new_range();
            
            match tag {
                similar::DiffTag::Delete => {
                    output.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                        old_range.start + 1, old_range.len(), 
                        new_range.start + 1, new_range.len()));
                    for change in diff.iter_changes(op) {
                        if change.tag() == similar::ChangeTag::Delete {
                            output.push_str(&format!("-{}", change));
                        }
                    }
                }
                similar::DiffTag::Insert => {
                    output.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                        old_range.start + 1, old_range.len(), 
                        new_range.start + 1, new_range.len()));
                    for change in diff.iter_changes(op) {
                        if change.tag() == similar::ChangeTag::Insert {
                            output.push_str(&format!("+{}", change));
                        }
                    }
                }
                similar::DiffTag::Equal => {
                    // Skip equal sections in hunks
                }
                similar::DiffTag::Replace => {
                    // Handle replace operations (combination of delete and insert)
                    output.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                        old_range.start + 1, old_range.len(), 
                        new_range.start + 1, new_range.len()));
                    for change in diff.iter_changes(op) {
                        match change.tag() {
                            similar::ChangeTag::Delete => output.push_str(&format!("-{}", change)),
                            similar::ChangeTag::Insert => output.push_str(&format!("+{}", change)),
                            similar::ChangeTag::Equal => output.push_str(&format!(" {}", change)),
                        }
                    }
                }
            }
        }
    }
    
    output
}

/// Trims common whitespace from diff output
fn trim_diff(diff: &str) -> String {
    let lines: Vec<&str> = diff.lines().collect();
    let content_lines: Vec<&str> = lines
        .iter()
        .filter(|line| {
            (line.starts_with('+') || line.starts_with('-') || line.starts_with(' '))
                && !line.starts_with("---")
                && !line.starts_with("+++")
        })
        .copied()
        .collect();

    if content_lines.is_empty() {
        return diff.to_string();
    }

    let mut min_indent = usize::MAX;
    for line in &content_lines {
        let content = &line[1..];
        if !content.trim().is_empty() {
            if let Some(matched) = content.find(|c: char| !c.is_whitespace()) {
                min_indent = min_indent.min(matched);
            }
        }
    }

    if min_indent == usize::MAX || min_indent == 0 {
        return diff.to_string();
    }

    let trimmed_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            if (line.starts_with('+') || line.starts_with('-') || line.starts_with(' '))
                && !line.starts_with("---")
                && !line.starts_with("+++")
            {
                let prefix = line.chars().next().unwrap();
                let content = &line[1..];
                if content.len() >= min_indent {
                    format!("{}{}", prefix, &content[min_indent..])
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect();

    trimmed_lines.join("\n")
}

/// Simple exact string replacer
fn simple_replacer(content: &str, find: &str) -> Vec<String> {
    if content.contains(find) {
        vec![find.to_string()]
    } else {
        Vec::new()
    }
}

/// Line-trimmed replacer - matches lines ignoring leading/trailing whitespace
fn line_trimmed_replacer(content: &str, find: &str) -> Vec<String> {
    let original_lines: Vec<&str> = content.lines().collect();
    let mut search_lines: Vec<&str> = find.lines().collect();

    // Remove trailing empty line if present
    if search_lines.last() == Some(&"") {
        search_lines.pop();
    }

    for i in 0..=original_lines.len().saturating_sub(search_lines.len()) {
        let mut matches = true;

        for (j, search_line) in search_lines.iter().enumerate() {
            let original_trimmed = original_lines[i + j].trim();
            let search_trimmed = search_line.trim();

            if original_trimmed != search_trimmed {
                matches = false;
                break;
            }
        }

        if matches {
            // Calculate the actual match indices
            let mut match_start_index = 0;
            for k in 0..i {
                match_start_index += original_lines[k].len() + 1; // +1 for newline
            }

            let mut match_end_index = match_start_index;
            for k in 0..search_lines.len() {
                match_end_index += original_lines[i + k].len();
                if k < search_lines.len() - 1 {
                    match_end_index += 1; // Add newline except for last line
                }
            }

            return vec![content[match_start_index..match_end_index].to_string()];
        }
    }

    Vec::new()
}

/// Block anchor replacer - uses first and last lines as anchors with fuzzy middle matching
fn block_anchor_replacer(content: &str, find: &str) -> Vec<String> {
    let original_lines: Vec<&str> = content.lines().collect();
    let mut search_lines: Vec<&str> = find.lines().collect();

    if search_lines.len() < 3 {
        return Vec::new();
    }

    // Remove trailing empty line if present
    if search_lines.last() == Some(&"") {
        search_lines.pop();
    }

    let first_line_search = search_lines[0].trim();
    let last_line_search = search_lines[search_lines.len() - 1].trim();
    let search_block_size = search_lines.len();

    // Collect all candidate positions where both anchors match
    let mut candidates = Vec::new();
    for i in 0..original_lines.len() {
        if original_lines[i].trim() != first_line_search {
            continue;
        }

        // Look for the matching last line after this first line
        for j in (i + 2)..original_lines.len() {
            if original_lines[j].trim() == last_line_search {
                candidates.push((i, j));
                break; // Only match the first occurrence of the last line
            }
        }
    }

    // Return immediately if no candidates
    if candidates.is_empty() {
        return Vec::new();
    }

    // Handle single candidate scenario (using relaxed threshold)
    if candidates.len() == 1 {
        let (start_line, end_line) = candidates[0];
        let actual_block_size = end_line - start_line + 1;

        let mut similarity = 0.0;
        let lines_to_check = (search_block_size - 2).min(actual_block_size - 2); // Middle lines only

        if lines_to_check > 0 {
            for j in 1..(search_block_size - 1).min(actual_block_size - 1) {
                let original_line = original_lines[start_line + j].trim();
                let search_line = search_lines[j].trim();
                let max_len = original_line.len().max(search_line.len());
                if max_len == 0 {
                    continue;
                }
                let distance = levenshtein(original_line, search_line);
                similarity += (1.0 - distance as f64 / max_len as f64) / lines_to_check as f64;

                // Exit early when threshold is reached
                if similarity >= SINGLE_CANDIDATE_SIMILARITY_THRESHOLD {
                    break;
                }
            }
        } else {
            // No middle lines to compare, just accept based on anchors
            similarity = 1.0;
        }

        if similarity >= SINGLE_CANDIDATE_SIMILARITY_THRESHOLD {
            let mut match_start_index = 0;
            for k in 0..start_line {
                match_start_index += original_lines[k].len() + 1;
            }
            let mut match_end_index = match_start_index;
            for k in start_line..=end_line {
                match_end_index += original_lines[k].len();
                if k < end_line {
                    match_end_index += 1; // Add newline except for last line
                }
            }
            return vec![content[match_start_index..match_end_index].to_string()];
        }
        return Vec::new();
    }

    // Calculate similarity for multiple candidates
    let mut best_match: Option<(usize, usize)> = None;
    let mut max_similarity = -1.0;

    for &(start_line, end_line) in &candidates {
        let actual_block_size = end_line - start_line + 1;

        let mut similarity = 0.0;
        let lines_to_check = (search_block_size - 2).min(actual_block_size - 2); // Middle lines only

        if lines_to_check > 0 {
            for j in 1..(search_block_size - 1).min(actual_block_size - 1) {
                let original_line = original_lines[start_line + j].trim();
                let search_line = search_lines[j].trim();
                let max_len = original_line.len().max(search_line.len());
                if max_len == 0 {
                    continue;
                }
                let distance = levenshtein(original_line, search_line);
                similarity += 1.0 - distance as f64 / max_len as f64;
            }
            similarity /= lines_to_check as f64; // Average similarity
        } else {
            // No middle lines to compare, just accept based on anchors
            similarity = 1.0;
        }

        if similarity > max_similarity {
            max_similarity = similarity;
            best_match = Some((start_line, end_line));
        }
    }

    // Threshold judgment
    if max_similarity >= MULTIPLE_CANDIDATES_SIMILARITY_THRESHOLD {
        if let Some((start_line, end_line)) = best_match {
            let mut match_start_index = 0;
            for k in 0..start_line {
                match_start_index += original_lines[k].len() + 1;
            }
            let mut match_end_index = match_start_index;
            for k in start_line..=end_line {
                match_end_index += original_lines[k].len();
                if k < end_line {
                    match_end_index += 1;
                }
            }
            return vec![content[match_start_index..match_end_index].to_string()];
        }
    }

    Vec::new()
}

/// Whitespace-normalized replacer - collapses multiple spaces into single spaces
fn whitespace_normalized_replacer(content: &str, find: &str) -> Vec<String> {
    let normalize_whitespace = |text: &str| text.replace(char::is_whitespace, " ").trim().to_string();
    let normalized_find = normalize_whitespace(find);

    // Handle single line matches
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        if normalize_whitespace(line) == normalized_find {
            return vec![line.to_string()];
        } else {
            // Only check for substring matches if the full line doesn't match
            let normalized_line = normalize_whitespace(line);
            if normalized_line.contains(&normalized_find) {
                // Find the actual substring in the original line that matches
                let words: Vec<&str> = find.trim().split_whitespace().collect();
                if !words.is_empty() {
                    let pattern = words
                        .iter()
                        .map(|word| regex::escape(word))
                        .collect::<Vec<_>>()
                        .join("\\s+");
                    if let Ok(regex) = Regex::new(&pattern) {
                        if let Some(matched) = regex.find(line) {
                            return vec![matched.as_str().to_string()];
                        }
                    }
                }
            }
        }
    }

    // Handle multi-line matches
    let find_lines: Vec<&str> = find.lines().collect();
    if find_lines.len() > 1 {
        for i in 0..=lines.len().saturating_sub(find_lines.len()) {
            let block: String = lines[i..i + find_lines.len()].join("\n");
            if normalize_whitespace(&block) == normalized_find {
                return vec![block];
            }
        }
    }

    Vec::new()
}

/// Indentation-flexible replacer - ignores leading indentation
fn indentation_flexible_replacer(content: &str, find: &str) -> Vec<String> {
    let remove_indentation = |text: &str| {
        let lines: Vec<&str> = text.lines().collect();
        let non_empty_lines: Vec<&str> = lines.iter().filter(|line| !line.trim().is_empty()).copied().collect();
        if non_empty_lines.is_empty() {
            return text.to_string();
        }

        let min_indent = non_empty_lines
            .iter()
            .map(|line| {
                line.find(|c: char| !c.is_whitespace()).unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        lines
            .iter()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else {
                    line.chars().skip(min_indent).collect()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let normalized_find = remove_indentation(find);
    let content_lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find.lines().collect();

    for i in 0..=content_lines.len().saturating_sub(find_lines.len()) {
        let block: String = content_lines[i..i + find_lines.len()].join("\n");
        if remove_indentation(&block) == normalized_find {
            return vec![block];
        }
    }

    Vec::new()
}

/// Escape-normalized replacer - handles escaped characters
fn escape_normalized_replacer(content: &str, find: &str) -> Vec<String> {
    let unescape_string = |str_: &str| {
        str_.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\'", "'")
            .replace("\\\"", "\"")
            .replace("\\`", "`")
            .replace("\\\\", "\\")
            .replace("\\\n", "\n")
            .replace("\\$", "$")
    };

    let unescaped_find = unescape_string(find);

    // Try direct match with unescaped find string
    if content.contains(&unescaped_find) {
        return vec![unescaped_find];
    }

    // Also try finding escaped versions in content that match unescaped find
    let lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = unescaped_find.lines().collect();

    for i in 0..=lines.len().saturating_sub(find_lines.len()) {
        let block: String = lines[i..i + find_lines.len()].join("\n");
        let unescaped_block = unescape_string(&block);

        if unescaped_block == unescaped_find {
            return vec![block];
        }
    }

    Vec::new()
}

/// Multi-occurrence replacer - finds all exact matches
fn multi_occurrence_replacer(content: &str, find: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let mut start_index = 0;

    while let Some(index) = content[start_index..].find(find) {
        let absolute_index = start_index + index;
        matches.push(find.to_string());
        start_index = absolute_index + find.len();
    }

    matches
}

/// Trimmed boundary replacer - tries trimmed versions
fn trimmed_boundary_replacer(content: &str, find: &str) -> Vec<String> {
    let trimmed_find = find.trim();

    if trimmed_find == find {
        // Already trimmed, no point in trying
        return Vec::new();
    }

    let mut results = Vec::new();

    // Try to find the trimmed version
    if content.contains(trimmed_find) {
        results.push(trimmed_find.to_string());
    }

    // Also try finding blocks where trimmed content matches
    let lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find.lines().collect();

    for i in 0..=lines.len().saturating_sub(find_lines.len()) {
        let block: String = lines[i..i + find_lines.len()].join("\n");

        if block.trim() == trimmed_find {
            results.push(block);
        }
    }

    results
}

/// Context-aware replacer - uses first and last lines as context anchors
fn context_aware_replacer(content: &str, find: &str) -> Vec<String> {
    let mut find_lines: Vec<&str> = find.lines().collect();
    if find_lines.len() < 3 {
        // Need at least 3 lines to have meaningful context
        return Vec::new();
    }

    // Remove trailing empty line if present
    if find_lines.last() == Some(&"") {
        find_lines.pop();
    }

    let content_lines: Vec<&str> = content.lines().collect();

    // Extract first and last lines as context anchors
    let first_line = find_lines[0].trim();
    let last_line = find_lines[find_lines.len() - 1].trim();

    // Find blocks that start and end with the context anchors
    for i in 0..content_lines.len() {
        if content_lines[i].trim() != first_line {
            continue;
        }

        // Look for the matching last line
        for j in (i + 2)..content_lines.len() {
            if content_lines[j].trim() == last_line {
                // Found a potential context block
                let block_lines = &content_lines[i..=j];
                let block = block_lines.join("\n");

                // Check if the middle content has reasonable similarity
                // (simple heuristic: at least 50% of non-empty lines should match when trimmed)
                if block_lines.len() == find_lines.len() {
                    let mut matching_lines = 0;
                    let mut total_non_empty_lines = 0;

                    for k in 1..block_lines.len() - 1 {
                        let block_line = block_lines[k].trim();
                        let find_line = find_lines[k].trim();

                        if !block_line.is_empty() || !find_line.is_empty() {
                            total_non_empty_lines += 1;
                            if block_line == find_line {
                                matching_lines += 1;
                            }
                        }
                    }

                    if total_non_empty_lines == 0 || (matching_lines as f64 / total_non_empty_lines as f64) >= 0.5 {
                        return vec![block];
                    }
                }
                break;
            }
        }
    }

    Vec::new()
}

/// Main replace function that tries all replacers in cascade
fn replace(content: &str, old_string: &str, new_string: &str, replace_all: bool) -> Result<String, String> {
    if old_string == new_string {
        return Err("oldString and newString must be different".to_string());
    }

    let mut not_found = true;

    // Try all replacers in order
    let replacers = [
        simple_replacer,
        line_trimmed_replacer,
        block_anchor_replacer,
        whitespace_normalized_replacer,
        indentation_flexible_replacer,
        escape_normalized_replacer,
        trimmed_boundary_replacer,
        context_aware_replacer,
        multi_occurrence_replacer,
    ];

    for replacer in &replacers {
        for search in replacer(content, old_string) {
            let index = content.find(&search);
            if index.is_none() {
                continue;
            }
            not_found = false;

            if replace_all {
                return Ok(content.replace(&search, new_string));
            }

            let last_index = content.rfind(&search);
            if index != last_index {
                continue; // Multiple matches, try to be more specific
            }

            let index = index.unwrap();
            return Ok(content[..index].to_string() + new_string + &content[index + search.len()..]);
        }
    }

    if not_found {
        return Err("oldString not found in content".to_string());
    }
    Err("Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.".to_string())
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        r#"Performs exact string replacements in files.

Usage:
- You must use your `Read` tool at least once in the conversation before editing. This tool will error if you attempt an edit without reading the file.
- When editing text from Read tool output, ensure you preserve the exact indentation (tabs/spaces) as it appears AFTER the line number prefix. The line number prefix format is: spaces + line number + tab. Everything after that tab is the actual file content to match. Never include any part of the line number prefix in the oldString or newString.
- ALWAYS prefer editing existing files in the codebase. NEVER write new files unless explicitly required.
- Only use emojis if the user explicitly requests it. Avoid adding emojis to files unless asked.
- The edit will FAIL if `oldString` is not found in the file with an error "oldString not found in content".
- The edit will FAIL if `oldString` is found multiple times in the file with an error "oldString found multiple times and requires more code context to uniquely identify the intended match". Either provide a larger string with more surrounding context to make it unique or use `replaceAll` to change every instance of `oldString`.
- Use `replaceAll` for replacing and renaming strings across the file. This parameter is useful if you want to rename a variable for instance."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "The absolute path to the file to modify"
                },
                "oldString": {
                    "type": "string",
                    "description": "The text to replace"
                },
                "newString": {
                    "type": "string",
                    "description": "The text to replace it with (must be different from oldString)"
                },
                "replaceAll": {
                    "type": "boolean",
                    "description": "Replace all occurrences of oldString (default false)",
                    "default": false
                }
            },
            "required": ["filePath", "oldString", "newString"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let edit_input: EditInput = match serde_json::from_value(input) {
            Ok(i) => i,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Invalid input: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        if edit_input.old_string == edit_input.new_string {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("oldString and newString must be different".to_string()),
                metadata: json!({}),
            };
        }

        // Validate file path is absolute
        let file_path = if Path::new(&edit_input.file_path).is_absolute() {
            edit_input.file_path.clone()
        } else {
            // For relative paths, join with working directory
            ctx.working_dir.join(&edit_input.file_path)
                .to_string_lossy()
                .to_string()
        };

        // Check if file exists
        let metadata = match fs::metadata(&file_path).await {
            Ok(m) => m,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("File {} not found: {}", file_path, e)),
                    metadata: json!({
                        "filepath": file_path,
                    }),
                };
            }
        };

        if metadata.is_dir() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Path is a directory, not a file: {}", file_path)),
                metadata: json!({
                    "filepath": file_path,
                }),
            };
        }

        // Read file content
        let content_old = match fs::read_to_string(&file_path).await {
            Ok(c) => c,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                    metadata: json!({
                        "filepath": file_path,
                    }),
                };
            }
        };

        // Handle empty oldString (create new file or append)
        if edit_input.old_string.is_empty() {
            let diff = trim_diff(&create_two_files_patch(
                &file_path,
                &file_path,
                "",
                &edit_input.new_string,
            ));

            if let Err(e) = fs::write(&file_path, &edit_input.new_string).await {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to write file: {}", e)),
                    metadata: json!({
                        "filepath": file_path,
                    }),
                };
            }

            let output = EditOutput {
                filepath: file_path.clone(),
                before: content_old,
                after: edit_input.new_string.clone(),
                additions: edit_input.new_string.lines().count(),
                deletions: 0,
                diff,
            };

            return ToolResult {
                status: ToolStatus::Completed,
                output: serde_json::to_string(&output).unwrap_or_default(),
                error: None,
                metadata: json!({
                    "filepath": file_path,
                    "additions": output.additions,
                    "deletions": output.deletions,
                    "size_delta": output.after.len() as i64 - output.before.len() as i64,
                }),
            };
        }

        // Perform replacement using fuzzy matching
        let content_new = match replace(&content_old, &edit_input.old_string, &edit_input.new_string, edit_input.replace_all) {
            Ok(result) => result,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(e),
                    metadata: json!({
                        "filepath": file_path,
                        "old_string_length": edit_input.old_string.len(),
                    }),
                };
            }
        };

        // Generate diff
        let diff = trim_diff(&create_two_files_patch(
            &file_path,
            &file_path,
            &normalize_line_endings(&content_old),
            &normalize_line_endings(&content_new),
        ));

        // Write the updated content
        if let Err(e) = fs::write(&file_path, &content_new).await {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to write file: {}", e)),
                metadata: json!({
                    "filepath": file_path,
                }),
            };
        }

        // Calculate additions and deletions
        let (mut additions, mut deletions) = (0, 0);
        let diff_obj = TextDiff::from_lines(&content_old, &content_new);
        for change in diff_obj.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => additions += 1,
                ChangeTag::Delete => deletions += 1,
                ChangeTag::Equal => {}
            }
        }

        let output = EditOutput {
            filepath: file_path.clone(),
            before: content_old,
            after: content_new,
            additions,
            deletions,
            diff,
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "filepath": file_path,
                "additions": output.additions,
                "deletions": output.deletions,
                "size_delta": output.after.len() as i64 - output.before.len() as i64,
            }),
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;
    use tokio::fs;

    #[tokio::test]
    async fn test_edit_simple_replacement() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Hello World\nHello Rust";
        fs::write(&temp_file, content).await.unwrap();

        let tool = EditTool;
        let input = json!({
            "filePath": temp_file.path().to_str().unwrap(),
            "oldString": "Hello World",
            "newString": "Hello Crow"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let new_content = fs::read_to_string(&temp_file).await.unwrap();
        assert_eq!(new_content, "Hello Crow\nHello Rust");
    }

    #[tokio::test]
    async fn test_edit_fuzzy_matching() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "    let x = 5;\n    let y = 10;";
        fs::write(&temp_file, content).await.unwrap();

        let tool = EditTool;
        // Try to replace with different indentation (should work with fuzzy matching)
        let input = json!({
            "filePath": temp_file.path().to_str().unwrap(),
            "oldString": "let x = 5;",
            "newString": "let x = 42;"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let new_content = fs::read_to_string(&temp_file).await.unwrap();
        assert_eq!(new_content, "    let x = 42;\n    let y = 10;");
    }

    #[tokio::test]
    async fn test_edit_replace_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "foo bar\nfoo baz";
        fs::write(&temp_file, content).await.unwrap();

        let tool = EditTool;
        let input = json!({
            "filePath": temp_file.path().to_str().unwrap(),
            "oldString": "foo",
            "newString": "bar",
            "replaceAll": true
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: EditOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.additions, 0);
        assert_eq!(output.deletions, 0);

        let new_content = fs::read_to_string(&temp_file).await.unwrap();
        assert_eq!(new_content, "bar bar\nbar baz");
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
    }

    #[test]
    fn test_trim_diff() {
        let diff = "--- a.txt\n+++ b.txt\n@@ -1,3 +1,3 @@\n-    old line\n+    new line\n   unchanged";
        let trimmed = trim_diff(diff);
        // Should trim common indentation from diff content lines
        assert!(trimmed.contains("-old line"));
        assert!(trimmed.contains("+new line"));
        assert!(trimmed.contains(" unchanged"));
    }

    #[test]
    fn test_simple_replacer() {
        let content = "hello world";
        let find = "hello";
        let matches = simple_replacer(content, find);
        assert_eq!(matches, vec!["hello"]);
    }

    #[test]
    fn test_line_trimmed_replacer() {
        let content = "    hello world\n    foo bar";
        let find = "hello world\nfoo bar";
        let matches = line_trimmed_replacer(content, find);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].contains("hello world"));
        assert!(matches[0].contains("foo bar"));
    }
}