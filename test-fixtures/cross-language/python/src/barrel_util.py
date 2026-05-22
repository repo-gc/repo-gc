"""Utility functions used by other modules"""


def helper() -> str:
    return "helper"


def swallow_read(path: str) -> None:
    try:
        with open(path) as f:
            f.read()
    except:
        pass
