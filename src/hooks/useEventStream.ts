import { useEffect, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { CrowEvent, Session, MessageWithParts, Part } from "../types";

// ============================================================================
// Event Stream Hook - Uses Tauri events instead of SSE
// ============================================================================

export function useEventStream(onEvent: (event: CrowEvent) => void) {
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    // Listen for events from the Tauri backend
    listen<CrowEvent>("crow-event", (event) => {
      onEventRef.current(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);
}

// ============================================================================
// Session API - Uses Tauri invoke
// ============================================================================

export async function fetchSessions(): Promise<Session[]> {
  return invoke<Session[]>("list_sessions");
}

export async function createSession(title?: string): Promise<Session> {
  return invoke<Session>("create_session", {
    request: { title, parentID: null },
  });
}

export async function fetchMessages(
  sessionId: string,
): Promise<MessageWithParts[]> {
  return invoke<MessageWithParts[]>("get_messages", { sessionId });
}

export async function deleteSession(_sessionId: string): Promise<void> {
  // TODO: Implement delete_session command in backend
  console.warn("deleteSession not yet implemented");
}

// ============================================================================
// Message Streaming - Uses Tauri Channels
// ============================================================================

export interface StreamCallbacks {
  onTextDelta?: (id: string, delta: string) => void;
  onPart?: (part: Part) => void;
  onComplete?: (message: MessageWithParts) => void;
  onError?: (error: string) => void;
}

interface StreamEvent {
  type:
    | "part"
    | "text_delta"
    | "thinking"
    | "tool_start"
    | "tool_end"
    | "complete"
    | "error";
  part?: Part;
  part_id?: string;
  delta?: string;
  text?: string;
  tool_name?: string;
  tool_id?: string;
  success?: boolean;
  output?: string;
  message_id?: string;
  message?: string;
}

export async function sendMessage(
  sessionId: string,
  text: string,
  _agent: string = "build",
): Promise<void> {
  // Create a channel to receive streaming events
  const channel = new Channel<StreamEvent>();

  // For non-streaming, we just await completion
  await invoke<string>("send_message", {
    sessionId,
    content: text,
    onEvent: channel,
  });
}

export function sendMessageStream(
  sessionId: string,
  text: string,
  callbacks: StreamCallbacks,
  _agent: string = "build",
): () => void {
  let cancelled = false;

  // Create a channel to receive streaming events
  const channel = new Channel<StreamEvent>();

  // Handle incoming events from the channel
  channel.onmessage = (event: StreamEvent) => {
    if (cancelled) return;

    switch (event.type) {
      case "text_delta":
        if (event.part_id && event.delta) {
          callbacks.onTextDelta?.(event.part_id, event.delta);
        }
        break;

      case "part":
        if (event.part) {
          callbacks.onPart?.(event.part);
        }
        break;

      case "thinking":
        if (event.text) {
          // Create a thinking part for the UI
          callbacks.onPart?.({
            type: "thinking",
            id: `thinking-${Date.now()}`,
            session_id: sessionId,
            message_id: "",
            text: event.text,
          } as Part);
        }
        break;

      case "tool_start":
        // Could emit a tool part in pending state
        break;

      case "tool_end":
        // Could update tool part to completed state
        break;

      case "complete":
        // Fetch the complete message to pass to callback
        fetchMessages(sessionId)
          .then((messages) => {
            const lastMessage = messages[messages.length - 1];
            if (lastMessage) {
              callbacks.onComplete?.(lastMessage);
            }
          })
          .catch((err) => callbacks.onError?.(err.message));
        break;

      case "error":
        callbacks.onError?.(event.message || "Unknown error");
        break;
    }
  };

  // Start the streaming request
  invoke<string>("send_message", {
    sessionId,
    content: text,
    onEvent: channel,
  }).catch((err) => {
    if (!cancelled) {
      callbacks.onError?.(err.message || String(err));
    }
  });

  // Return cancel function
  return () => {
    cancelled = true;
  };
}

// ============================================================================
// File API - Uses Tauri invoke
// ============================================================================

import type { FileListResponse } from "../types";

export async function fetchFiles(
  path: string = ".",
): Promise<FileListResponse> {
  return invoke<FileListResponse>("list_directory", { path });
}

export async function fetchFileContent(path: string): Promise<string> {
  return invoke<string>("read_file", { path });
}

// ============================================================================
// Revert API - TODO: Implement in backend
// ============================================================================

export async function revertSession(
  _sessionId: string,
  _messageId: string,
  _partId: string,
): Promise<Session> {
  // TODO: Implement revert_session command in backend
  console.warn("revertSession not yet implemented");
  throw new Error("Not implemented");
}

export async function unrevertSession(_sessionId: string): Promise<Session> {
  // TODO: Implement unrevert_session command in backend
  console.warn("unrevertSession not yet implemented");
  throw new Error("Not implemented");
}
