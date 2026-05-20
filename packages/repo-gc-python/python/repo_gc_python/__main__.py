"""Entry point for `python -m repo_gc_python`."""

import sys

from .cli import main

if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()
