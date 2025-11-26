import { useState, useEffect } from "react";
import type { FileEntry } from "../types";
import { fetchFiles } from "../hooks/useEventStream";

interface Props {
  onFileSelect: (path: string) => void;
  selectedFile: string | null;
}

export function FileTree({ onFileSelect, selectedFile }: Props) {
  const [currentPath, setCurrentPath] = useState(".");
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadFiles(currentPath);
  }, [currentPath]);

  const loadFiles = async (path: string) => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchFiles(path);
      const sorted = response.entries.sort((a, b) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name);
      });
      setEntries(sorted);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load");
    } finally {
      setLoading(false);
    }
  };

  const handleClick = (entry: FileEntry) => {
    if (entry.is_dir) {
      setCurrentPath(entry.path);
    } else {
      onFileSelect(entry.path);
    }
  };

  const goUp = () => {
    const parent = currentPath.split("/").slice(0, -1).join("/") || ".";
    setCurrentPath(parent);
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <div
        style={{
          padding: "8px 12px",
          borderBottom: "1px solid #334155",
          fontSize: "11px",
          color: "#64748b",
        }}
      >
        {currentPath === "." ? "/" : currentPath.split("/").pop()}
      </div>

      {loading ? (
        <div style={{ padding: "16px", color: "#64748b", fontSize: "13px" }}>
          Loading...
        </div>
      ) : error ? (
        <div style={{ padding: "16px", color: "#f87171", fontSize: "13px" }}>
          {error}
        </div>
      ) : (
        <div style={{ flex: 1, overflowY: "auto" }}>
          {currentPath !== "." && (
            <div
              onClick={goUp}
              style={{
                padding: "8px 12px",
                fontSize: "13px",
                color: "#64748b",
                cursor: "pointer",
              }}
            >
              ↩ ..
            </div>
          )}
          {entries.map((entry) => (
            <div
              key={entry.path}
              onClick={() => handleClick(entry)}
              style={{
                padding: "8px 12px",
                fontSize: "13px",
                cursor: "pointer",
                backgroundColor:
                  selectedFile === entry.path ? "#334155" : "transparent",
                color: selectedFile === entry.path ? "#60a5fa" : "#e2e8f0",
              }}
            >
              {entry.is_dir ? "📁 " : "📄 "}
              {entry.name}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
