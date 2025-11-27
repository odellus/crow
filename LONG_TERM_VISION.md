# Porting Zed Editor Core to Bevy

A comprehensive plan for extracting Zed's proven text editing kernel and rebuilding it on Bevy instead of GPUI.

## Executive Summary

**Goal:** Create a minimal, fast, Rust-native code editor by combining:
- Zed's battle-tested text buffer and language support
- Bevy's mature rendering and ECS architecture
- Avoid maintaining a custom UI framework (GPUI)

**Timeline:** 6-8 weeks for MVP (basic editing + syntax highlighting)

---

## Why Port From GPUI to Bevy?

### The GPUI Problem

**What GPUI is:**
- Custom UI framework built by Zed team specifically for Zed
- Direct GPU rendering (bypasses traditional widget toolkits)
- Very fast, very tailored to text editing
- Small community (basically only Zed uses it)

**Why it's problematic for us:**
- **Limited community**: Few people know GPUI outside Zed contributors
- **Sparse documentation**: Built for internal use, docs are minimal
- **Tight coupling**: Hard to extract just the parts we want
- **Maintenance burden**: Would need to maintain a fork of a custom framework
- **Learning curve**: Custom concepts, custom APIs, no transferable skills

### The Bevy Advantage

**What Bevy is:**
- Mature game engine with focus on data-driven architecture
- ECS (Entity Component System) for managing state
- Excellent rendering pipeline (2D/3D)
- `bevy_ui` for UI layout (flexbox-based)
- Active community, excellent documentation
- Plugin ecosystem

**Why it works for us:**
- **Large community**: Thousands of developers, active Discord, tutorials everywhere
- **Great docs**: Bevy book, examples, API docs are all excellent
- **Proven in production**: Used in shipped games and applications
- **Plugin ecosystem**: bevy_egui, bevy_text, bevy_prototype_lyon, etc.
- **Transferable skills**: Learning Bevy = learning game dev patterns
- **Active development**: Frequent releases, rapid feature additions

**The tradeoff:**
- GPUI is more optimized for text editing specifically
- Bevy is more general-purpose
- But Bevy's flexibility + community support > GPUI's specialization

---

## What We're Actually Porting

### Framework-Agnostic Zed Components (USE AS-IS)

These crates from Zed don't depend on GPUI and can be used directly:

```
zed/crates/
├── rope/              ✅ Text buffer using rope data structure
│                         - Efficient insertions/deletions
│                         - O(log n) operations
│                         - Undo/redo support
│
├── sum_tree/          ✅ Persistent B-tree (rope dependency)
│                         - Immutable data structure
│                         - Efficient cloning
│
├── collections/       ✅ Utility data structures
│                         - HashMap, BTreeMap wrappers
│                         - Small optimizations
│
├── text/              ✅ Text buffer abstraction
│                         - Selection ranges
│                         - Multi-cursor support
│                         - Anchors (stable positions)
│
└── language/          ✅ Language support
                          - Tree-sitter integration
                          - LSP client
                          - Syntax highlighting queries
```

**Key insight:** These are pure Rust libraries with minimal dependencies. We can use them directly without modification.

### GPUI-Specific Parts (REWRITE IN BEVY)

These parts are tightly coupled to GPUI and need Bevy equivalents:

| GPUI Component | Bevy Equivalent | Notes |
|----------------|-----------------|-------|
| Text rendering | `bevy_text` or custom shader | GPUI does GPU glyphs, Bevy can too |
| Input handling | `bevy_input` | Keyboard, mouse events |
| Layout system | `bevy_ui` | Flexbox-based layout |
| Window management | `bevy_window` + `bevy_winit` | Multi-window support |
| Asset loading | `bevy_asset` | Fonts, icons, etc. |

---

## Architecture Overview

### Bevy ECS Approach

Bevy uses Entity Component System (ECS) architecture:

```rust
// ENTITIES: Unique IDs (just numbers)
let editor = commands.spawn(EditorBundle::default()).id();

// COMPONENTS: Pure data (structs)
#[derive(Component)]
struct TextBuffer {
    rope: Rope,              // From zed-rope
    syntax: TreeSitterTree,  // From tree-sitter
}

#[derive(Component)]
struct Cursor {
    line: usize,
    column: usize,
}

// SYSTEMS: Functions that operate on components
fn handle_typing(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextBuffer, &mut Cursor)>,
) {
    // Logic here
}

// RESOURCES: Global state
#[derive(Resource)]
struct LspClient {
    // ...
}
```

