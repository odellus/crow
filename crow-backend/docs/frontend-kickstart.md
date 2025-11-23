# Crow Frontend Kickstart

Ready-to-use starter code for the crow frontend.

---

## Quick Start

```bash
# Create project
npm create vite@latest crow-frontend -- --template react-ts
cd crow-frontend

# Install dependencies
npm install @codemirror/state @codemirror/view @codemirror/language \
  @codemirror/commands @codemirror/lang-javascript @codemirror/theme-one-dark \
  lucide-react

npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Start development
npm run dev
```

---

## Project Structure

```
crow-frontend/
├── index.html
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── tsconfig.json
├── package.json
├── .env
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── types/
│   │   └── index.ts
│   ├── api/
│   │   ├── client.ts
│   │   └── events.ts
│   ├── context/
│   │   └── AppContext.tsx
│   ├── components/
│   │   ├── Header.tsx
│   │   ├── Sidebar.tsx
│   │   ├── ChatPane.tsx
│   │   ├── MessageList.tsx
│   │   ├── MessageInput.tsx
│   │   ├── EditorPane.tsx
│   │   └── FileTree.tsx
│   └── hooks/
│       └── useApp.ts
```

---

## Configuration Files

### vite.config.ts
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:7070',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
});
```

### tailwind.config.js
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        'crow-bg': '#1e1e2e',
        'crow-surface': '#313244',
        'crow-overlay': '#45475a',
        'crow-text': '#cdd6f4',
        'crow-subtext': '#a6adc8',
        'crow-blue': '#89b4fa',
        'crow-green': '#a6e3a1',
        'crow-red': '#f38ba8',
        'crow-yellow': '#f9e2af',
      },
    },
  },
  plugins: [],
};
```

### .env
```bash
VITE_API_URL=http://localhost:7070
```

### src/index.css
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  @apply bg-crow-bg text-crow-text;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  @apply bg-crow-bg;
}

::-webkit-scrollbar-thumb {
  @apply bg-crow-overlay rounded;
}

::-webkit-scrollbar-thumb:hover {
  @apply bg-crow-subtext;
}
```

---

## Core Files

### src/main.tsx
```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import { AppProvider } from './context/AppContext';
import App from './App';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <AppProvider>
      <App />
    </AppProvider>
  </React.StrictMode>
);
```

### src/App.tsx
```typescript
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { ChatPane } from './components/ChatPane';
import { EditorPane } from './components/EditorPane';
import { useApp } from './hooks/useApp';

export default function App() {
  const { state } = useApp();

  return (
    <div className="h-screen flex flex-col">
      <Header />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <main className="flex-1 flex">
          <EditorPane />
          <ChatPane />
        </main>
      </div>
      {!state.connected && (
        <div className="absolute bottom-4 left-4 bg-crow-red text-crow-bg px-4 py-2 rounded">
          Disconnected from server
        </div>
      )}
    </div>
  );
}
```

---

## API Client

### src/api/client.ts
```typescript
const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:7070';

export interface Session {
  id: string;
  parentID: string | null;
  title: string;
  directory: string;
  createdAt: number;
  updatedAt: number;
}

