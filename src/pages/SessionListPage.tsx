import { useNavigate } from "react-router-dom";
import type { Session } from "../types";

interface Props {
  sessions: Session[];
  onCreateSession: () => void;
  onDeleteSession: (id: string) => void;
}

export function SessionListPage({
  sessions,
  onCreateSession,
  onDeleteSession,
}: Props) {
  const navigate = useNavigate();

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = diffMs / (1000 * 60 * 60);
    const diffDays = diffMs / (1000 * 60 * 60 * 24);

    if (diffHours < 1) return "Just now";
    if (diffHours < 24) return `${Math.floor(diffHours)} hours ago`;
    if (diffDays < 7) return `${Math.floor(diffDays)} days ago`;
    return date.toLocaleDateString();
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        backgroundColor: "#0f172a",
        color: "white",
        padding: "32px",
      }}
    >
      {/* Header */}
      <div
        style={{
          maxWidth: "800px",
          margin: "0 auto",
          marginBottom: "32px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <h1
          style={{
            fontSize: "28px",
            fontWeight: "bold",
            margin: 0,
          }}
        >
          Crow IDE
        </h1>
        <button
          onClick={onCreateSession}
          style={{
            padding: "10px 20px",
            backgroundColor: "#3b82f6",
            color: "white",
            border: "none",
            borderRadius: "6px",
            cursor: "pointer",
            fontSize: "14px",
            fontWeight: "500",
          }}
        >
          + New Session
        </button>
      </div>

      {/* Session List */}
      <div
        style={{
          maxWidth: "800px",
          margin: "0 auto",
        }}
      >
        <h2
          style={{
            fontSize: "18px",
            fontWeight: "600",
            marginBottom: "16px",
            color: "#94a3b8",
          }}
        >
          Recent Sessions
        </h2>

        {sessions.length === 0 ? (
          <div
            style={{
              padding: "48px",
              textAlign: "center",
              color: "#64748b",
              backgroundColor: "#1e293b",
              borderRadius: "8px",
            }}
          >
            No sessions yet. Create one to get started!
          </div>
        ) : (
          <div
            style={{ display: "flex", flexDirection: "column", gap: "12px" }}
          >
            {sessions.map((session) => (
              <div
                key={session.id}
                onClick={() => navigate(`/session/${session.id}`)}
                style={{
                  padding: "16px 20px",
                  backgroundColor: "#1e293b",
                  borderRadius: "8px",
                  cursor: "pointer",
                  border: "1px solid #334155",
                  transition: "background-color 0.15s",
                }}
                onMouseEnter={(e) =>
                  (e.currentTarget.style.backgroundColor = "#334155")
                }
                onMouseLeave={(e) =>
                  (e.currentTarget.style.backgroundColor = "#1e293b")
                }
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-start",
                    marginBottom: "8px",
                  }}
                >
                  <span
                    style={{
                      fontFamily: "monospace",
                      fontSize: "14px",
                      color: "#e2e8f0",
                    }}
                  >
                    {session.id.length > 20
                      ? session.id.slice(0, 20) + "..."
                      : session.id}
                  </span>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: "12px",
                    }}
                  >
                    <span
                      style={{
                        fontSize: "12px",
                        color: "#64748b",
                      }}
                    >
                      {formatTime(session.time.created)}
                    </span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onDeleteSession(session.id);
                      }}
                      style={{
                        padding: "4px 8px",
                        backgroundColor: "transparent",
                        color: "#ef4444",
                        border: "1px solid #ef4444",
                        borderRadius: "4px",
                        cursor: "pointer",
                        fontSize: "11px",
                      }}
                    >
                      Delete
                    </button>
                  </div>
                </div>
                <div
                  style={{
                    fontSize: "13px",
                    color: "#94a3b8",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {session.title || session.directory || "No description"}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
