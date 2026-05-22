"""Dead-weight: never imported by any other file"""


def orphan_function(input_str: str) -> str:
    return input_str.upper()


class OrphanClass:
    def __init__(self, name: str, value: int) -> None:
        self.name = name
        self.value = value

    def process(self) -> int:
        return self.value * 2
