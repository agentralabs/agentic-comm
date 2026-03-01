"""Data models for AgenticComm Python bindings."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class Channel:
    """A communication channel."""

    id: str
    name: str
    channel_type: str
    created_at: str
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class Message:
    """A message in a channel."""

    id: str
    channel_id: str
    sender: str
    content: str
    timestamp: str
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class Subscription:
    """A subscription to a channel or topic."""

    id: str
    channel_id: str
    subscriber: str
    pattern: str | None = None
    created_at: str = ""


@dataclass
class StoreInfo:
    """Summary information about a CommStore."""

    path: str
    channels: int
    messages: int
    subscriptions: int
    file_size: int = 0
