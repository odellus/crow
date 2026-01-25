"""Crow CLI - Command-line interface for the Crow agent."""

import sys

from crow import __version__
from crow.agent.acp_server import sync_main


def main() -> None:
    """Main CLI entry point."""
    # Get command from arguments
    if len(sys.argv) < 2:
        print("Crow AI - ACP-based coding agent")
        print()
        print("Usage:")
        print("  crow acp           Start ACP server (for use with ACP clients)")
        print("  crow --version     Show version information")
        print()
        print("Examples:")
        print("  crow acp                                    # Start ACP server")
        print(
            "  uv run acp-python-sdk/examples/client.py crow acp  # Connect with ACP client"
        )
        sys.exit(1)

    command = sys.argv[1]

    if command == "acp":
        # Start ACP server
        sync_main()
    elif command in ["--version", "-v"]:
        print(f"crow {__version__}")
    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        print("Run 'crow' without arguments to see usage.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
