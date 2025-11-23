// Session types
export interface Session {
  id: string;
  title: string;
  directory: string;
  project_id: string;
  parent_id?: string;
  time: {
    created: number;
    updated: number;
  };
  revert?: {
    message_id: string;
    part_id: string;
    snapshot: string;
    diff: string;
  };
}

// Message types - tagged union with role field
export type Message = UserMessage | AssistantMessage;

export interface UserMessage {
  role: "user";
  id: string;
  session_id: string;
  time: {
    created: number;
    completed?: number;
  };
  summary?: string;
  metadata?: unknown;
}

export interface AssistantMessage {
  role: "assistant";
  id: string;
  session_id: string;
  parent_id: string;
  time: {
    created: number;
    completed?: number;
  };
  model?: {
    provider_id: string;
    model_id: string;
  };
  tokens?: {
    input: number;
    output: number;
  };
  cost?: number;
  summary?: string;
  metadata?: unknown;
}

export interface MessageWithParts {
  info: Message;
  parts: Part[];
}

// Part types
export type Part = TextPart | ToolPart | ThinkingPart | FilePart | PatchPart;

export interface TextPart {
  type: "text";
  id: string;
  session_id: string;
  message_id: string;
  text: string;
}

export interface ToolPart {
  type: "tool";
  id: string;
  session_id: string;
  message_id: string;
  call_id: string;
  tool: string;
  state: ToolState;
}

export interface ThinkingPart {
  type: "thinking";
  id: string;
  session_id: string;
  message_id: string;
  text: string;
}

export interface FilePart {
  type: "file";
  id: string;
  session_id: string;
  message_id: string;
  filename?: string;
  url: string;
}

export interface PatchPart {
  type: "patch";
  id: string;
  session_id: string;
  message_id: string;
  hash: string;
  files: string[];
}

export type ToolState =
  | { type: "pending"; raw: string }
  | { type: "running"; input: unknown; title: string; time: { start: number } }
  | {
      type: "completed";
      input: unknown;
      output: string;
      title: string;
      time: { start: number; end: number };
    };

// Event types from /event SSE
export type CrowEvent =
  | { type: "server.connected"; properties: Record<string, never> }
  | { type: "session.created"; properties: { info: Session } }
  | { type: "session.updated"; properties: { info: Session } }
  | { type: "session.deleted"; properties: { info: Session } }
  | {
      type: "session.status";
      properties: { sessionID: string; status: { type: "busy" | "idle" } };
    }
  | { type: "session.idle"; properties: { sessionID: string } }
  | { type: "message.updated"; properties: { info: Message } }
  | {
      type: "message.part.updated";
      properties: { part: Part; delta?: string };
    };

// API response types
export interface SendMessageRequest {
  agent?: string;
  parts: Array<{ type: "text"; text: string }>;
}

// File types
export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number;
}

export interface FileListResponse {
  path: string;
  entries: FileEntry[];
  files?: FileEntry[];
  count: number;
}

// Simplified message for UI
export interface SimpleMessage {
  id: string;
  role: "user" | "assistant";
  parts: Part[];
}
