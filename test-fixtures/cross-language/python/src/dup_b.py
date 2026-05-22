"""Code-duplication: identical function body to dup_a.py"""


def duplicate_function_b(input_str: str) -> str:
    result = ""
    for ch in input_str:
        if ch.isalpha():
            result += ch.upper()
        elif ch.isdigit():
            result += ch
        else:
            result += "_"
    return result


def another_unique_b(y: int) -> int:
    return y * y + 3 * y + 2
