import type { Session } from "../types";

interface Props {
  sessions: Session[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onDelete: (id: string) => void;
}

export function SessionList({
  sessions,
  selectedId,
  onSelect,
  onCreate,
  onDelete,
}: Props) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        flex: 1,
        overflow: "hidden",
      }}
    >
      <div style={{ padding: "12px" }}>
        <button
          onClick={onCreate}
          style={{
            width: "100%",
            padding: "10px 16px",
            backgroundColor: "#3b82f6",
            border: "none",
            borderRadius: "6px",
            color: "white",
            fontSize: "14px",
            fontWeight: "500",
            cursor: "pointer",
          }}
        >
          + New Session
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "0 12px 12px" }}>
        {sessions.length === 0 ? (
          <div
            style={{
              padding: "24px",
              textAlign: "center",
              color: "#64748b",
              fontSize: "14px",
            }}
          >
            No sessions yet
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
            {sessions.map((session) => (
              <div
                key={session.id}
                onClick={() => onSelect(session.id)}
                style={{
                  padding: "12px",
                  backgroundColor:
                    selectedId === session.id ? "#334155" : "transparent",
                  borderRadius: "6px",
                  cursor: "pointer",
                  position: "relative",
                }}
                onMouseEnter={(e) => {
                  if (selectedId !== session.id) {
                    e.currentTarget.style.backgroundColor = "#2d3a4f";
                  }
                }}
                onMouseLeave={(e) => {
                  if (selectedId !== session.id) {
                    e.currentTarget.style.backgroundColor = "transparent";
                  }
                }}
              >
                <div
                  style={{
                    fontSize: "14px",
                    fontWeight: "500",
                    marginBottom: "4px",
                    paddingRight: "24px",
                  }}
                >
                  {session.title || "Untitled"}
                </div>
                <div style={{ fontSize: "12px", color: "#64748b" }}>
                  {formatTime(session.time.created)}
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete(session.id);
                  }}
                  style={{
                    position: "absolute",
                    right: "8px",
                    top: "8px",
                    padding: "4px 8px",
                    backgroundColor: "transparent",
                    border: "none",
                    color: "#ef4444",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  ✕
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60000) return "Just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;

  return date.toLocaleDateString();
}
