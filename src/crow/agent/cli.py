"""Crow CLI - Command-line interface for the Crow agent."""

import sys


def main() -> None:
    """Main CLI entry point."""
    # Get command from arguments
    if len(sys.argv) < 2:
        print("Usage: crow <command>", file=sys.stderr)
        print("\nAvailable commands:", file=sys.stderr)
        print("  acp    Start the ACP server", file=sys.stderr)
        print("  help   Show this help message", file=sys.stderr)
        sys.exit(1)

    command = sys.argv[1]

    if command == "acp":
        # Import and run ACP server
        from crow.agent.acp_server import sync_main
        sync_main()
    elif command == "help":
        # Show help
        print("Crow - AI Coding Agent")
        print("\nUsage: crow <command>")
        print("\nAvailable commands:")
        print("  acp    Start the ACP (Agent Client Protocol) server")
        print("  help   Show this help message")
        print("\nExamples:")
        print("  crow acp       # Start ACP server")
        print("  crow help      # Show this help")
    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        print("\nAvailable commands: acp, help", file=sys.stderr)
        print("Run 'crow help' for more information.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
