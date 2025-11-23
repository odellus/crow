//! Language ID mappings for LSP

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Map file extensions to LSP language IDs
pub static LANGUAGE_EXTENSIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // TypeScript/JavaScript
    m.insert(".ts", "typescript");
    m.insert(".tsx", "typescriptreact");
    m.insert(".js", "javascript");
    m.insert(".jsx", "javascriptreact");
    m.insert(".mjs", "javascript");
    m.insert(".cjs", "javascript");
    m.insert(".mts", "typescript");
    m.insert(".cts", "typescript");

    // Rust
    m.insert(".rs", "rust");

    // Python
    m.insert(".py", "python");
    m.insert(".pyi", "python");

    // Go
    m.insert(".go", "go");

    // C/C++
    m.insert(".c", "c");
    m.insert(".h", "c");
    m.insert(".cpp", "cpp");
    m.insert(".cc", "cpp");
    m.insert(".cxx", "cpp");
    m.insert(".hpp", "cpp");
    m.insert(".hh", "cpp");
    m.insert(".hxx", "cpp");

    // Java
    m.insert(".java", "java");

    // Ruby
    m.insert(".rb", "ruby");
    m.insert(".rake", "ruby");
    m.insert(".gemspec", "ruby");

    // PHP
    m.insert(".php", "php");

    // Swift
    m.insert(".swift", "swift");

    // Zig
    m.insert(".zig", "zig");
    m.insert(".zon", "zig");

    // Elixir
    m.insert(".ex", "elixir");
    m.insert(".exs", "elixir");

    // Lua
    m.insert(".lua", "lua");

    // YAML
    m.insert(".yaml", "yaml");
    m.insert(".yml", "yaml");

    // Vue/Svelte/Astro
    m.insert(".vue", "vue");
    m.insert(".svelte", "svelte");
    m.insert(".astro", "astro");

    // C#
    m.insert(".cs", "csharp");

    // Shell
    m.insert(".sh", "shellscript");
    m.insert(".bash", "shellscript");
    m.insert(".zsh", "shellscript");

    // Markdown
    m.insert(".md", "markdown");
    m.insert(".markdown", "markdown");

    // JSON
    m.insert(".json", "json");
    m.insert(".jsonc", "jsonc");

    // TOML
    m.insert(".toml", "toml");

    // HTML/CSS
    m.insert(".html", "html");
    m.insert(".htm", "html");
    m.insert(".css", "css");
    m.insert(".scss", "scss");
    m.insert(".less", "less");

    m
});

/// Get language ID for a file extension
pub fn get_language_id(extension: &str) -> &'static str {
    LANGUAGE_EXTENSIONS
        .get(extension)
        .copied()
        .unwrap_or("plaintext")
}
