"""Error types for the AgenticComm Python bindings."""

from __future__ import annotations


class AcommError(Exception):
    """Base exception for all AgenticComm errors."""


class AcommNotFoundError(AcommError):
    """Raised when the acomm CLI binary cannot be found."""

    def __init__(self, searched: list[str] | None = None) -> None:
        locations = ", ".join(searched) if searched else "(none)"
        super().__init__(
            f"acomm binary not found. Searched: {locations}. "
            "Install with: curl -sSL https://raw.githubusercontent.com/"
            "agentralabs/agentic-comm/main/scripts/install.sh | bash"
        )
        self.searched = searched or []


class CLIError(AcommError):
    """Raised when the acomm CLI returns a non-zero exit code."""

    def __init__(self, returncode: int, stderr: str) -> None:
        super().__init__(f"acomm exited with code {returncode}: {stderr}")
        self.returncode = returncode
        self.stderr = stderr


class StoreNotFoundError(AcommError):
    """Raised when a .acomm file does not exist."""

    def __init__(self, path: str) -> None:
        super().__init__(f"Store file not found: {path}")
        self.path = path


class ChannelNotFoundError(AcommError):
    """Raised when a referenced channel does not exist."""

    def __init__(self, channel_id: str) -> None:
        super().__init__(f"Channel not found: {channel_id}")
        self.channel_id = channel_id


class ValidationError(AcommError):
    """Raised when input validation fails."""
