"""Smoke tests for package imports."""


def test_top_level_imports() -> None:
    from agentic_comm import CommStore, Channel, Message, StoreInfo
    assert CommStore is not None
    assert Channel is not None
    assert Message is not None
    assert StoreInfo is not None


def test_error_imports() -> None:
    from agentic_comm import (
        AcommError,
        AcommNotFoundError,
        CLIError,
        StoreNotFoundError,
        ChannelNotFoundError,
        ValidationError,
    )
    assert issubclass(AcommNotFoundError, AcommError)
    assert issubclass(CLIError, AcommError)
    assert issubclass(StoreNotFoundError, AcommError)
    assert issubclass(ChannelNotFoundError, AcommError)
    assert issubclass(ValidationError, AcommError)


def test_version() -> None:
    import agentic_comm
    assert agentic_comm.__version__ == "0.1.0"