### High-Level Structure

```
crow-editor-bevy/
├── src/
│   ├── main.rs              # Bevy app setup
│   ├── text/
│   │   ├── buffer.rs        # TextBuffer component using zed-rope
│   │   ├── cursor.rs        # Cursor/selection components
│   │   └── render.rs        # Text rendering system
│   ├── syntax/
│   │   ├── highlight.rs     # Tree-sitter integration
│   │   └── languages.rs     # Language definitions
│   ├── lsp/
│   │   ├── client.rs        # LSP client (from Zed)
│   │   └── handlers.rs      # LSP response handlers
│   ├── input/
│   │   ├── keyboard.rs      # Keyboard handling
│   │   └── mouse.rs         # Mouse/scroll handling
│   └── ui/
│       ├── editor.rs        # Main editor view
│       └── gutter.rs        # Line numbers, etc.
└── Cargo.toml
```

---

## Phase-by-Phase Implementation Plan

### Phase 0: Project Setup (Day 1)

**Goal:** Get Bevy running with a window

```bash
cargo new crow-editor-bevy
cd crow-editor-bevy
```

**Cargo.toml:**
```toml
[package]
name = "crow-editor-bevy"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.14"

# Zed crates (use directly from their repo)
rope = { git = "https://github.com/zed-industries/zed", package = "rope" }
sum_tree = { git = "https://github.com/zed-industries/zed", package = "sum_tree" }
collections = { git = "https://github.com/zed-industries/zed", package = "collections" }

# Tree-sitter for syntax
tree-sitter = "0.20"
tree-sitter-rust = "0.20"

# Other utilities
anyhow = "1.0"
```

**main.rs:**
```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
```

**Success criteria:** Window opens, black screen

---

### Phase 1: Text Buffer Integration (Week 1)

**Goal:** Display static text using Zed's rope

**Components:**
```rust
use bevy::prelude::*;
use rope::Rope;

#[derive(Component)]
struct TextBuffer {
    rope: Rope,
}

#[derive(Component)]
struct EditorView {
    scroll_offset: f32,
    line_height: f32,
}
```

**Systems:**
```rust
fn setup_editor(mut commands: Commands) {
    let initial_text = r#"
fn main() {
    println!("Hello from Bevy + Zed!");
}
"#;
    
    commands.spawn((
        TextBuffer {
            rope: Rope::from(initial_text),
        },
        EditorView {
            scroll_offset: 0.0,
            line_height: 20.0,
        },
    ));
}

fn render_text(
    query: Query<(&TextBuffer, &EditorView)>,
    mut gizmos: Gizmos,  // For debug rendering
) {
    for (buffer, view) in query.iter() {
        // For now: just prove we can iterate over lines
        for (i, line) in buffer.rope.lines().enumerate() {
            let y = i as f32 * view.line_height - view.scroll_offset;
            // TODO: Actually render text (Phase 1b)
            println!("Line {}: {}", i, line);
        }
    }
}
```

**Phase 1b: Actual text rendering**

Options:
1. **Use bevy_text:** Built-in, simple, not optimized for code
2. **Use bevy_egui:** Immediate mode GUI with good text support
3. **Custom shader:** Full control, more work

**Recommendation for MVP: bevy_text**

```rust
fn render_text_with_bevy(
    query: Query<(&TextBuffer, &EditorView)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (buffer, view) in query.iter() {
        let font = asset_server.load("fonts/FiraCode-Regular.ttf");
        
        for (i, line) in buffer.rope.lines().enumerate() {
            let y = i as f32 * view.line_height - view.scroll_offset;
            
            commands.spawn(Text2dBundle {
                text: Text::from_section(
                    line.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_xyz(0.0, y, 0.0),
                ..default()
            });
        }
    }
}
```

**Success criteria:** 
- See text from rope on screen
- Can scroll up/down (with arrow keys or mouse wheel)

---

### Phase 2: Input Handling (Week 2)

**Goal:** Type characters, backspace, enter

**Components:**
```rust
#[derive(Component)]
struct Cursor {
    line: usize,
    column: usize,
}

#[derive(Component)]
struct Selection {
    start: (usize, usize),  // (line, column)
    end: (usize, usize),
}
```

