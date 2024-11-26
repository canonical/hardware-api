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
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
"""Import data from C3"""

import logging
from requests.exceptions import HTTPError

from sqlalchemy.orm import Session

from hwapi.data_models.setup import engine
from hwapi.external.c3.client import C3Client


logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO)


if __name__ == "__main__":
    session = Session(bind=engine)
    c3_client = C3Client(db=session)
    logger.info("Importing data from C3")
    try:
        c3_client.load_hardware_data()
    except HTTPError as exc:
        logger.error(
            "The %d error code was received from an upstream server",
            exc.response.status_code,
        )
        raise
