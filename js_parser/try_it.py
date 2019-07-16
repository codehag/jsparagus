#!/usr/bin/env python

"""js.py - Repl-like toy to explore parsing of lines of JS.

See README.md for instructions.
"""

from .parser import JSParser
from jsparagus.lexer import SyntaxError


def main():
    while True:
        parser = JSParser()
        try:
            result = parser.read()
        except SyntaxError as exc:
            print(exc.__class__.__name__ + ": " + str(exc))
            continue
        print(result)


if __name__ == '__main__':
    main()