**Systems:**
```rust
fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_input: EventReader<ReceivedCharacter>,
    mut query: Query<(&mut TextBuffer, &mut Cursor)>,
) {
    for (mut buffer, mut cursor) in query.iter_mut() {
        // Handle backspace
        if keyboard.just_pressed(KeyCode::Backspace) {
            if cursor.column > 0 {
                let offset = buffer.rope.offset_of_line(cursor.line) + cursor.column - 1;
                buffer.rope.replace(offset..offset + 1, "");
                cursor.column -= 1;
            }
        }
        
        // Handle enter
        if keyboard.just_pressed(KeyCode::Enter) {
            let offset = buffer.rope.offset_of_line(cursor.line) + cursor.column;
            buffer.rope.replace(offset..offset, "\n");
            cursor.line += 1;
            cursor.column = 0;
        }
        
        // Handle character input
        for event in char_input.read() {
            let c = event.char;
            if !c.is_control() {
                let offset = buffer.rope.offset_of_line(cursor.line) + cursor.column;
                buffer.rope.replace(offset..offset, &c.to_string());
                cursor.column += 1;
            }
        }
    }
}

fn handle_cursor_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&TextBuffer, &mut Cursor)>,
) {
    for (buffer, mut cursor) in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            if cursor.column > 0 {
                cursor.column -= 1;
            }
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            let line_len = buffer.rope.line(cursor.line).len();
            if cursor.column < line_len {
                cursor.column += 1;
            }
        }
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            if cursor.line > 0 {
                cursor.line -= 1;
            }
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            if cursor.line < buffer.rope.lines().count() - 1 {
                cursor.line += 1;
            }
        }
    }
}
```

**Success criteria:**
- Can type characters
- Backspace works
- Enter creates new lines
- Arrow keys move cursor
- Cursor visible on screen

---

### Phase 3: Syntax Highlighting (Week 3)

**Goal:** Use tree-sitter for syntax highlighting

**Components:**
```rust
use tree_sitter::{Parser, Tree, Language};

#[derive(Component)]
struct SyntaxHighlighting {
    tree: Option<Tree>,
    language: Language,
    parser: Parser,
}

#[derive(Component)]
struct HighlightRanges {
    ranges: Vec<HighlightRange>,
}

struct HighlightRange {
    start_byte: usize,
    end_byte: usize,
    color: Color,
}
```

**Systems:**
```rust
fn update_syntax(
    mut query: Query<(&TextBuffer, &mut SyntaxHighlighting), Changed<TextBuffer>>,
) {
    for (buffer, mut syntax) in query.iter_mut() {
        let source = buffer.rope.to_string();
        syntax.tree = syntax.parser.parse(&source, None);
    }
}

fn compute_highlights(
    query: Query<(&TextBuffer, &SyntaxHighlighting, &mut HighlightRanges)>,
) {
    for (buffer, syntax, mut highlights) in query.iter() {
        if let Some(tree) = &syntax.tree {
            let query = tree_sitter::Query::new(
                syntax.language,
                r#"
                (function_item name: (identifier) @function)
                (string_literal) @string
                (integer_literal) @number
                "#
            ).unwrap();
            
            let mut cursor = tree_sitter::QueryCursor::new();
            highlights.ranges.clear();
            
            for match_ in cursor.matches(&query, tree.root_node(), buffer.rope.as_bytes()) {
                for capture in match_.captures {
                    let color = match capture.index {
                        0 => Color::rgb(0.4, 0.8, 1.0), // function
                        1 => Color::rgb(0.8, 1.0, 0.6), // string
                        2 => Color::rgb(1.0, 0.8, 0.4), // number
                        _ => Color::WHITE,
                    };
                    
                    highlights.ranges.push(HighlightRange {
                        start_byte: capture.node.start_byte(),
                        end_byte: capture.node.end_byte(),
                        color,
                    });
                }
            }
        }
    }
}

fn render_with_highlights(
    query: Query<(&TextBuffer, &HighlightRanges)>,
    // Use the highlight ranges when rendering
) {
    // Apply colors from HighlightRanges to text rendering
}
```

**Success criteria:**
- Keywords are colored
- Strings are colored
- Functions are colored
- Updates as you type

---

### Phase 4: Advanced Cursor Features (Week 4)

**Goal:** Multiple cursors, selections, clipboard

**Components:**
```rust
#[derive(Component)]
struct Cursors {
    cursors: Vec<CursorPosition>,
    primary: usize,  // Index of primary cursor
}

#[derive(Clone)]
struct CursorPosition {
    line: usize,
    column: usize,
    selection_start: Option<(usize, usize)>,
}
```

