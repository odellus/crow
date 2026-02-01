"""Crow CLI - Multi-mode agent interface."""

from __future__ import annotations

import sys
from pathlib import Path

import typer

app = typer.Typer(
    name="crow",
    help="Crow AI agent framework",
    add_completion=False,
)


def _version_callback(value: bool) -> None:
    """Show version and exit."""
    if value:
        from crow import __version__
        typer.echo(f"crow {__version__}")
        raise typer.Exit()


@app.callback()
def main(
    version: bool = typer.Option(
        False,
        "--version",
        "-v",
        help="Show version and exit",
        callback=_version_callback,
    ),
) -> None:
    """Crow AI agent framework."""
    pass


@app.command()
def acp() -> None:
    """Start ACP server mode (stdin/stdout communication)."""
    from crow.acp.server import sync_main
    
    # Run the ACP server
    sync_main()


@app.command()
def editor(
    port: int = typer.Option(9873, "--port", "-p", help="Port to run the server on"),
    host: str = typer.Option("localhost", "--host", "-H", help="Host to bind to"),
    reload: bool = typer.Option(False, "--reload", help="Enable auto-reload for development"),
) -> None:
    """Start web IDE mode (browser-based interface)."""
    import uvicorn
    
    from crow.editor.server import app
    
    if reload:
        # Development mode with auto-reload
        uvicorn.run(
            "crow.editor.server:app",
            host=host,
            port=port,
            reload=True,
        )
    else:
        # Production mode
        uvicorn.run(
            app,
            host=host,
            port=port,
        )


if __name__ == "__main__":
    app()
