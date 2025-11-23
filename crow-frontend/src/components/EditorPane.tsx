import { useEffect, useRef, useState } from "react";
import { EditorView, basicSetup } from "codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { python } from "@codemirror/lang-python";
import { EditorState } from "@codemirror/state";
import { fetchFileContent } from "../hooks/useEventStream";

interface Props {
  filePath: string | null;
}

// Dark theme matching our slate palette
const darkTheme = EditorView.theme(
  {
    "&": {
      backgroundColor: "#0f172a",
      color: "#e2e8f0",
      height: "100%",
    },
    ".cm-content": {
      caretColor: "#fff",
    },
    ".cm-gutters": {
      backgroundColor: "#1e293b",
      color: "#64748b",
      border: "none",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "#334155",
    },
    ".cm-activeLine": {
      backgroundColor: "#1e293b",
    },
  },
  { dark: true },
);

export function EditorPane({ filePath }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filePath || !containerRef.current) return;

    const loadFile = async () => {
      setLoading(true);
      setError(null);

      try {
        const content = await fetchFileContent(filePath);

        // Destroy previous editor
        if (viewRef.current) {
          viewRef.current.destroy();
        }

        // Get language extension based on file type
        const langExt = getLanguageExtension(filePath);

        // Create new editor
        const state = EditorState.create({
          doc: content,
          extensions: [
            basicSetup,
            darkTheme,
            EditorView.editable.of(false), // Read-only for now
            ...(langExt ? [langExt] : []),
          ],
        });

        viewRef.current = new EditorView({
          state,
          parent: containerRef.current!,
        });
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load file");
      } finally {
        setLoading(false);
      }
    };

    loadFile();

    return () => {
      if (viewRef.current) {
        viewRef.current.destroy();
        viewRef.current = null;
      }
    };
  }, [filePath]);

  if (!filePath) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#64748b",
          backgroundColor: "#0f172a",
          fontSize: "14px",
        }}
      >
        Select a file to view
      </div>
    );
  }

  if (loading) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#64748b",
          backgroundColor: "#0f172a",
          fontSize: "14px",
        }}
      >
        Loading...
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#f87171",
          backgroundColor: "#0f172a",
          fontSize: "14px",
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
          padding: "8px 12px",
          borderBottom: "1px solid #334155",
          fontSize: "12px",
          color: "#94a3b8",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          backgroundColor: "#1e293b",
        }}
      >
        {filePath}
      </div>
      <div ref={containerRef} style={{ flex: 1, overflow: "auto" }} />
    </div>
  );
}

function getLanguageExtension(filePath: string) {
  const ext = filePath.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "js":
    case "jsx":
    case "ts":
    case "tsx":
      return javascript({ jsx: true, typescript: ext.includes("t") });
    case "py":
      return python();
    default:
      return null;
  }
}
