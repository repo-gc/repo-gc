"""Code-duplication: identical function body to dup_b.py"""


def duplicate_function_a(input_str: str) -> str:
    result = ""
    for ch in input_str:
        if ch.isalpha():
            result += ch.upper()
        elif ch.isdigit():
            result += ch
        else:
            result += "_"
    return result


def another_unique_a(x: int) -> int:
    return x * x + 2 * x + 1
