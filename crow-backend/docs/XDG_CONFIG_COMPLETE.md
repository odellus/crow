# Crow XDG Configuration - Complete! ✅

## What We Built

Crow now uses **XDG Base Directory** specification, exactly like OpenCode does!

### Directory Structure (Matches OpenCode)

```bash
~/.config/crow/           # Config files (API keys, settings)
~/.local/share/crow/      # Data, binaries, logs
  ├── bin/                # Downloaded tools (like ripgrep, fzf)
  ├── log/                # Application logs
  └── storage/            # Session storage (SQLite, etc.)
~/.local/state/crow/      # State files (history, cache)
~/.cache/crow/            # Temporary cache
```

### How It Works

1. **User creates config once:**
   ```bash
   mkdir -p ~/.config/crow
   cat > ~/.config/crow/.env << 'EOF'
   MOONSHOT_API_KEY=sk-your-key-here
   OPENAI_API_KEY=sk-your-key-here
   # Add other provider keys as needed
   EOF
   ```

2. **Run crow from any project:**
   ```bash
   cd /home/thomas/my-project
   crow
   ```

3. **Crow automatically:**
   - Uses the current working directory as project context
   - Loads API keys from `~/.config/crow/.env`
   - Stores sessions in `~/.local/share/crow/storage/`
   - Logs to `~/.local/share/crow/log/`

## Implementation Details

### Global Paths Module (`crow/packages/api/src/global.rs`)

```rust
pub struct GlobalPaths {
    pub home: PathBuf,
    pub data: PathBuf,      // ~/.local/share/crow
    pub bin: PathBuf,       // ~/.local/share/crow/bin
    pub log: PathBuf,       // ~/.local/share/crow/log
    pub cache: PathBuf,     // ~/.cache/crow
    pub config: PathBuf,    // ~/.config/crow
    pub state: PathBuf,     // ~/.local/state/crow
}
```

### Startup Flow

1. Initialize `GlobalPaths` using XDG environment variables
2. Create all directories if they don't exist
3. Load config from `~/.config/crow/.env` or `~/.config/crow/config`
4. Get current working directory (the project)
5. Start API server on port 7070

### Benefits

✅ **Secure** - API keys stored in `~/.config/crow/`, not in every shell environment
✅ **Standard** - Follows XDG spec like all good Linux apps
✅ **Isolated** - Each project uses same config, but has independent sessions
✅ **Clean** - No `.env` files scattered across projects
✅ **Transparent** - User knows exactly where everything is

## Comparison to OpenCode

| Feature | OpenCode | Crow |
|---------|----------|------|
| Config location | `~/.config/opencode/` | `~/.config/crow/` ✅ |
| Data location | `~/.local/share/opencode/` | `~/.local/share/crow/` ✅ |
| State location | `~/.local/state/opencode/` | `~/.local/state/crow/` ✅ |
| Cache location | `~/.cache/opencode/` | `~/.cache/crow/` ✅ |
| Working directory | Uses `cwd` | Uses `cwd` ✅ |
| Single binary | `opencode` | `crow` ✅ |

## Usage Examples

### First Time Setup

```bash
# 1. Create config directory
mkdir -p ~/.config/crow

# 2. Add API key
echo 'MOONSHOT_API_KEY=sk-...' > ~/.config/crow/.env

# 3. Run crow in any project
cd ~/my-project
crow

# Output:
# 🦅 Crow starting in: /home/user/my-project
# 📁 Config: /home/user/.config/crow
# 📝 Logs: /home/user/.local/share/crow/log
# Starting Crow server...
#   API: http://127.0.0.1:7070
```

### Running in Different Projects

```bash
# Project 1
cd ~/work/backend
crow

# Project 2 (different terminal)
cd ~/personal/website
crow  # Shares config, but separate sessions

# Both use same API keys
# Both have independent session storage
```

### Checking Logs

```bash
# Logs are in one place
tail -f ~/.local/share/crow/log/crow-$(date +%Y-%m-%d).log
```

## What's Next

Now that crow works like opencode:
- ✅ XDG directory structure
- ✅ Config in `~/.config/crow/`
- ✅ Uses current working directory
- ✅ Single `crow` binary
- ✅ API keys loaded from config

Next steps from MIDWAY_PLAN.md:
- [ ] Add Plan and Explore agents
- [ ] Implement background bash execution
- [ ] Build Dioxus web UI
- [ ] Run comparative tests vs OpenCode

---

**We're matching OpenCode's architecture exactly. Carbon copy, baby! 🦅**
