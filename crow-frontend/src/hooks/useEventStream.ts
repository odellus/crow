import { useEffect, useRef } from "react";
import type { CrowEvent } from "../types";

const API_BASE = "http://localhost:7070";

export function useEventStream(onEvent: (event: CrowEvent) => void) {
  const eventSourceRef = useRef<EventSource | null>(null);
  const onEventRef = useRef(onEvent);

  // Keep callback ref updated
  onEventRef.current = onEvent;

  useEffect(() => {
    const eventSource = new EventSource(`${API_BASE}/event`);
    eventSourceRef.current = eventSource;

    eventSource.addEventListener("message", (event) => {
      try {
        const data = JSON.parse(event.data) as CrowEvent;
        onEventRef.current(data);
      } catch (e) {
        console.error("Failed to parse event:", e, event.data);
      }
    });

    eventSource.onerror = (error) => {
      console.error("EventSource error:", error);
    };

    return () => {
      eventSource.close();
      eventSourceRef.current = null;
    };
  }, []);

  return eventSourceRef;
}

// API functions
export async function fetchSessions(): Promise<import("../types").Session[]> {
  const res = await fetch(`${API_BASE}/session`);
  if (!res.ok) throw new Error(`Failed to fetch sessions: ${res.statusText}`);
  return res.json();
}

export async function createSession(
  title?: string,
): Promise<import("../types").Session> {
  const res = await fetch(`${API_BASE}/session`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title }),
  });
  if (!res.ok) throw new Error(`Failed to create session: ${res.statusText}`);
  return res.json();
}

export async function fetchMessages(
  sessionId: string,
): Promise<import("../types").MessageWithParts[]> {
  const res = await fetch(`${API_BASE}/session/${sessionId}/message`);
  if (!res.ok) throw new Error(`Failed to fetch messages: ${res.statusText}`);
  return res.json();
}

export async function sendMessage(
  sessionId: string,
  text: string,
  agent: string = "build",
): Promise<void> {
  const res = await fetch(`${API_BASE}/session/${sessionId}/message`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      agent,
      parts: [{ type: "text", text }],
    }),
  });
  if (!res.ok) throw new Error(`Failed to send message: ${res.statusText}`);
}

// Streaming message send
export interface StreamCallbacks {
  onTextDelta?: (id: string, delta: string) => void;
  onPart?: (part: import("../types").Part) => void;
  onComplete?: (message: import("../types").MessageWithParts) => void;
  onError?: (error: string) => void;
}

export function sendMessageStream(
  sessionId: string,
  text: string,
  callbacks: StreamCallbacks,
  agent: string = "build",
): () => void {
  const abortController = new AbortController();

  fetch(`${API_BASE}/session/${sessionId}/message/stream`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      agent,
      parts: [{ type: "text", text }],
    }),
    signal: abortController.signal,
  })
    .then(async (res) => {
      if (!res.ok) {
        throw new Error(`Failed to send message: ${res.statusText}`);
      }

      const reader = res.body?.getReader();
      if (!reader) throw new Error("No response body");

      const decoder = new TextDecoder();
      let buffer = "";
      let currentEvent = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        // Process complete SSE messages (separated by double newline)
        const messages = buffer.split("\n\n");
        buffer = messages.pop() || "";

        for (const message of messages) {
          if (!message.trim()) continue;

          const lines = message.split("\n");
          let eventType = "";
          let dataStr = "";

          for (const line of lines) {
            if (line.startsWith("event: ")) {
              eventType = line.slice(7);
            } else if (line.startsWith("data: ")) {
              dataStr = line.slice(6);
            }
          }

          if (!dataStr || dataStr === "keep-alive") continue;

          try {
            const data = JSON.parse(dataStr);

            // Route based on event type or data structure
            if (
              eventType === "text.delta" ||
              (data.delta !== undefined && data.id !== undefined)
            ) {
              callbacks.onTextDelta?.(data.id, data.delta);
            } else if (
              eventType === "message.complete" ||
              (data.info !== undefined && data.parts !== undefined)
            ) {
              callbacks.onComplete?.(data);
            } else if (
              eventType === "part" ||
              (data.id !== undefined &&
                (data.text !== undefined || data.name !== undefined))
            ) {
              callbacks.onPart?.(data);
            } else if (eventType === "error" || data.error !== undefined) {
              callbacks.onError?.(data.error);
            }
          } catch (e) {
            // Ignore parse errors for malformed data
          }
        }
      }
    })
    .catch((err) => {
      if (err.name !== "AbortError") {
        callbacks.onError?.(err.message);
      }
    });

  return () => abortController.abort();
}

export async function deleteSession(sessionId: string): Promise<void> {
  const res = await fetch(`${API_BASE}/session/${sessionId}`, {
    method: "DELETE",
  });
  if (!res.ok) throw new Error(`Failed to delete session: ${res.statusText}`);
}

// File API
export async function fetchFiles(
  path: string = ".",
): Promise<import("../types").FileListResponse> {
  const res = await fetch(`${API_BASE}/file?path=${encodeURIComponent(path)}`);
  if (!res.ok) throw new Error(`Failed to fetch files: ${res.statusText}`);
  return res.json();
}

export async function fetchFileContent(path: string): Promise<string> {
  const res = await fetch(
    `${API_BASE}/file/content?path=${encodeURIComponent(path)}`,
  );
  if (!res.ok)
    throw new Error(`Failed to fetch file content: ${res.statusText}`);
  return res.text();
}

// Revert API
export async function revertSession(
  sessionId: string,
  messageId: string,
  partId: string,
): Promise<import("../types").Session> {
  const res = await fetch(`${API_BASE}/session/${sessionId}/revert`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message_id: messageId, part_id: partId }),
  });
  if (!res.ok) throw new Error(`Failed to revert: ${res.statusText}`);
  return res.json();
}

export async function unrevertSession(
  sessionId: string,
): Promise<import("../types").Session> {
  const res = await fetch(`${API_BASE}/session/${sessionId}/unrevert`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`Failed to unrevert: ${res.statusText}`);
  return res.json();
}
