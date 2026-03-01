"""AgenticComm — Agent-to-agent communication engine.

Quick start::

    >>> from agentic_comm import CommStore
    >>> store = CommStore("agents.acomm")
    >>> store.create_channel("general", "broadcast")
    >>> store.send_message("general", "agent-1", "Hello!")
"""

from agentic_comm.store import CommStore
from agentic_comm.models import (
    Channel,
    Message,
    Subscription,
    StoreInfo,
)
from agentic_comm.errors import (
    AcommError,
    AcommNotFoundError,
    CLIError,
    StoreNotFoundError,
    ChannelNotFoundError,
    ValidationError,
)

__version__ = "0.1.0"
__all__ = [
    "CommStore",
    "Channel",
    "Message",
    "Subscription",
    "StoreInfo",
    "AcommError",
    "AcommNotFoundError",
    "CLIError",
    "StoreNotFoundError",
    "ChannelNotFoundError",
    "ValidationError",
]
