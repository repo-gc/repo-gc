"""Import-diversity: imports from many different packages"""

import os
import sys
import json
import math
import random
import datetime
import collections
import itertools
import functools
import pathlib
import re
import typing


def diverse_function() -> None:
    cwd = os.getcwd()
    py_ver = sys.version
    data = json.dumps({"a": 1})
    pi = math.pi
    r = random.randint(1, 100)
    now = datetime.datetime.now()
    cnt = collections.Counter()
    odds = list(itertools.islice(itertools.count(1, 2), 10))
    result = functools.reduce(lambda a, b: a + b, [1, 2, 3])
    p = pathlib.Path(".")
    pattern = re.compile(r"\d+")
    t = typing.cast(int, 42)
    void(cwd, py_ver, data, pi, r, now, cnt, odds, result, p, pattern, t)


def void(*args: object) -> None:
    pass
