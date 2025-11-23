# Crow Frontend Architecture

React + TypeScript + Tailwind frontend for the crow agent system.

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    React Application                     │
├─────────────┬─────────────┬─────────────┬───────────────┤
│   Header    │  File Tree  │   Editor    │  Chat Panel   │
│   (navbar)  │  (sidebar)  │ (CodeMirror)│  (messages)   │
├─────────────┴─────────────┴─────────────┴───────────────┤
│                  State Management (Context)              │
├─────────────────────────────────────────────────────────┤
│                   API Layer (SSE + Fetch)                │
├─────────────────────────────────────────────────────────┤
│                    Crow Backend (Rust)                   │
└─────────────────────────────────────────────────────────┘
```

---

## Component Hierarchy

```
App
├── AppProvider (Context)
│   ├── Header
│   │   ├── Logo
│   │   ├── SessionSelector
│   │   ├── ModelSelector
│   │   └── SettingsButton
│   │
│   └── MainLayout
│       ├── Sidebar (collapsible)
│       │   ├── FileTree
│       │   └── TodoList
│       │
│       ├── EditorPane (resizable)
│       │   ├── TabBar
│       │   ├── CodeMirrorEditor
│       │   └── StatusBar
│       │
│       └── ChatPane (resizable)
│           ├── MessageList
│           │   ├── UserMessage
│           │   ├── AssistantMessage
│           │   │   ├── TextPart
│           │   │   ├── ToolPart
│           │   │   └── ThinkingPart
│           │   └── StreamingMessage
│           │
│           ├── InputArea
│           │   ├── TextArea
│           │   └── SendButton
│           │
│           └── PermissionDialog (modal)
```

---

## State Management

Using React Context + useReducer for simplicity. No external state library needed for this scope.

### Global State Structure

```typescript
interface AppState {
  // Connection
  connected: boolean;
  serverUrl: string;
  
  // Sessions
  sessions: Session[];
  currentSessionId: string | null;
  
  // Messages
  messages: Map<string, MessageWithParts[]>; // sessionId -> messages
  streamingMessage: StreamingMessage | null;
  
  // Files
  openFiles: OpenFile[];
  activeFileIndex: number;
  fileTree: FileNode[];
  
  // Permissions
  pendingPermissions: PermissionRequest[];
  
  // UI
  sidebarOpen: boolean;
  theme: 'light' | 'dark';
}

interface AppActions {
  // Sessions
  createSession: (directory?: string) => Promise<Session>;
  selectSession: (id: string) => void;
  deleteSession: (id: string) => void;
  
  // Messages
  sendMessage: (text: string) => void;
  abortMessage: () => void;
  revertMessage: (messageId: string) => void;
  
  // Files
  openFile: (path: string) => void;
  closeFile: (index: number) => void;
  saveFile: (path: string, content: string) => void;
  
  // Permissions
  respondToPermission: (id: string, response: 'allow' | 'deny') => void;
}
```

### Context Provider

```typescript
const AppContext = createContext<{
  state: AppState;
  actions: AppActions;
} | null>(null);

export function AppProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  
  // SSE connection
  useEffect(() => {
    const eventSource = new EventSource(`${state.serverUrl}/event`);
    
    eventSource.addEventListener('server.connected', () => {
      dispatch({ type: 'SET_CONNECTED', payload: true });
    });
    
    eventSource.addEventListener('message.created', (e) => {
      const message = JSON.parse(e.data);
      dispatch({ type: 'ADD_MESSAGE', payload: message });
    });
    
    // ... other event handlers
    
    return () => eventSource.close();
  }, [state.serverUrl]);
  
  const actions = useMemo(() => ({
    sendMessage: async (text: string) => {
      // POST to /session/:id/message
    },
    // ... other actions
  }), [state.currentSessionId]);
  
  return (
    <AppContext.Provider value={{ state, actions }}>
      {children}
    </AppContext.Provider>
  );
}
```

---

## Backend Communication Layer

### API Client

```typescript
// src/api/client.ts

const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:7070';

