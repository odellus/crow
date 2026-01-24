"""Crow CLI - Command-line interface for the Crow agent."""

import sys


def main() -> None:
    """Main CLI entry point - starts the ACP server."""
    from crow.agent.acp_server import sync_main

    sync_main()


if __name__ == "__main__":
    main()
