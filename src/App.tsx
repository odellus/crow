import { useState, useEffect, useCallback } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  useNavigate,
  useParams,
} from "react-router-dom";
import type {
  Session,
  Part,
  FileEntry,
  CrowEvent,
  SimpleMessage,
} from "./types";

// Use SimpleMessage for UI state
type Message = SimpleMessage;
import {
  useEventStream,
  fetchSessions,
  fetchMessages,
  createSession,
  deleteSession,
  fetchFiles,
  revertSession,
  sendMessageStream,
} from "./hooks/useEventStream";
import { SessionListPage } from "./pages/SessionListPage";
import { SessionWorkspacePage } from "./pages/SessionWorkspacePage";

function AppContent() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [streamingParts, setStreamingParts] = useState<Part[]>([]);
  const navigate = useNavigate();

  // Load sessions on mount
  useEffect(() => {
    fetchSessions().then(setSessions).catch(console.error);
  }, []);

  // Handle SSE events
  const handleEvent = useCallback(
    (event: CrowEvent) => {
      switch (event.type) {
        case "session.created":
          setSessions((prev) => [
            ...prev,
            (event.properties as any).info as Session,
          ]);
          break;
        case "session.deleted":
          setSessions((prev) =>
            prev.filter((s) => s.id !== (event.properties as any).info?.id),
          );
          break;
        case "message.updated":
          if (currentSessionId) {
            fetchMessages(currentSessionId)
              .then((msgs) => {
                setMessages(
                  msgs.map(
                    (m) =>
                      ({
                        id: m.info.id || "",
                        role: m.info.role,
                        parts: m.parts,
                      }) as Message,
                  ),
                );
              })
              .catch(console.error);
          }
          break;
      }
    },
    [currentSessionId],
  );

  useEventStream(handleEvent);

  // Load messages when session changes
  useEffect(() => {
    if (currentSessionId) {
      fetchMessages(currentSessionId)
        .then((msgs) => {
          setMessages(
            msgs.map(
              (m) =>
                ({
                  id: m.info.id || "",
                  role: m.info.role,
                  parts: m.parts,
                }) as Message,
            ),
          );
        })
        .catch(console.error);

      // Load files
      fetchFiles(".")
        .then((response) => {
          setFiles(response.entries || []);
        })
        .catch(console.error);
    } else {
      setMessages([]);
      setFiles([]);
    }
  }, [currentSessionId]);

  const handleCreateSession = async () => {
    try {
      const session = await createSession();
      setSessions((prev) => [...prev, session]);
      navigate(`/session/${session.id}`);
    } catch (err) {
      console.error("Failed to create session:", err);
    }
  };

  const handleDeleteSession = async (id: string) => {
    try {
      await deleteSession(id);
      setSessions((prev) => prev.filter((s) => s.id !== id));
      if (currentSessionId === id) {
        setCurrentSessionId(null);
        navigate("/");
      }
    } catch (err) {
      console.error("Failed to delete session:", err);
    }
  };

  const handleSendMessage = (text: string) => {
    if (!currentSessionId || !text.trim()) return;

    // Add user message immediately
    const userMsg: Message = {
      id: `temp-${Date.now()}`,
      role: "user",
      parts: [
        {
          id: `part-${Date.now()}`,
          session_id: currentSessionId,
          message_id: `temp-${Date.now()}`,
          text,
        } as Part,
      ],
    };
    setMessages((prev) => [...prev, userMsg]);

    // Start streaming
    setIsStreaming(true);
    setStreamingText("");
    setStreamingParts([]);

    sendMessageStream(currentSessionId, text, {
      onTextDelta: (_id, delta) => {
        setStreamingText((prev) => prev + delta);
      },
      onPart: (part) => {
        // Add part to streaming parts for real-time display
        setStreamingParts((prev) => [...prev, part]);
      },
      onComplete: (_message) => {
        setIsStreaming(false);
        setStreamingText("");
        setStreamingParts([]);
        // Refresh messages to get final state
        fetchMessages(currentSessionId!)
          .then((msgs) => {
            setMessages(
              msgs.map(
                (m) =>
                  ({
                    id: m.info.id || "",
                    role: m.info.role,
                    parts: m.parts,
                  }) as Message,
              ),
            );
          })
          .catch(console.error);
      },
      onError: (error) => {
        setIsStreaming(false);
        setStreamingText("");
        console.error("Stream error:", error);
      },
    });
  };

  const handleRevert = async (messageId: string) => {
    if (!currentSessionId) return;
    try {
      await revertSession(currentSessionId, messageId, "");
      // Refresh messages and files
      fetchMessages(currentSessionId)
        .then((msgs) => {
          setMessages(
            msgs.map(
              (m) =>
                ({
                  id: m.info.id || "",
                  role: m.info.role,
                  parts: m.parts,
                }) as Message,
            ),
          );
        })
        .catch(console.error);
      fetchFiles(".")
        .then((response) => {
          setFiles(response.entries || []);
        })
        .catch(console.error);
    } catch (err) {
      console.error("Failed to revert:", err);
    }
  };

  return (
    <Routes>
      <Route
        path="/"
        element={
          <SessionListPage
            sessions={sessions}
            onCreateSession={handleCreateSession}
            onDeleteSession={handleDeleteSession}
          />
        }
      />
      <Route
        path="/session/:id"
        element={
          <SessionWorkspaceWrapper
            sessions={sessions}
            messages={messages}
            files={files}
            onSendMessage={handleSendMessage}
            onDeleteSession={handleDeleteSession}
            onRevert={handleRevert}
            setCurrentSessionId={setCurrentSessionId}
            isStreaming={isStreaming}
            streamingText={streamingText}
            streamingParts={streamingParts}
          />
        }
      />
    </Routes>
  );
}

// Wrapper to handle session ID from URL
function SessionWorkspaceWrapper(props: {
  sessions: Session[];
  messages: Message[];
  files: FileEntry[];
  onSendMessage: (text: string) => void;
  onDeleteSession: (id: string) => void;
  onRevert: (messageId: string) => void;
  setCurrentSessionId: (id: string | null) => void;
  isStreaming: boolean;
  streamingText: string;
  streamingParts: Part[];
}) {
  const { id } = useParams<{ id: string }>();

  useEffect(() => {
    props.setCurrentSessionId(id || null);
    return () => props.setCurrentSessionId(null);
  }, [id]);

  return (
    <SessionWorkspacePage
      sessions={props.sessions}
      messages={props.messages}
      files={props.files}
      onSendMessage={props.onSendMessage}
      onDeleteSession={props.onDeleteSession}
      onRevert={props.onRevert}
      isStreaming={props.isStreaming}
      streamingText={props.streamingText}
      streamingParts={props.streamingParts}
    />
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  );
}