**Systems:**
```rust
fn handle_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&TextBuffer, &mut Cursors)>,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) 
                  || keyboard.pressed(KeyCode::ShiftRight);
    
    for (buffer, mut cursors) in query.iter_mut() {
        let primary = &mut cursors.cursors[cursors.primary];
        
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            if shift_held && primary.selection_start.is_none() {
                primary.selection_start = Some((primary.line, primary.column));
            }
            
            if primary.column > 0 {
                primary.column -= 1;
            }
            
            if !shift_held {
                primary.selection_start = None;
            }
        }
        // Similar for other arrow keys...
    }
}

fn handle_multi_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Cursors>,
) {
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft)
                 || keyboard.pressed(KeyCode::ControlRight);
    
    for mut cursors in query.iter_mut() {
        // Ctrl+D: Add cursor at next occurrence
        if ctrl_held && keyboard.just_pressed(KeyCode::KeyD) {
            // TODO: Find next occurrence of selected text
            // Add new cursor to cursors.cursors
        }
    }
}

fn handle_clipboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&TextBuffer, &Cursors)>,
    // TODO: Add clipboard resource
) {
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft)
                 || keyboard.pressed(KeyCode::ControlRight);
    
    for (buffer, cursors) in query.iter() {
        // Ctrl+C: Copy
        if ctrl_held && keyboard.just_pressed(KeyCode::KeyC) {
            // Get selected text
            // Copy to clipboard
        }
        
        // Ctrl+V: Paste
        if ctrl_held && keyboard.just_pressed(KeyCode::KeyV) {
            // Get clipboard text
            // Insert at cursor(s)
        }
    }
}
```

**Success criteria:**
- Can select text with Shift+arrows
- Copy/paste works
- Multiple cursors work (Ctrl+D style)

---

### Phase 5: LSP Integration (Week 5-6)

**Goal:** Code completions, diagnostics, go-to-definition

**Use Zed's LSP client:**
```rust
use language::LanguageServerName;
use lsp::{LanguageServer, notification::*, request::*};

#[derive(Resource)]
struct LspClient {
    server: Option<Arc<LanguageServer>>,
}

#[derive(Component)]
struct Completions {
    items: Vec<CompletionItem>,
    visible: bool,
}

#[derive(Component)]
struct Diagnostics {
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}
```

**Systems:**
```rust
async fn init_lsp(
    mut commands: Commands,
) {
    // Initialize rust-analyzer
    let server = LanguageServer::new(
        /* ... config ... */
    ).await.unwrap();
    
    commands.insert_resource(LspClient {
        server: Some(Arc::new(server)),
    });
}

fn request_completions(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&TextBuffer, &Cursor)>,
    lsp: Res<LspClient>,
    mut completions: Query<&mut Completions>,
) {
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft);
    
    if ctrl_held && keyboard.just_pressed(KeyCode::Space) {
        for (buffer, cursor) in query.iter() {
            if let Some(server) = &lsp.server {
                // Send completion request
                let position = Position {
                    line: cursor.line as u32,
                    character: cursor.column as u32,
                };
                
                // In async task:
                // let items = server.request::<Completion>(params).await;
                // Send event to update Completions component
            }
        }
    }
}

fn render_completions(
    query: Query<&Completions>,
    // Render completion popup
) {
    for completions in query.iter() {
        if completions.visible {
            // Draw completion menu
        }
    }
}
```

**Success criteria:**
- Ctrl+Space shows completions
- Diagnostics appear as you type
- Go-to-definition works (Ctrl+click)

---

### Phase 6: File System & Projects (Week 7)

**Goal:** Open files, file tree, project management

**Components:**
```rust
#[derive(Component)]
struct OpenFile {
    path: PathBuf,
}

#[derive(Resource)]
struct Project {
    root: PathBuf,
    files: Vec<PathBuf>,
}

#[derive(Component)]
struct FileTree {
    expanded: HashSet<PathBuf>,
}
```

**Systems:**
```rust
fn watch_file_changes(
    query: Query<(&OpenFile, &mut TextBuffer)>,
    // Use notify crate for file watching
) {
    // Reload file if changed externally
}

fn save_file(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&OpenFile, &TextBuffer)>,
) {
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft);
    
    if ctrl_held && keyboard.just_pressed(KeyCode::KeyS) {
        for (file, buffer) in query.iter() {
            std::fs::write(&file.path, buffer.rope.to_string()).unwrap();
        }
    }
}
```

