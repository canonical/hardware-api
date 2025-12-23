#!/usr/bin/env python3
"""Script to generate the OpenAPI schema for the hardware API."""

import argparse
import difflib
from pathlib import Path

import yaml

from hwapi.main import app


def parse_args():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description="Generate OpenAPI schema for the hardware API."
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--diff",
        type=Path,
        help="Compare generated schema with an existing file.",
        default=None,
    )
    group.add_argument(
        "--output",
        type=Path,
        help="Output file path for the generated OpenAPI schema.",
        default=None,
    )
    return parser.parse_args()


def diff_schemas(expected: str, schema: dict) -> str:
    """Generate a diff between two OpenAPI schemas."""
    expected_lines = expected.splitlines(keepends=True)
    schema_lines = yaml.dump(schema, sort_keys=True).splitlines(keepends=True)
    diff = difflib.unified_diff(expected_lines, schema_lines, lineterm="")
    return "".join(diff)


def main():
    """Generate OpenAPI schema and save it to a YAML file."""
    args = parse_args()
    schema = app.openapi()
    if args.diff is not None:
        expected = Path(args.diff).read_text(encoding="utf-8")
        if diff := diff_schemas(expected, schema):
            print(diff)
            raise SystemExit("Schema differences found.")
        print("No differences found. The schema is up to date.")
    elif args.output is not None:
        with Path(args.output).open("w", encoding="utf-8") as f:
            yaml.dump(schema, f, sort_keys=True)
    else:
        print(yaml.dump(schema, sort_keys=True))


if __name__ == "__main__":
    main()
