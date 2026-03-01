"""Low-level acomm CLI wrapper. NOT part of the public API.

This module handles subprocess management, CLI binary discovery,
output parsing, and error translation. The CommStore class calls this;
users never import it directly.

In a future version, this will be replaced by direct FFI bindings
to the Rust library. The public API (CommStore class) will not change.
"""

from __future__ import annotations

import json
import logging
import os
import shutil
import subprocess
from pathlib import Path

from agentic_comm.errors import AcommNotFoundError, CLIError

logger = logging.getLogger(__name__)

DEFAULT_TIMEOUT = 30


def find_acomm_binary(override: str | Path | None = None) -> Path:
    """Find the acomm CLI binary.

    Search order:
    1. Explicit override path (if provided)
    2. ACOMM_BINARY environment variable
    3. System PATH (shutil.which)
    4. ~/.cargo/bin/acomm (Rust cargo install location)
    5. /usr/local/bin/acomm

    Args:
        override: Explicit path to the binary. Checked first if provided.

    Returns:
        Path to the acomm binary.

    Raises:
        AcommNotFoundError: If the binary cannot be found anywhere.
    """
    searched: list[str] = []

    if override is not None:
        p = Path(override)
        searched.append(str(p))
        if p.is_file() and os.access(str(p), os.X_OK):
            return p
        raise AcommNotFoundError(searched)

    env_path = os.environ.get("ACOMM_BINARY")
    if env_path:
        p = Path(env_path)
        searched.append(str(p))
        if p.is_file() and os.access(str(p), os.X_OK):
            return p

    which_result = shutil.which("acomm")
    searched.append("PATH")
    if which_result:
        return Path(which_result)

    cargo_bin = Path.home() / ".cargo" / "bin" / "acomm"
    searched.append(str(cargo_bin))
    if cargo_bin.is_file() and os.access(str(cargo_bin), os.X_OK):
        return cargo_bin

    usr_local = Path("/usr/local/bin/acomm")
    searched.append(str(usr_local))
    if usr_local.is_file() and os.access(str(usr_local), os.X_OK):
        return usr_local

    raise AcommNotFoundError(searched)


def run_cli(
    binary: Path,
    args: list[str],
    *,
    timeout: int = DEFAULT_TIMEOUT,
    input_data: str | None = None,
) -> str:
    """Run an acomm CLI command and return stdout.

    Args:
        binary: Path to the acomm binary.
        args: Command-line arguments.
        timeout: Subprocess timeout in seconds.
        input_data: Optional stdin data.

    Returns:
        stdout as a string.

    Raises:
        CLIError: If the process exits with non-zero status.
    """
    cmd = [str(binary)] + args
    logger.debug("Running: %s", " ".join(cmd))

    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=timeout,
        input=input_data,
    )

    if result.returncode != 0:
        raise CLIError(result.returncode, result.stderr.strip())

    return result.stdout


def run_cli_json(
    binary: Path,
    args: list[str],
    *,
    timeout: int = DEFAULT_TIMEOUT,
) -> dict | list:
    """Run an acomm CLI command and parse JSON output.

    Args:
        binary: Path to the acomm binary.
        args: Command-line arguments (--json is appended automatically).
        timeout: Subprocess timeout in seconds.

    Returns:
        Parsed JSON output.

    Raises:
        CLIError: If the process exits with non-zero status.
    """
    if "--json" not in args:
        args = args + ["--json"]
    stdout = run_cli(binary, args, timeout=timeout)
    return json.loads(stdout)
