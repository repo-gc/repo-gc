"""God module — high fan-in AND high fan-out → coupling-hotspot"""

from .barrel_util import helper, swallow_read
from .dangerous import dangerous_function, risky_call
from .dup_a import duplicate_function_a
from .dup_b import duplicate_function_b


def orchestrator() -> None:
    helper()
    duplicate_function_a("test")
    duplicate_function_b("test")
    dangerous_function()
    risky_call("https://example.com")
    swallow_read("/dev/null")
