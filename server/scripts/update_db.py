#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License version 3, as
# published by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
"""Update the hwapi database against the DB pointed at by DB_URL.

Ensures the schema exists, then refreshes hardware data from C3. Designed to be
run after the server is deployed — either inside the server container
(`uv run scripts/update_db.py`) or as a separate one-shot job pointed at the
same DB_URL.
"""

import argparse
import logging
import os
import sys
import time

from requests.exceptions import HTTPError
from sqlalchemy import text
from sqlalchemy.exc import OperationalError
from sqlalchemy.orm import Session

from hwapi.data_models.setup import engine, init_db
from hwapi.external.c3.client import C3Client

logger = logging.getLogger(__name__)


def wait_for_db(timeout: int) -> None:
    """Block until the database accepts a connection, or fail after `timeout` seconds."""
    deadline = time.monotonic() + timeout
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            with engine.connect() as conn:
                conn.execute(text("SELECT 1"))
            return
        except OperationalError as exc:
            last_error = exc
            logger.info("Database not ready yet, retrying...")
            time.sleep(2)
    raise RuntimeError(
        f"Database at {engine.url} not reachable after {timeout}s"
    ) from last_error


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--skip-import",
        action="store_true",
        help="Only ensure the schema exists; do not pull data from C3.",
    )
    parser.add_argument(
        "--wait-timeout",
        type=int,
        default=int(os.getenv("DB_WAIT_TIMEOUT", "60")),
        help="Seconds to wait for the database to become reachable (default: 60).",
    )
    return parser.parse_args()


def main() -> int:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )
    args = parse_args()

    logger.info("Target database: %s", engine.url.render_as_string(hide_password=True))
    wait_for_db(args.wait_timeout)

    logger.info("Ensuring schema is up to date")
    init_db()

    if args.skip_import:
        logger.info("--skip-import set; not importing data from C3")
        return 0

    logger.info("Importing data from C3 (%s)", os.getenv("C3_URL", "default"))
    with Session(bind=engine) as session:
        client = C3Client(db=session)
        try:
            client.load_hardware_data()
        except HTTPError as exc:
            logger.error("Upstream C3 returned %d", exc.response.status_code)
            raise

    logger.info("Database update complete")
    return 0


if __name__ == "__main__":
    sys.exit(main())