**Success criteria:**
- Can open files
- Can save files
- File tree shows project structure
- File watcher detects external changes

---

## Dependencies

```toml
[dependencies]
# Core Bevy
bevy = "0.14"

# Zed crates (framework-agnostic)
rope = { git = "https://github.com/zed-industries/zed", package = "rope" }
sum_tree = { git = "https://github.com/zed-industries/zed", package = "sum_tree" }
collections = { git = "https://github.com/zed-industries/zed", package = "collections" }
text = { git = "https://github.com/zed-industries/zed", package = "text" }
language = { git = "https://github.com/zed-industries/zed", package = "language" }
lsp = { git = "https://github.com/zed-industries/zed", package = "lsp" }

# Syntax highlighting
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"

# File system
notify = "6.0"

# Async runtime (for LSP)
tokio = { version = "1", features = ["full"] }

# Utilities
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

---

## Comparison: Minimal Feature Set

**What we're building (MVP):**
- ✅ Text editing (insert, delete, undo/redo)
- ✅ Syntax highlighting (tree-sitter)
- ✅ Multiple cursors
- ✅ LSP integration (completions, diagnostics)
- ✅ File operations (open, save)
- ✅ Basic project support

**What we're NOT building (initially):**
- ❌ Git integration
- ❌ Extensions/plugins
- ❌ Collaboration features
- ❌ Advanced refactoring
- ❌ Debugger integration
- ❌ Terminal integration

**What we're GAINING vs Zed:**
- ✅ Bevy's plugin ecosystem
- ✅ Huge community support
- ✅ Easy to extend with Bevy plugins
- ✅ Transferable skills (game dev)

---

## Success Metrics

**Week 1:** Text displays from rope, can scroll
**Week 2:** Can type, edit, cursor moves
**Week 3:** Syntax highlighting works
**Week 4:** Selections, clipboard work
**Week 5:** LSP completions show
**Week 6:** Can open/save files
**Week 7-8:** Polish, performance, bugs

**Final MVP criteria:**
- Can open a Rust file
- Syntax highlighting works
- Can edit with multiple cursors
- LSP provides completions
- Can save changes
- Feels responsive (60fps)

---

## Integration with Crow

**Once the editor works, integrate with crow-core:**

```rust
// The editor becomes a Bevy plugin
pub struct CrowEditorPlugin;

impl Plugin for CrowEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_editor)
            .add_systems(Update, (
                handle_keyboard,
                handle_cursor,
                update_syntax,
                render_text,
            ))
            .init_resource::<LspClient>();
    }
}

// In crow-tauri
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CrowEditorPlugin)  // ← Editor as plugin
        .add_plugins(CrowAgentPlugin)   // ← Agent system as plugin
        .run();
}
```

**The editor surfaces agent output:**
- Agent tool calls render inline
- Executor/Arbiter sessions in split panes
- Real-time streaming of agent thinking

---

## Alternative: Hybrid Approach

**If full rewrite is too much, consider:**

```rust
// Use bevy_egui for UI framework
// Embed Zed's text editing widget
// Get immediate mode GUI + proven editor

use bevy::prelude::*;
use bevy_egui::{egui, EguiPlugin, EguiContexts};

fn ui_system(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        // Use egui::TextEdit (built-in)
        // Or embed custom widget using Zed's rope
    });
}
```

**Pros:** Faster to implement, still Bevy-based
**Cons:** Immediate mode UI (rebuilds every frame)

---

## Resources

**Zed source code:**
- https://github.com/zed-industries/zed
- Focus on: `crates/rope`, `crates/text`, `crates/language`

**Bevy documentation:**
- https://bevyengine.org/learn/
- Book: https://bevy-cheatbook.github.io/

**Tree-sitter:**
- https://tree-sitter.github.io/tree-sitter/

**LSP:**
- https://microsoft.github.io/language-server-protocol/

---

## Conclusion

**This is ambitious but achievable:**
- Use proven components (Zed's text logic)
- Build on mature framework (Bevy)
- Start minimal, iterate
- Huge community support

**The payoff:**
- Own your editor stack
- Integrate deeply with crow agents
- Learn valuable Bevy/game dev skills
- Build something unique

**Start with Phase 0 this week. Get text on screen. Iterate from there.** 🚀
