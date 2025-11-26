import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import type { Session, FileEntry, SimpleMessage } from "../types";

type Message = SimpleMessage;
import { ChatView } from "../components/ChatView";
import { FileTree } from "../components/FileTree";
import { EditorPane } from "../components/EditorPane";

interface Props {
  sessions: Session[];
  messages: Message[];
  files: FileEntry[];
  onSendMessage: (text: string) => void;
  onDeleteSession: (id: string) => void;
  onRevert: (messageId: string) => void;
  isStreaming?: boolean;
  streamingText?: string;
}

export function SessionWorkspacePage({
  sessions,
  messages,
  files,
  onSendMessage,
  onDeleteSession,
  onRevert,
  isStreaming = false,
  streamingText = "",
}: Props) {
  void files; // Suppress unused warning - FileTree loads its own files
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  const session = sessions.find((s) => s.id === id);

  if (!session) {
    return (
      <div
        style={{
          height: "100vh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#0f172a",
          color: "white",
        }}
      >
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: "18px", marginBottom: "16px" }}>
            Session not found
          </div>
          <button
            onClick={() => navigate("/")}
            style={{
              padding: "8px 16px",
              backgroundColor: "#3b82f6",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: "pointer",
            }}
          >
            Back to Sessions
          </button>
        </div>
      </div>
    );
  }

  const handleDelete = () => {
    onDeleteSession(session.id);
    navigate("/");
  };

  // Count modified files (placeholder - FileEntry doesn't have modified yet)
  const modifiedCount = 0;

  return (
    <div
      style={{
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        backgroundColor: "#0f172a",
        color: "white",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "12px 16px",
          backgroundColor: "#1e293b",
          borderBottom: "2px solid #334155",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
          <button
            onClick={() => navigate("/")}
            style={{
              padding: "6px 12px",
              backgroundColor: "transparent",
              color: "#94a3b8",
              border: "1px solid #475569",
              borderRadius: "4px",
              cursor: "pointer",
              fontSize: "13px",
            }}
          >
            ← Back
          </button>
          <span
            style={{
              fontFamily: "monospace",
              fontSize: "14px",
              color: "#e2e8f0",
            }}
          >
            Session: {session.id.slice(0, 16)}...
          </span>
        </div>
        <button
          onClick={handleDelete}
          style={{
            padding: "6px 12px",
            backgroundColor: "transparent",
            color: "#ef4444",
            border: "1px solid #ef4444",
            borderRadius: "4px",
            cursor: "pointer",
            fontSize: "13px",
          }}
        >
          Delete
        </button>
      </div>

      {/* Main Content */}
      <div
        style={{
          flex: 1,
          display: "grid",
          gridTemplateColumns: "240px 1fr",
          gridTemplateRows: "1fr 1fr",
          overflow: "hidden",
        }}
      >
        {/* Left Panel - File Tree */}
        <div
          style={{
            gridRow: "1 / 3",
            backgroundColor: "#1e293b",
            borderRight: "2px solid #334155",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              padding: "12px",
              borderBottom: "1px solid #334155",
              fontSize: "12px",
              fontWeight: "600",
              color: "#94a3b8",
              textTransform: "uppercase",
            }}
          >
            Files
          </div>
          <div style={{ flex: 1, overflow: "auto" }}>
            <FileTree
              selectedFile={selectedFile}
              onFileSelect={setSelectedFile}
            />
          </div>
          {modifiedCount > 0 && (
            <div
              style={{
                padding: "12px",
                borderTop: "1px solid #334155",
                display: "flex",
                flexDirection: "column",
                gap: "8px",
              }}
            >
              <div style={{ fontSize: "12px", color: "#f59e0b" }}>
                Modified: {modifiedCount}
              </div>
              <button
                onClick={() => {
                  const lastMsg = messages[messages.length - 1];
                  if (lastMsg) onRevert(lastMsg.id);
                }}
                style={{
                  padding: "6px 12px",
                  backgroundColor: "#f59e0b",
                  color: "black",
                  border: "none",
                  borderRadius: "4px",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "500",
                }}
              >
                Revert
              </button>
            </div>
          )}
        </div>

        {/* Top Right - Chat */}
        <div
          style={{
            backgroundColor: "#0f172a",
            borderBottom: "2px solid #334155",
            overflow: "hidden",
          }}
        >
          <ChatView
            messages={messages}
            onSendMessage={onSendMessage}
            onToolClick={(filePath) => setSelectedFile(filePath)}
            isStreaming={isStreaming}
            streamingText={streamingText}
          />
        </div>

        {/* Bottom Right - Editor */}
        <div
          style={{
            backgroundColor: "#0f172a",
            overflow: "hidden",
          }}
        >
          <EditorPane filePath={selectedFile} />
        </div>
      </div>
    </div>
  );
}
