# Crow

AI-powered coding assistant with a web interface.

## Features

- Chat-based coding assistance
- File operations (read, write, edit)
- Shell command execution
- Web search and URL fetching
- Task planning with todo lists
- Real-time streaming responses
- Session persistence and history

## Quick Start

### Prerequisites
- Rust (for backend)
- Node.js (for frontend)
- An OpenAI-compatible LLM API

### 1. Configure LLM Provider

Create `.crow/config.jsonc` in your project directory:

```jsonc
{
  "provider": {
    "name": "openai-compatible",
    "model": "gpt-4",
    "api_key": "your-api-key",
    "base_url": "https://api.openai.com/v1"
  }
}
```

### 2. Start Backend

```bash
cd crow-backend
cargo build --bin crow-serve --features server
cd /path/to/your/project
/path/to/crow-backend/target/debug/crow-serve
```

Backend runs on `http://localhost:7070`

### 3. Start Frontend

```bash
cd crow-frontend
npm install
npm run dev
```

Frontend runs on `http://localhost:5173`

### 4. Open Browser

Navigate to `http://localhost:5173` to start chatting with Crow.

## Usage

1. Create a new session from the home page
2. Type your coding request in the chat
3. Crow will analyze your request and use appropriate tools
4. View file changes, command outputs, and explanations in real-time

## Project Structure

```
crow/
├── crow-backend/     # Rust backend (API, tools, LLM integration)
├── crow-frontend/    # React frontend (chat UI)
└── docs/            # Planning and design documents
```

## Configuration Options

### Provider Settings

| Field | Description |
|-------|-------------|
| `name` | Provider type (`openai-compatible`) |
| `model` | Model name (e.g., `gpt-4`, `claude-3-sonnet`) |
| `api_key` | API key for the provider |
| `base_url` | API endpoint URL |

### Supported Providers

Any OpenAI-compatible API including:
- OpenAI
- Anthropic (via proxy)
- Local models (llama.cpp, ollama)
- Azure OpenAI

## Development

See [AGENTS.md](./AGENTS.md) for detailed architecture and development information.

## Based On

Crow is based on [OpenCode](https://github.com/opencode-ai/opencode), an open-source AI coding assistant.

## License

MIT
