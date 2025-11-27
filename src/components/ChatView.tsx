import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Part, SimpleMessage } from "../types";

interface Props {
  messages: SimpleMessage[];
  onSendMessage: (text: string) => void;
  onToolClick?: (filePath: string) => void;
  isStreaming?: boolean;
  streamingText?: string;
  streamingParts?: Part[];
}

export function ChatView({
  messages,
  onSendMessage,
  onToolClick,
  isStreaming = false,
  streamingText = "",
  streamingParts = [],
}: Props) {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isStreaming) return;
    onSendMessage(input.trim());
    setInput("");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Status */}
      <div
        style={{
          padding: "8px 16px",
          borderBottom: "1px solid #334155",
          fontSize: "12px",
          color: isStreaming ? "#fbbf24" : "#4ade80",
        }}
      >
        {isStreaming ? "● Agent working..." : "● Ready"}
      </div>

      {/* Messages */}
      <div style={{ flex: 1, overflowY: "auto", padding: "16px" }}>
        {messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} onToolClick={onToolClick} />
        ))}

        {/* Streaming indicator - show when streaming with text OR parts */}
        {isStreaming && (streamingText || streamingParts.length > 0) && (
          <div
            style={{
              marginBottom: "16px",
              padding: "12px 16px",
              backgroundColor: "#1e293b",
              borderRadius: "8px",
              borderLeft: "3px solid #10b981",
            }}
          >
            <div
              style={{
                fontSize: "12px",
                color: "#64748b",
                marginBottom: "8px",
              }}
            >
              🦅 Assistant
            </div>
            {/* Render streaming parts (tool calls, etc.) */}
            {streamingParts.map((part) => (
              <PartRenderer
                key={part.id}
                part={part}
                onToolClick={onToolClick}
              />
            ))}
            {/* Render streaming text */}
            {streamingText && (
              <div style={{ whiteSpace: "pre-wrap", lineHeight: "1.6" }}>
                {streamingText}
                <span style={{ animation: "blink 1s infinite" }}>▊</span>
              </div>
            )}
          </div>
        )}

        {isStreaming && !streamingText && streamingParts.length === 0 && (
          <div
            style={{
              marginBottom: "16px",
              padding: "12px 16px",
              backgroundColor: "#1e293b",
              borderRadius: "8px",
              borderLeft: "3px solid #fbbf24",
            }}
          >
            <div style={{ fontSize: "12px", color: "#fbbf24" }}>
              ● Thinking...
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <form
        onSubmit={handleSubmit}
        style={{
          padding: "16px",
          borderTop: "1px solid #334155",
          backgroundColor: "#1e293b",
        }}
      >
        <div style={{ display: "flex", gap: "8px" }}>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={
              isStreaming ? "Agent is working..." : "Type a message..."
            }
            disabled={isStreaming}
            style={{
              flex: 1,
              padding: "12px 16px",
              backgroundColor: "#0f172a",
              border: "1px solid #334155",
              borderRadius: "8px",
              color: "white",
              fontSize: "14px",
              outline: "none",
            }}
          />
          <button
            type="submit"
            disabled={isStreaming || !input.trim()}
            style={{
              padding: "12px 24px",
              backgroundColor:
                isStreaming || !input.trim() ? "#475569" : "#3b82f6",
              border: "none",
              borderRadius: "8px",
              color: "white",
              fontSize: "14px",
              cursor: isStreaming || !input.trim() ? "not-allowed" : "pointer",
            }}
          >
            Send
          </button>
        </div>
      </form>
    </div>
  );
}

function MessageBubble({
  message,
  onToolClick,
}: {
  message: SimpleMessage;
  onToolClick?: (filePath: string) => void;
}) {
  const isUser = message.role === "user";

  return (
    <div
      style={{
        marginBottom: "16px",
        padding: "12px 16px",
        backgroundColor: isUser ? "#1e3a5f" : "#1e293b",
        borderRadius: "8px",
        borderLeft: isUser ? "3px solid #3b82f6" : "3px solid #10b981",
      }}
    >
      <div style={{ fontSize: "12px", color: "#64748b", marginBottom: "8px" }}>
        {isUser ? "👤 You" : "🦅 Assistant"}
      </div>
      <div>
        {message.parts.map((part) => (
          <PartRenderer key={part.id} part={part} onToolClick={onToolClick} />
        ))}
      </div>
    </div>
  );
}