export interface MessageWithParts {
  info: {
    type: 'user' | 'assistant';
    id: string;
    sessionID: string;
    [key: string]: unknown;
  };
  parts: Array<{
    type: string;
    id: string;
    text?: string;
    [key: string]: unknown;
  }>;
}

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = BASE_URL) {
    this.baseUrl = baseUrl;
  }

  // Sessions
  async listSessions(): Promise<Session[]> {
    const res = await fetch(`${this.baseUrl}/session`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }

  async createSession(directory?: string): Promise<Session> {
    const res = await fetch(`${this.baseUrl}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ directory }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }

  async deleteSession(id: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${id}`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error(await res.text());
  }

  async abortSession(id: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${id}/abort`, {
      method: 'POST',
    });
    if (!res.ok) throw new Error(await res.text());
  }

  // Messages
  async getMessages(sessionId: string): Promise<MessageWithParts[]> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/message`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }

  async sendMessage(
    sessionId: string,
    text: string
  ): Promise<MessageWithParts> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/message`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parts: [{ type: 'text', text }],
      }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }

  // Streaming message
  streamMessage(
    sessionId: string,
    text: string,
    onDelta: (delta: string) => void,
    onComplete: (message: MessageWithParts) => void,
    onError: (error: string) => void
  ): () => void {
    const controller = new AbortController();

    fetch(`${this.baseUrl}/session/${sessionId}/message/stream`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parts: [{ type: 'text', text }],
      }),
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(await response.text());
        }

        const reader = response.body?.getReader();
        if (!reader) throw new Error('No response body');

        const decoder = new TextDecoder();
        let buffer = '';

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.startsWith('event: ')) {
              const eventType = line.slice(7);
              // Next line should be data
              continue;
            }
            if (line.startsWith('data: ')) {
              const data = JSON.parse(line.slice(6));
              if (data.delta) {
                onDelta(data.delta);
              }
              if (data.info) {
                onComplete(data);
              }
              if (data.error) {
                onError(data.error);
              }
            }
          }
        }
      })
      .catch((err) => {
        if (err.name !== 'AbortError') {
          onError(err.message);
        }
      });

    return () => controller.abort();
  }

  // Files
  async readFile(path: string): Promise<string> {
    const res = await fetch(
      `${this.baseUrl}/file/content?path=${encodeURIComponent(path)}`
    );
    if (!res.ok) throw new Error(await res.text());
    return res.text();
  }

  async listFiles(path: string): Promise<{
    path: string;
    entries: Array<{
      name: string;
      path: string;
      is_dir: boolean;
      size: number | null;
    }>;
  }> {
    const res = await fetch(
      `${this.baseUrl}/file?path=${encodeURIComponent(path)}`
    );
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }
}

export const api = new ApiClient();
```

---

## State Management

### src/context/AppContext.tsx
```typescript
import React, {
  createContext,
  useReducer,
  useEffect,
  useMemo,
  ReactNode,
} from 'react';
import { api, Session, MessageWithParts } from '../api/client';

interface AppState {
  connected: boolean;
  sessions: Session[];
  currentSessionId: string | null;
  messages: MessageWithParts[];
  streamingText: string;
  isStreaming: boolean;
  error: string | null;
}

type Action =
  | { type: 'SET_CONNECTED'; payload: boolean }
  | { type: 'SET_SESSIONS'; payload: Session[] }
  | { type: 'ADD_SESSION'; payload: Session }
  | { type: 'REMOVE_SESSION'; payload: string }
  | { type: 'SET_CURRENT_SESSION'; payload: string | null }
  | { type: 'SET_MESSAGES'; payload: MessageWithParts[] }
  | { type: 'ADD_MESSAGE'; payload: MessageWithParts }
  | { type: 'SET_STREAMING_TEXT'; payload: string }
  | { type: 'APPEND_STREAMING_TEXT'; payload: string }
  | { type: 'SET_IS_STREAMING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null };

const initialState: AppState = {
  connected: false,
  sessions: [],
  currentSessionId: null,
  messages: [],
  streamingText: '',
  isStreaming: false,
  error: null,
};

function appReducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case 'SET_CONNECTED':
      return { ...state, connected: action.payload };
    case 'SET_SESSIONS':
      return { ...state, sessions: action.payload };
    case 'ADD_SESSION':
      return { ...state, sessions: [...state.sessions, action.payload] };
    case 'REMOVE_SESSION':
      return {
        ...state,
        sessions: state.sessions.filter((s) => s.id !== action.payload),
      };
    case 'SET_CURRENT_SESSION':
      return { ...state, currentSessionId: action.payload, messages: [] };
    case 'SET_MESSAGES':
      return { ...state, messages: action.payload };
    case 'ADD_MESSAGE':
      return { ...state, messages: [...state.messages, action.payload] };
    case 'SET_STREAMING_TEXT':
      return { ...state, streamingText: action.payload };
    case 'APPEND_STREAMING_TEXT':
      return { ...state, streamingText: state.streamingText + action.payload };
    case 'SET_IS_STREAMING':
      return { ...state, isStreaming: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload };
    default:
      return state;
  }
}

interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  actions: {
    loadSessions: () => Promise<void>;
    createSession: (directory?: string) => Promise<void>;
    deleteSession: (id: string) => Promise<void>;
    selectSession: (id: string) => Promise<void>;
    sendMessage: (text: string) => void;
    abortMessage: () => void;
  };
}

export const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  const abortRef = React.useRef<(() => void) | null>(null);

  // Load sessions on mount
  useEffect(() => {
    api
      .listSessions()
      .then((sessions) => {
        dispatch({ type: 'SET_SESSIONS', payload: sessions });
        dispatch({ type: 'SET_CONNECTED', payload: true });
      })
      .catch((err) => {
        dispatch({ type: 'SET_ERROR', payload: err.message });
      });
  }, []);

  // Load messages when session changes
  useEffect(() => {
    if (state.currentSessionId) {
      api
        .getMessages(state.currentSessionId)
        .then((messages) => {
          dispatch({ type: 'SET_MESSAGES', payload: messages });
        })
        .catch((err) => {
          dispatch({ type: 'SET_ERROR', payload: err.message });
        });
    }
  }, [state.currentSessionId]);

  const actions = useMemo(
    () => ({
      loadSessions: async () => {
        const sessions = await api.listSessions();
        dispatch({ type: 'SET_SESSIONS', payload: sessions });
      },

      createSession: async (directory?: string) => {
        const session = await api.createSession(directory);
        dispatch({ type: 'ADD_SESSION', payload: session });
        dispatch({ type: 'SET_CURRENT_SESSION', payload: session.id });
      },

      deleteSession: async (id: string) => {
        await api.deleteSession(id);
        dispatch({ type: 'REMOVE_SESSION', payload: id });
        if (state.currentSessionId === id) {
          dispatch({ type: 'SET_CURRENT_SESSION', payload: null });
        }
      },

      selectSession: async (id: string) => {
        dispatch({ type: 'SET_CURRENT_SESSION', payload: id });
      },

      sendMessage: (text: string) => {
        if (!state.currentSessionId) return;

        dispatch({ type: 'SET_IS_STREAMING', payload: true });
        dispatch({ type: 'SET_STREAMING_TEXT', payload: '' });

        // Add user message immediately
        const userMessage: MessageWithParts = {
          info: {
            type: 'user',
            id: `msg-${Date.now()}`,
            sessionID: state.currentSessionId,
          },
          parts: [
            {
              type: 'text',
              id: `prt-${Date.now()}`,
              text,
            },
          ],
        };
        dispatch({ type: 'ADD_MESSAGE', payload: userMessage });

        // Stream response
        abortRef.current = api.streamMessage(
          state.currentSessionId,
          text,
          (delta) => {
            dispatch({ type: 'APPEND_STREAMING_TEXT', payload: delta });
          },
          (message) => {
            dispatch({ type: 'SET_IS_STREAMING', payload: false });
            dispatch({ type: 'SET_STREAMING_TEXT', payload: '' });
            dispatch({ type: 'ADD_MESSAGE', payload: message });
          },
          (error) => {
            dispatch({ type: 'SET_IS_STREAMING', payload: false });
            dispatch({ type: 'SET_ERROR', payload: error });
          }
        );
      },

      abortMessage: () => {
        if (abortRef.current) {
          abortRef.current();
          abortRef.current = null;
        }
        if (state.currentSessionId) {
          api.abortSession(state.currentSessionId).catch(console.error);
        }
        dispatch({ type: 'SET_IS_STREAMING', payload: false });
      },
    }),
    [state.currentSessionId]
  );

  return (
    <AppContext.Provider value={{ state, dispatch, actions }}>
      {children}
    </AppContext.Provider>
  );
}
```

### src/hooks/useApp.ts
```typescript
import { useContext } from 'react';
import { AppContext } from '../context/AppContext';

export function useApp() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within AppProvider');
  }
  return context;
}
```

---

## Components

### src/components/Header.tsx
```typescript
import { Plus } from 'lucide-react';
import { useApp } from '../hooks/useApp';

export function Header() {
  const { state, actions } = useApp();

  return (
    <header className="h-12 bg-crow-surface border-b border-crow-overlay flex items-center px-4 gap-4">
      <h1 className="text-lg font-semibold text-crow-blue">crow</h1>

      <select
        value={state.currentSessionId || ''}
        onChange={(e) => actions.selectSession(e.target.value)}
        className="bg-crow-bg border border-crow-overlay rounded px-2 py-1 text-sm"
      >
        <option value="">Select session...</option>
        {state.sessions.map((session) => (
          <option key={session.id} value={session.id}>
            {session.title}
          </option>
        ))}
      </select>

      <button
        onClick={() => actions.createSession()}
        className="p-1 hover:bg-crow-overlay rounded"
        title="New session"
      >
        <Plus className="w-5 h-5" />
      </button>

      <div className="flex-1" />

      {state.connected ? (
        <span className="text-xs text-crow-green">Connected</span>
      ) : (
        <span className="text-xs text-crow-red">Disconnected</span>
      )}
    </header>
  );
}
```

### src/components/ChatPane.tsx
```typescript
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';

export function ChatPane() {
  return (
    <div className="w-96 border-l border-crow-overlay flex flex-col bg-crow-bg">
      <div className="flex-1 overflow-hidden">
        <MessageList />
      </div>
      <MessageInput />
    </div>
  );
}
```

### src/components/MessageList.tsx
```typescript
import { useRef, useEffect } from 'react';
import { useApp } from '../hooks/useApp';

export function MessageList() {
  const { state } = useApp();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [state.messages, state.streamingText]);

  return (
    <div className="h-full overflow-y-auto p-4 space-y-4">
      {state.messages.map((msg) => (
        <div
          key={msg.info.id}
          className={`p-3 rounded-lg ${
            msg.info.type === 'user'
              ? 'bg-crow-blue/20 ml-8'
              : 'bg-crow-surface mr-8'
          }`}
        >
          <div className="text-xs text-crow-subtext mb-1">
            {msg.info.type === 'user' ? 'You' : 'Assistant'}
          </div>
          {msg.parts.map((part) => (
            <div key={part.id}>
              {part.type === 'text' && (
                <p className="whitespace-pre-wrap">{part.text}</p>
              )}
              {part.type === 'tool' && (
                <div className="text-xs bg-crow-overlay rounded p-2 mt-2">
                  <span className="text-crow-yellow">Tool: </span>
                  {(part as any).tool}
                </div>
              )}
            </div>
          ))}
        </div>
      ))}

      {state.isStreaming && state.streamingText && (
        <div className="p-3 rounded-lg bg-crow-surface mr-8">
          <div className="text-xs text-crow-subtext mb-1">Assistant</div>
          <p className="whitespace-pre-wrap">{state.streamingText}</p>
          <span className="inline-block w-2 h-4 bg-crow-blue animate-pulse" />
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}
```

### src/components/MessageInput.tsx
```typescript
import { useState, KeyboardEvent } from 'react';
import { Send, StopCircle } from 'lucide-react';
import { useApp } from '../hooks/useApp';

export function MessageInput() {
  const { state, actions } = useApp();
  const [text, setText] = useState('');

  const handleSend = () => {
    if (!text.trim() || !state.currentSessionId) return;
    actions.sendMessage(text);
    setText('');
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="p-4 border-t border-crow-overlay">
      <div className="flex gap-2">
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            state.currentSessionId
              ? 'Type a message...'
              : 'Select a session first'
          }
          disabled={!state.currentSessionId || state.isStreaming}
          className="flex-1 bg-crow-surface border border-crow-overlay rounded px-3 py-2 text-sm resize-none h-20 focus:outline-none focus:border-crow-blue disabled:opacity-50"
        />
        {state.isStreaming ? (
          <button
            onClick={actions.abortMessage}
            className="p-2 bg-crow-red rounded hover:bg-crow-red/80"
            title="Stop"
          >
            <StopCircle className="w-5 h-5" />
          </button>
        ) : (
          <button
            onClick={handleSend}
            disabled={!text.trim() || !state.currentSessionId}
            className="p-2 bg-crow-blue rounded hover:bg-crow-blue/80 disabled:opacity-50 disabled:cursor-not-allowed"
            title="Send"
          >
            <Send className="w-5 h-5" />
          </button>
        )}
      </div>
    </div>
  );
}
```

### src/components/Sidebar.tsx
```typescript
export function Sidebar() {
  return (
    <aside className="w-64 bg-crow-surface border-r border-crow-overlay p-4">
      <h2 className="text-sm font-semibold text-crow-subtext mb-4">Files</h2>
      <p className="text-sm text-crow-subtext">File tree coming soon...</p>
    </aside>
  );
}
```

### src/components/EditorPane.tsx
```typescript
export function EditorPane() {
  return (
    <div className="flex-1 bg-crow-bg p-4">
      <p className="text-crow-subtext">Editor coming soon...</p>
    </div>
  );
}
```

---

## Running the Frontend

1. Start crow backend:
```bash
cd crow
./target/release/crow-serve --port 7070
```

2. Start frontend:
```bash
cd crow-frontend
npm run dev
```

3. Open http://localhost:3000

---

## Next Steps

After this MVP is working:
1. Add CodeMirror editor
2. Add file tree with react-arborist
3. Add SSE event handling for real-time updates
4. Add permission dialogs
5. Add diff viewer

See `frontend-phases.md` for detailed implementation plan.