export const api = {
  // Sessions
  listSessions: () => 
    fetch(`${BASE_URL}/session`).then(r => r.json()),
  
  createSession: (directory?: string) =>
    fetch(`${BASE_URL}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ directory }),
    }).then(r => r.json()),
  
  getSession: (id: string) =>
    fetch(`${BASE_URL}/session/${id}`).then(r => r.json()),
  
  deleteSession: (id: string) =>
    fetch(`${BASE_URL}/session/${id}`, { method: 'DELETE' }),
  
  // Messages
  getMessages: (sessionId: string) =>
    fetch(`${BASE_URL}/session/${sessionId}/message`).then(r => r.json()),
  
  sendMessage: (sessionId: string, text: string, noReply = false) =>
    fetch(`${BASE_URL}/session/${sessionId}/message`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parts: [{ type: 'text', text }],
        noReply,
      }),
    }).then(r => r.json()),
  
  abortSession: (sessionId: string) =>
    fetch(`${BASE_URL}/session/${sessionId}/abort`, { method: 'POST' }),
  
  // Files
  readFile: (path: string) =>
    fetch(`${BASE_URL}/file/content?path=${encodeURIComponent(path)}`)
      .then(r => r.text()),
  
  listFiles: (path: string) =>
    fetch(`${BASE_URL}/file?path=${encodeURIComponent(path)}`)
      .then(r => r.json()),
  
  // Permissions
  respondToPermission: (sessionId: string, permissionId: string, response: string) =>
    fetch(`${BASE_URL}/session/${sessionId}/permissions/${permissionId}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ response }),
    }),
};
```

### SSE Event Handling

```typescript
// src/api/events.ts

export function createEventSource(
  url: string,
  handlers: {
    onConnect?: () => void;
    onSessionUpdated?: (session: Session) => void;
    onMessageCreated?: (message: MessageWithParts) => void;
    onMessageUpdated?: (message: MessageWithParts) => void;
    onPartUpdated?: (part: Part) => void;
    onPermissionRequested?: (permission: PermissionRequest) => void;
    onError?: (error: Event) => void;
  }
) {
  const eventSource = new EventSource(`${url}/event`);
  
  eventSource.addEventListener('server.connected', () => {
    handlers.onConnect?.();
  });
  
  eventSource.addEventListener('session.updated', (e) => {
    handlers.onSessionUpdated?.(JSON.parse(e.data));
  });
  
  eventSource.addEventListener('message.created', (e) => {
    handlers.onMessageCreated?.(JSON.parse(e.data));
  });
  
  eventSource.addEventListener('message.updated', (e) => {
    handlers.onMessageUpdated?.(JSON.parse(e.data));
  });
  
  eventSource.addEventListener('part.updated', (e) => {
    handlers.onPartUpdated?.(JSON.parse(e.data));
  });
  
  eventSource.addEventListener('permission.requested', (e) => {
    handlers.onPermissionRequested?.(JSON.parse(e.data));
  });
  
  eventSource.onerror = (e) => {
    handlers.onError?.(e);
  };
  
  return eventSource;
}
```

---

## CodeMirror 6 Integration

### Setup

```typescript
// src/components/Editor/CodeMirrorEditor.tsx

import { EditorState } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
import { javascript } from '@codemirror/lang-javascript';
import { rust } from '@codemirror/lang-rust';
import { python } from '@codemirror/lang-python';
import { oneDark } from '@codemirror/theme-one-dark';

interface EditorProps {
  content: string;
  language: string;
  onChange?: (content: string) => void;
  readOnly?: boolean;
}

export function CodeMirrorEditor({ content, language, onChange, readOnly }: EditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView>();
  
  useEffect(() => {
    if (!editorRef.current) return;
    
    const extensions = [
      lineNumbers(),
      highlightActiveLine(),
      history(),
      syntaxHighlighting(defaultHighlightStyle),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      getLanguageExtension(language),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (update.docChanged && onChange) {
          onChange(update.state.doc.toString());
        }
      }),
    ];
    
    if (readOnly) {
      extensions.push(EditorState.readOnly.of(true));
    }
    
    const state = EditorState.create({
      doc: content,
      extensions,
    });
    
    const view = new EditorView({
      state,
      parent: editorRef.current,
    });
    
    viewRef.current = view;
    
    return () => view.destroy();
  }, [language, readOnly]);
  
  // Update content when prop changes
  useEffect(() => {
    if (viewRef.current) {
      const currentContent = viewRef.current.state.doc.toString();
      if (currentContent !== content) {
        viewRef.current.dispatch({
          changes: { from: 0, to: currentContent.length, insert: content },
        });
      }
    }
  }, [content]);
  
  return <div ref={editorRef} className="h-full w-full" />;
}

function getLanguageExtension(language: string) {
  switch (language) {
    case 'javascript':
    case 'typescript':
    case 'jsx':
    case 'tsx':
      return javascript({ jsx: true, typescript: language.includes('typescript') });
    case 'rust':
      return rust();
    case 'python':
      return python();
    default:
      return [];
  }
}
```

### Diff View

For showing agent changes:

```typescript
// src/components/Editor/DiffView.tsx