function PartRenderer({
  part,
  onToolClick,
}: {
  part: Part;
  onToolClick?: (filePath: string) => void;
}) {
  // Handle Part type - it's a union type with discriminant in each variant
  const partAny = part as any;

  // Thinking part - render as blockquote
  if (partAny.type === "thinking" && partAny.text !== undefined) {
    return (
      <div
        style={{
          marginBottom: "8px",
          padding: "8px 12px",
          borderLeft: "3px solid #64748b",
          color: "#94a3b8",
          fontStyle: "italic",
          whiteSpace: "pre-wrap",
          lineHeight: "1.5",
        }}
      >
        {partAny.text}
      </div>
    );
  }

  // Text part - render as markdown
  if (partAny.text !== undefined && !partAny.name) {
    return (
      <div className="markdown-content" style={{ lineHeight: "1.6" }}>
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {partAny.text}
        </ReactMarkdown>
      </div>
    );
  }

  // Tool call part - check for 'tool' field (our ToolPart structure)
  if (partAny.type === "tool" && partAny.tool !== undefined) {
    const toolName = partAny.tool;
    const state = partAny.state || {};
    // state can be: { status: "pending"|"running"|"completed"|"error", input, output, ... }
    const stateStatus = state.status || state.type || "completed";
    const input = state.input || {};
    const output = state.output || "";

    const getStatusInfo = () => {
      switch (stateStatus) {
        case "pending":
          return { bg: "#854d0e", border: "#a16207", label: "⏳ Pending" };
        case "running":
          return { bg: "#1e3a8a", border: "#3b82f6", label: "▶ Running" };
        case "completed":
          return { bg: "#14532d", border: "#22c55e", label: "✓ Done" };
        case "error":
          return { bg: "#7f1d1d", border: "#ef4444", label: "✗ Error" };
        default:
          return { bg: "#14532d", border: "#22c55e", label: "✓ Done" };
      }
    };

    const { bg, border, label } = getStatusInfo();
    const filePath =
      input?.file_path ||
      input?.filePath ||
      input?.path ||
      input?.command ||
      input?.pattern;

    // Format input as JSON for display
    const inputJson = JSON.stringify(input, null, 2);

    return (
      <div
        style={{
          marginTop: "8px",
          padding: "12px",
          backgroundColor: bg,
          borderLeft: `3px solid ${border}`,
          borderRadius: "4px",
          fontSize: "13px",
          cursor: filePath && onToolClick ? "pointer" : "default",
        }}
        onClick={() => {
          if (filePath && onToolClick) {
            onToolClick(String(filePath));
          }
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <span style={{ fontWeight: "500" }}>🔧 {toolName}</span>
          <span style={{ fontSize: "12px" }}>{label}</span>
        </div>

        {/* Full input */}
        <details style={{ marginTop: "8px" }}>
          <summary
            style={{ cursor: "pointer", fontSize: "11px", color: "#94a3b8" }}
          >
            Input
          </summary>
          <pre
            style={{
              fontSize: "11px",
              color: "#e2e8f0",
              marginTop: "4px",
              padding: "8px",
              backgroundColor: "rgba(0,0,0,0.3)",
              borderRadius: "4px",
              overflow: "auto",
              maxHeight: "300px",
              whiteSpace: "pre-wrap",
              wordBreak: "break-all",
            }}
          >
            {inputJson}
          </pre>
        </details>

        {/* Full output */}
        {output && (
          <details style={{ marginTop: "8px" }}>
            <summary
              style={{ cursor: "pointer", fontSize: "11px", color: "#94a3b8" }}
            >
              Output
            </summary>
            <pre
              style={{
                fontSize: "11px",
                color: "#e2e8f0",
                marginTop: "4px",
                padding: "8px",
                backgroundColor: "rgba(0,0,0,0.3)",
                borderRadius: "4px",
                overflow: "auto",
                maxHeight: "300px",
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
              }}
            >
              {output}
            </pre>
          </details>
        )}

        {filePath && (
          <div
            style={{
              fontSize: "11px",
              color: "#94a3b8",
              marginTop: "4px",
              fontFamily: "monospace",
              wordBreak: "break-all",
            }}
          >
            {String(filePath)}
          </div>
        )}
      </div>
    );
  }

  return null;
}
