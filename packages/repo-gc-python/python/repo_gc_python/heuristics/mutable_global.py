"""Mutable global state heuristic (stub)."""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    return None
