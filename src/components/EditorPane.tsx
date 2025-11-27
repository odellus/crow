import { useState, useEffect } from "react";
import Editor from "@monaco-editor/react";
import { fetchFileContent } from "../hooks/useEventStream";

interface Props {
  filePath: string | null;
}

function getLanguage(filePath: string): string {
  const ext = filePath.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "js":
      return "javascript";
    case "jsx":
      return "javascript";
    case "ts":
      return "typescript";
    case "tsx":
      return "typescript";
    case "py":
      return "python";
    case "rs":
      return "rust";
    case "go":
      return "go";
    case "json":
      return "json";
    case "jsonc":
      return "jsonc";
    case "html":
      return "html";
    case "css":
      return "css";
    case "md":
      return "markdown";
    case "yaml":
    case "yml":
      return "yaml";
    case "toml":
      return "toml";
    case "sh":
    case "bash":
      return "shell";
    case "sql":
      return "sql";
    default:
      return "plaintext";
  }
}

export function EditorPane({ filePath }: Props) {
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filePath) {
      setContent("");
      return;
    }

    const loadFile = async () => {
      setLoading(true);
      setError(null);

      try {
        const fileContent = await fetchFileContent(filePath);
        setContent(fileContent);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load file");
      } finally {
        setLoading(false);
      }
    };

    loadFile();
  }, [filePath]);

  const centerStyle: React.CSSProperties = {
    height: "100%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#0f172a",
    fontSize: "13px",
  };

  if (!filePath) {
    return (
      <div style={{ ...centerStyle, color: "#64748b" }}>
        Select a file to view
      </div>
    );
  }

  if (loading) {
    return <div style={{ ...centerStyle, color: "#64748b" }}>Loading...</div>;
  }

  if (error) {
    return (
      <div
        style={{
          ...centerStyle,
          color: "#f87171",
          padding: "16px",
          textAlign: "center",
        }}
      >
        {error}
      </div>
    );
  }

  return (
    <div
      style={{
        height: "100%",
        display: "flex",
        flexDirection: "column",
        backgroundColor: "#0f172a",
      }}
    >
      <div
        style={{
          padding: "6px 12px",
          borderBottom: "1px solid #334155",
          fontSize: "12px",
          color: "#94a3b8",
          backgroundColor: "#1e293b",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {filePath}
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          height="100%"
          width="100%"
          language={getLanguage(filePath)}
          value={content}
          theme="vs-dark"
          options={{
            readOnly: true,
            minimap: { enabled: false },
            fontSize: 13,
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            wordWrap: "on",
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
}
