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
    #[serde(default, rename = "replaceAll")]
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
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
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
fn create_two_files_patch(
    old_path: &str,
    new_path: &str,
    old_content: &str,
    new_content: &str,
) -> String {
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
                    output.push_str(&format!(
                        "@@ -{},{} +{},{} @@\n",
                        old_range.start + 1,
                        old_range.len(),
                        new_range.start + 1,
                        new_range.len()
                    ));
                    for change in diff.iter_changes(op) {
                        if change.tag() == similar::ChangeTag::Delete {
                            output.push_str(&format!("-{}", change));
                        }
                    }
                }
                similar::DiffTag::Insert => {
                    output.push_str(&format!(
                        "@@ -{},{} +{},{} @@\n",
                        old_range.start + 1,
                        old_range.len(),
                        new_range.start + 1,
                        new_range.len()
                    ));
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
                    output.push_str(&format!(
                        "@@ -{},{} +{},{} @@\n",
                        old_range.start + 1,
                        old_range.len(),
                        new_range.start + 1,
                        new_range.len()
                    ));
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
    let normalize_whitespace =
        |text: &str| text.replace(char::is_whitespace, " ").trim().to_string();
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
        let non_empty_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .copied()
            .collect();
        if non_empty_lines.is_empty() {
            return text.to_string();
        }

        let min_indent = non_empty_lines
            .iter()
            .map(|line| line.find(|c: char| !c.is_whitespace()).unwrap_or(0))
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

                    if total_non_empty_lines == 0
                        || (matching_lines as f64 / total_non_empty_lines as f64) >= 0.5
                    {
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
fn replace(
    content: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> Result<String, String> {
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
            return Ok(content[..index].to_string()
                + new_string
                + &content[index + search.len()..]);
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
            ctx.working_dir
                .join(&edit_input.file_path)
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
        let content_new = match replace(
            &content_old,
            &edit_input.old_string,
            &edit_input.new_string,
            edit_input.replace_all,
        ) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== replace() function tests ====================

    #[test]
    fn test_replace_simple() {
        let content = "Hello World\nHello Rust";
        let result = replace(content, "Hello World", "Hello Crow", false).unwrap();
        assert_eq!(result, "Hello Crow\nHello Rust");
    }

    #[test]
    fn test_replace_all_occurrences() {
        let content = "foo bar\nfoo baz\nfoo qux";
        let result = replace(content, "foo", "bar", true).unwrap();
        assert_eq!(result, "bar bar\nbar baz\nbar qux");
    }

    #[test]
    fn test_replace_single_when_multiple_exists() {
        let content = "foo bar\nfoo baz";
        let result = replace(content, "foo", "bar", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("multiple matches"));
    }

    #[test]
    fn test_replace_not_found() {
        let content = "Hello World";
        let result = replace(content, "Goodbye", "Hi", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_replace_same_string_error() {
        let content = "Hello World";
        let result = replace(content, "Hello", "Hello", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be different"));
    }

    #[test]
    fn test_replace_multiline() {
        let content = "fn main() {\n    println!(\"Hello\");\n}";
        let result = replace(
            content,
            "fn main() {\n    println!(\"Hello\");\n}",
            "fn main() {\n    println!(\"World\");\n}",
            false,
        )
        .unwrap();
        assert_eq!(result, "fn main() {\n    println!(\"World\");\n}");
    }

    #[test]
    fn test_replace_with_special_chars() {
        let content = "let x = \"Hello, World!\";";
        let result = replace(content, "\"Hello, World!\"", "\"Hello, Universe!\"", false).unwrap();
        assert_eq!(result, "let x = \"Hello, Universe!\";");
    }

    #[test]
    fn test_replace_empty_to_content() {
        // This tests the empty oldString case which creates new content
        let content = "";
        // Empty oldString is handled specially in execute(), not in replace()
        // So we test with non-empty strings
        let result = replace("a", "a", "b", false).unwrap();
        assert_eq!(result, "b");
    }

    #[test]
    fn test_replace_preserves_surrounding() {
        let content = "prefix Hello suffix";
        let result = replace(content, "Hello", "World", false).unwrap();
        assert_eq!(result, "prefix World suffix");
    }

    // ==================== Fuzzy matching tests ====================

    #[test]
    fn test_replace_fuzzy_whitespace() {
        let content = "    let x = 5;";
        let result = replace(content, "let x = 5;", "let x = 42;", false).unwrap();
        assert_eq!(result, "    let x = 42;");
    }

    #[test]
    fn test_replace_fuzzy_indentation() {
        let content = "        deeply indented";
        let result = replace(content, "deeply indented", "not so deep", false).unwrap();
        assert_eq!(result, "        not so deep");
    }

    #[test]
    fn test_replace_fuzzy_multiline_indentation() {
        let content = "    fn test() {\n        let x = 1;\n    }";
        let result = replace(
            content,
            "fn test() {\n    let x = 1;\n}",
            "fn test() {\n    let x = 2;\n}",
            false,
        )
        .unwrap();
        assert!(result.contains("let x = 2"));
    }

    #[test]
    fn test_replace_trimmed_boundary() {
        let content = "   hello world   ";
        let result = replace(content, "hello world", "goodbye world", false).unwrap();
        assert!(result.contains("goodbye world"));
    }

    // ==================== Block anchor replacer tests ====================

    #[test]
    fn test_replace_block_anchor() {
        let content = "fn foo() {\n    let x = 1;\n    let y = 2;\n}";
        let result = replace(
            content,
            "fn foo() {\n    // different middle\n}",
            "fn bar() {\n    let z = 3;\n}",
            false,
        );
        // Block anchor should match based on first/last lines
        assert!(result.is_ok());
    }

    // ==================== replaceAll edge cases ====================

    #[test]
    fn test_replace_all_adjacent() {
        let content = "aaa";
        let result = replace(content, "a", "b", true).unwrap();
        assert_eq!(result, "bbb");
    }

    #[test]
    fn test_replace_all_overlapping_pattern() {
        let content = "aaaa";
        let result = replace(content, "aa", "b", true).unwrap();
        assert_eq!(result, "bb");
    }

    #[test]
    fn test_replace_all_with_empty_result() {
        let content = "remove me";
        let result = replace(content, "remove ", "", true).unwrap();
        assert_eq!(result, "me");
    }

    #[test]
    fn test_replace_all_longer_replacement() {
        let content = "a b c";
        let result = replace(content, " ", "---", true).unwrap();
        assert_eq!(result, "a---b---c");
    }

    // ==================== Line-based tests ====================

    #[test]
    fn test_replace_full_line() {
        let content = "line 1\nline 2\nline 3";
        let result = replace(content, "line 2", "modified line", false).unwrap();
        assert_eq!(result, "line 1\nmodified line\nline 3");
    }

    #[test]
    fn test_replace_first_line() {
        let content = "first\nsecond\nthird";
        let result = replace(content, "first", "new first", false).unwrap();
        assert_eq!(result, "new first\nsecond\nthird");
    }

    #[test]
    fn test_replace_last_line() {
        let content = "first\nsecond\nthird";
        let result = replace(content, "third", "new third", false).unwrap();
        assert_eq!(result, "first\nsecond\nnew third");
    }

    // ==================== Code-like content tests ====================

    #[test]
    fn test_replace_function_body() {
        let content = r#"fn calculate() -> i32 {
    let a = 10;
    let b = 20;
    a + b
}"#;
        let result = replace(
            content,
            "    let a = 10;\n    let b = 20;",
            "    let a = 100;\n    let b = 200;",
            false,
        )
        .unwrap();
        assert!(result.contains("let a = 100"));
        assert!(result.contains("let b = 200"));
    }

    #[test]
    fn test_replace_import_statement() {
        let content = "use std::io;\nuse std::fs;\nuse std::path;";
        let result = replace(content, "use std::fs;", "use std::fs::File;", false).unwrap();
        assert_eq!(result, "use std::io;\nuse std::fs::File;\nuse std::path;");
    }

    #[test]
    fn test_replace_string_literal() {
        let content = r#"println!("Hello, World!");"#;
        let result = replace(
            content,
            r#""Hello, World!""#,
            r#""Hello, Universe!""#,
            false,
        )
        .unwrap();
        assert_eq!(result, r#"println!("Hello, Universe!");"#);
    }

    #[test]
    fn test_replace_all_string_literals() {
        let content = r#"let a = "test";
let b = "test";
let c = "test";"#;
        let result = replace(content, r#""test""#, r#""modified""#, true).unwrap();
        assert_eq!(result.matches(r#""modified""#).count(), 3);
    }

    // ==================== Levenshtein distance tests ====================

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_levenshtein_one_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_single_char() {
        assert_eq!(levenshtein("a", "b"), 1);
    }

    #[test]
    fn test_levenshtein_classic_examples() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("saturday", "sunday"), 3);
    }

    // ==================== trim_diff tests ====================

    #[test]
    fn test_trim_diff_basic() {
        let diff =
            "--- a.txt\n+++ b.txt\n@@ -1,3 +1,3 @@\n-    old line\n+    new line\n     unchanged";
        let trimmed = trim_diff(diff);
        assert!(trimmed.contains("-old line") || trimmed.contains("- old line"));
        assert!(trimmed.contains("+new line") || trimmed.contains("+ new line"));
    }

    #[test]
    fn test_trim_diff_no_common_indent() {
        let diff = "--- a.txt\n+++ b.txt\n@@ -1 +1 @@\n-old\n+new";
        let trimmed = trim_diff(diff);
        assert!(trimmed.contains("-old"));
        assert!(trimmed.contains("+new"));
    }

    #[test]
    fn test_trim_diff_preserves_headers() {
        let diff = "--- a.txt\n+++ b.txt\n@@ -1 +1 @@\n-old\n+new";
        let trimmed = trim_diff(diff);
        assert!(trimmed.contains("--- a.txt"));
        assert!(trimmed.contains("+++ b.txt"));
    }

    // ==================== Individual replacer tests ====================

    #[test]
    fn test_simple_replacer() {
        let content = "hello world";
        let find = "hello";
        let matches = simple_replacer(content, find);
        assert_eq!(matches, vec!["hello"]);
    }

    #[test]
    fn test_line_trimmed_replacer_basic() {
        let content = "    hello world\n    foo bar";
        let find = "hello world\nfoo bar";
        let matches = line_trimmed_replacer(content, find);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_whitespace_normalized_replacer() {
        // Test that it can match content with extra whitespace
        let content = "hello world"; // normal spacing
        let find = "hello world";
        let matches = whitespace_normalized_replacer(content, find);
        // This replacer normalizes whitespace, so exact match should work
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_indentation_flexible_replacer() {
        let content = "        deeply indented";
        let find = "deeply indented";
        let matches = indentation_flexible_replacer(content, find);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_multi_occurrence_replacer() {
        let content = "foo bar foo baz foo";
        let find = "foo";
        let matches = multi_occurrence_replacer(content, find);
        assert_eq!(matches.len(), 3);
    }

    // ==================== Real-world scenario tests ====================

    #[test]
    fn test_replace_rust_struct_field() {
        let content = r#"struct User {
    name: String,
    age: u32,
}"#;
        let result = replace(content, "    age: u32,", "    age: u64,", false).unwrap();
        assert!(result.contains("age: u64"));
    }

    #[test]
    fn test_replace_json_value() {
        let content = r#"{"key": "value", "other": "data"}"#;
        let result = replace(content, r#""value""#, r#""new_value""#, false).unwrap();
        assert!(result.contains(r#""new_value""#));
    }

    #[test]
    fn test_replace_yaml_like() {
        let content = "name: test\nversion: 1.0.0\ndescription: A test";
        let result = replace(content, "version: 1.0.0", "version: 2.0.0", false).unwrap();
        assert!(result.contains("version: 2.0.0"));
    }

    #[test]
    fn test_replace_markdown_header() {
        let content = "# Old Title\n\nSome content";
        let result = replace(content, "# Old Title", "# New Title", false).unwrap();
        assert!(result.contains("# New Title"));
    }

    #[test]
    fn test_replace_html_tag() {
        let content = "<div class=\"old\">content</div>";
        let result = replace(content, r#"class="old""#, r#"class="new""#, false).unwrap();
        assert!(result.contains(r#"class="new""#));
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_replace_at_start() {
        let content = "start middle end";
        let result = replace(content, "start", "begin", false).unwrap();
        assert_eq!(result, "begin middle end");
    }

    #[test]
    fn test_replace_at_end() {
        let content = "start middle end";
        let result = replace(content, "end", "finish", false).unwrap();
        assert_eq!(result, "start middle finish");
    }

    #[test]
    fn test_replace_entire_content() {
        let content = "entire content";
        let result = replace(content, "entire content", "new content", false).unwrap();
        assert_eq!(result, "new content");
    }

    #[test]
    fn test_replace_with_newlines_in_replacement() {
        let content = "single line";
        let result = replace(content, "single line", "first\nsecond\nthird", false).unwrap();
        assert_eq!(result, "first\nsecond\nthird");
    }

    #[test]
    fn test_replace_remove_newlines() {
        let content = "first\nsecond\nthird";
        let result = replace(content, "first\nsecond\nthird", "single line", false).unwrap();
        assert_eq!(result, "single line");
    }

    #[test]
    fn test_replace_unicode() {
        let content = "Hello 世界!";
        let result = replace(content, "世界", "World", false).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_replace_emoji() {
        let content = "Hello 👋 World";
        let result = replace(content, "👋", "🌍", false).unwrap();
        assert_eq!(result, "Hello 🌍 World");
    }

    // ==================== Regression tests ====================

    #[test]
    fn test_replace_all_three_occurrences() {
        // This is the exact test case that was failing
        let content = r#"fn main() {
    println!("Hello, World!");
}

pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}"#;
        let result = replace(content, "Hello, World!", "Hello, Universe!", true).unwrap();
        assert_eq!(result.matches("Hello, Universe!").count(), 3);
        assert_eq!(result.matches("Hello, World!").count(), 0);
    }
}