import { unifiedMergeView } from '@codemirror/merge';

export function DiffView({ original, modified }: { original: string; modified: string }) {
  const editorRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    if (!editorRef.current) return;
    
    const view = new EditorView({
      parent: editorRef.current,
      extensions: [
        unifiedMergeView({
          original,
          modified,
          highlightChanges: true,
        }),
        oneDark,
      ],
    });
    
    return () => view.destroy();
  }, [original, modified]);
  
  return <div ref={editorRef} className="h-full w-full" />;
}
```

---

## File Tree Component

Using `react-arborist` for virtualized tree:

```typescript
// src/components/FileTree/FileTree.tsx

import { Tree } from 'react-arborist';

interface FileNode {
  id: string;
  name: string;
  isDir: boolean;
  children?: FileNode[];
}

export function FileTree({ 
  data, 
  onSelect 
}: { 
  data: FileNode[]; 
  onSelect: (path: string) => void;
}) {
  return (
    <Tree
      data={data}
      openByDefault={false}
      width="100%"
      height={600}
      indent={16}
      rowHeight={28}
      onSelect={(nodes) => {
        if (nodes.length > 0 && !nodes[0].data.isDir) {
          onSelect(nodes[0].id);
        }
      }}
    >
      {({ node, style, dragHandle }) => (
        <div 
          style={style} 
          ref={dragHandle}
          className={`flex items-center px-2 py-1 hover:bg-gray-700 cursor-pointer ${
            node.isSelected ? 'bg-gray-600' : ''
          }`}
        >
          {node.isLeaf ? (
            <FileIcon className="w-4 h-4 mr-2 text-gray-400" />
          ) : (
            <FolderIcon className="w-4 h-4 mr-2 text-yellow-500" />
          )}
          <span className="text-sm text-gray-200">{node.data.name}</span>
        </div>
      )}
    </Tree>
  );
}
```

---

## Key Design Decisions

### Why Context over Redux/Zustand?

1. **Simplicity** - App state is not deeply nested or complex
2. **No middleware needed** - SSE provides real-time updates
3. **Fewer dependencies** - Smaller bundle size
4. **React 18 optimization** - useSyncExternalStore not needed

### Why CodeMirror over Monaco?

1. **Bundle size** - CodeMirror 6 is significantly smaller
2. **Modularity** - Import only what you need
3. **Performance** - Better mobile/embedded performance
4. **Customization** - Easier to theme with Tailwind

### Why per-request streaming over global event bus?

For MVP, crow uses `POST /session/:id/message/stream` which simplifies implementation. The global `/event` endpoint is still needed for:
- Multi-session updates
- Permission requests
- File change notifications

We'll add `/event` as a priority backend enhancement.

---

## Error Handling Strategy

### API Errors

```typescript
// src/api/client.ts

class ApiError extends Error {
  status: number;
  
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, options);
  
  if (!response.ok) {
    const text = await response.text();
    throw new ApiError(text || response.statusText, response.status);
  }
  
  return response.json();
}
```

### SSE Reconnection

```typescript
// Auto-reconnect on disconnect
eventSource.onerror = () => {
  dispatch({ type: 'SET_CONNECTED', payload: false });
  
  // Exponential backoff reconnect
  setTimeout(() => {
    reconnect();
  }, reconnectDelay);
  
  reconnectDelay = Math.min(reconnectDelay * 2, 30000);
};
```

### UI Error Display

```typescript
// src/components/ErrorBoundary.tsx

export function ErrorBoundary({ children }: { children: React.ReactNode }) {
  const [error, setError] = useState<Error | null>(null);
  
  if (error) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900">
        <div className="text-center">
          <h1 className="text-xl text-red-500 mb-4">Something went wrong</h1>
          <pre className="text-sm text-gray-400 mb-4">{error.message}</pre>
          <button 
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-600 text-white rounded"
          >
            Reload
          </button>
        </div>
      </div>
    );
  }
  
  return (
    <ErrorContext.Provider value={setError}>
      {children}
    </ErrorContext.Provider>
  );
}
```

---

## Performance Considerations

1. **Virtualized lists** - Use `react-arborist` for file tree, `react-virtual` for messages
2. **Memoization** - Wrap expensive components in `React.memo`
3. **Lazy loading** - Code-split editor languages
4. **Debounced updates** - Throttle rapid SSE events
5. **Service worker** - Cache static assets (Vite handles this)
