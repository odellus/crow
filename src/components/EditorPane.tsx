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

  if (!filePath) {
    return (
      <div className="h-full flex items-center justify-center text-slate-500 bg-slate-900 text-sm">
        Select a file to view
      </div>
    );
  }

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center text-slate-500 bg-slate-900 text-sm">
        Loading...
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center text-red-400 bg-slate-900 text-sm p-4 text-center">
        {error}
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-slate-900">
      <div className="px-3 py-2 border-b border-slate-700 text-xs text-slate-400 truncate bg-slate-800">
        {filePath}
      </div>
      <div className="flex-1 overflow-hidden">
        <Editor
          height="100%"
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
