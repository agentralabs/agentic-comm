"""Shared test fixtures for the AgenticComm SDK test suite."""

import pytest
import tempfile
import shutil
from pathlib import Path


@pytest.fixture
def tmp_dir():
    """Temporary directory for comm store files. Cleaned up after test."""
    d = tempfile.mkdtemp(prefix="acomm_test_")
    yield d
    shutil.rmtree(d, ignore_errors=True)


@pytest.fixture
def store_path(tmp_dir):
    """Path for a temporary comm store file."""
    return str(Path(tmp_dir) / "test.acomm")


@pytest.fixture
def store(store_path):
    """A CommStore instance with a temporary store file.
    Requires the acomm CLI to be available."""
    from agentic_comm import CommStore
    s = CommStore(store_path)
    return s
