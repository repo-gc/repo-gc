"""Error-swallow: bare except: pass blocks"""


def swallow_function(path: str) -> None:
    try:
        with open(path) as f:
            return f.read()
    except:
        pass


def parse_silent(input_str: str) -> int:
    try:
        return int(input_str)
    except:
        return 0


def fetch_silent(url: str) -> None:
    try:
        import urllib.request
        urllib.request.urlopen(url)
    except:
        pass
