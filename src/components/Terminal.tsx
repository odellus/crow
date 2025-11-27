import { useEffect, useRef, useCallback } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke, Channel } from "@tauri-apps/api/core";
import "@xterm/xterm/css/xterm.css";

interface TerminalOutput {
  type: "stdout" | "stderr" | "exit";
  data?: string;
  code?: number;
}

interface Props {
  workingDir?: string;
}

export function Terminal({ workingDir = "." }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const shellIdRef = useRef<string | null>(null);
  const inputBufferRef = useRef<string>("");

  const runCommand = useCallback(async (command: string) => {
    if (!termRef.current) return;

    const term = termRef.current;

    try {
      const channel = new Channel<TerminalOutput>();

      channel.onmessage = (event: TerminalOutput) => {
        if (event.type === "stdout" && event.data) {
          term.write(event.data);
        } else if (event.type === "stderr" && event.data) {
          term.write(`\x1b[31m${event.data}\x1b[0m`);
        } else if (event.type === "exit") {
          term.write(`\r\n`);
          writePrompt();
        }
      };

      shellIdRef.current = await invoke<string>("run_shell_command", {
        command,
        cwd: workingDir,
        onOutput: channel,
      });
    } catch (err) {
      term.write(`\x1b[31mError: ${err}\x1b[0m\r\n`);
      writePrompt();
    }
  }, [workingDir]);

  const writePrompt = useCallback(() => {
    if (termRef.current) {
      termRef.current.write("\x1b[32m$\x1b[0m ");
    }
  }, []);

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new XTerm({
      theme: {
        background: "#0f172a",
        foreground: "#e2e8f0",
        cursor: "#60a5fa",
        cursorAccent: "#0f172a",
        selectionBackground: "#334155",
        black: "#1e293b",
        red: "#f87171",
        green: "#4ade80",
        yellow: "#facc15",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#e2e8f0",
        brightBlack: "#475569",
        brightRed: "#fca5a5",
        brightGreen: "#86efac",
        brightYellow: "#fde047",
        brightBlue: "#93c5fd",
        brightMagenta: "#d8b4fe",
        brightCyan: "#67e8f9",
        brightWhite: "#f8fafc",
      },
      fontFamily: "JetBrains Mono, Menlo, Monaco, monospace",
      fontSize: 13,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: "block",
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerRef.current);
    fitAddon.fit();

    termRef.current = term;
    fitAddonRef.current = fitAddon;

    // Write welcome message
    term.writeln("\x1b[36mCrow Terminal\x1b[0m");
    term.writeln("\x1b[90mType commands to execute in the shell\x1b[0m");
    term.writeln("");
    writePrompt();

    // Handle input
    term.onData((data) => {
      const code = data.charCodeAt(0);

      if (code === 13) { // Enter
        term.write("\r\n");
        const command = inputBufferRef.current.trim();
        inputBufferRef.current = "";

        if (command) {
          runCommand(command);
        } else {
          writePrompt();
        }
      } else if (code === 127) { // Backspace
        if (inputBufferRef.current.length > 0) {
          inputBufferRef.current = inputBufferRef.current.slice(0, -1);
          term.write("\b \b");
        }
      } else if (code === 3) { // Ctrl+C
        inputBufferRef.current = "";
        term.write("^C\r\n");
        writePrompt();
      } else if (code >= 32) { // Printable characters
        inputBufferRef.current += data;
        term.write(data);
      }
    });

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      term.dispose();
    };
  }, [runCommand, writePrompt]);

  return (
    <div
      ref={containerRef}
      style={{
        height: "100%",
        width: "100%",
        padding: "8px",
        boxSizing: "border-box",
        backgroundColor: "#0f172a",
      }}
    />
  );
}
