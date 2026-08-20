#!/usr/bin/env python3
"""Import specific certificates into the database."""

import argparse
import logging

from sqlalchemy.orm import Session

from hwapi.data_models.setup import engine
from hwapi.external.c3.client import C3Client

logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Import specific certificates into the database."
    )
    parser.add_argument(
        "canonical_ids",
        nargs="+",
        metavar="CID",
        type=str,
        help="Canonical ID(s) of the certificate(s) to import",
    )
    return parser.parse_args()


def main() -> None:
    """Main entrypoint of the import_certificates script."""
    args = parse_args()
    session = Session(bind=engine)
    c3_client = C3Client(db=session)
    c3_client.load_hardware_data(canonical_ids=args.canonical_ids)


if __name__ == "__main__":
    main()
