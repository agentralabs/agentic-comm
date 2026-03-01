"""CommStore — High-level Python API for AgenticComm.

Wraps the acomm CLI binary to provide a Pythonic interface for
agent-to-agent communication with channels, pub/sub, and messaging.

Example::

    >>> from agentic_comm import CommStore
    >>> store = CommStore("agents.acomm")
    >>> store.create_channel("general", "broadcast")
    >>> store.send_message("general", "agent-1", "Hello!")
"""

from __future__ import annotations

import logging
from pathlib import Path

from agentic_comm.cli_bridge import find_acomm_binary, run_cli, run_cli_json
from agentic_comm.models import Channel, Message, StoreInfo

logger = logging.getLogger(__name__)


class CommStore:
    """Python interface to a .acomm communication store.

    The store wraps the acomm CLI binary. All operations are executed
    as subprocess calls to the Rust binary, ensuring format compatibility
    and leveraging the same integrity checks (Blake3 / CRC32).

    Args:
        path: Path to the .acomm file. Created if it does not exist.
        binary: Explicit path to the acomm binary. Auto-detected if None.
        timeout: Default subprocess timeout in seconds.
    """

    def __init__(
        self,
        path: str | Path,
        *,
        binary: str | Path | None = None,
        timeout: int = 30,
    ) -> None:
        self.path = Path(path)
        self._binary = find_acomm_binary(binary)
        self._timeout = timeout

    def _run(self, args: list[str]) -> str:
        return run_cli(
            self._binary,
            ["--store", str(self.path)] + args,
            timeout=self._timeout,
        )

    def _run_json(self, args: list[str]) -> dict | list:
        return run_cli_json(
            self._binary,
            ["--store", str(self.path)] + args,
            timeout=self._timeout,
        )

    def info(self) -> StoreInfo:
        """Get summary information about this store."""
        data = self._run_json(["info"])
        assert isinstance(data, dict)
        return StoreInfo(
            path=str(self.path),
            channels=data.get("channels", 0),
            messages=data.get("messages", 0),
            subscriptions=data.get("subscriptions", 0),
            file_size=data.get("file_size", 0),
        )

    def create_channel(self, name: str, channel_type: str = "broadcast") -> str:
        """Create a new communication channel.

        Args:
            name: Human-readable channel name.
            channel_type: Channel type (broadcast, direct, topic).

        Returns:
            The channel ID.
        """
        data = self._run_json(["channel", "create", name, "--type", channel_type])
        assert isinstance(data, dict)
        return str(data.get("id", ""))

    def list_channels(self) -> list[Channel]:
        """List all channels in the store."""
        data = self._run_json(["channel", "list"])
        assert isinstance(data, list)
        return [
            Channel(
                id=ch["id"],
                name=ch["name"],
                channel_type=ch.get("channel_type", "broadcast"),
                created_at=ch.get("created_at", ""),
                metadata=ch.get("metadata", {}),
            )
            for ch in data
        ]

    def send_message(self, channel_id: str, sender: str, content: str) -> str:
        """Send a message to a channel.

        Args:
            channel_id: Target channel ID.
            sender: Sender identifier.
            content: Message content.

        Returns:
            The message ID.
        """
        data = self._run_json(
            ["message", "send", channel_id, "--sender", sender, "--content", content]
        )
        assert isinstance(data, dict)
        return str(data.get("id", ""))

    def search_messages(self, query: str) -> list[Message]:
        """Search messages by content.

        Args:
            query: Search query string.

        Returns:
            List of matching messages.
        """
        data = self._run_json(["message", "search", query])
        assert isinstance(data, list)
        return [
            Message(
                id=msg["id"],
                channel_id=msg.get("channel_id", ""),
                sender=msg.get("sender", ""),
                content=msg.get("content", ""),
                timestamp=msg.get("timestamp", ""),
                metadata=msg.get("metadata", {}),
            )
            for msg in data
        ]

    def __repr__(self) -> str:
        return f"CommStore({str(self.path)!r})"
