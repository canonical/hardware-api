# Copyright 2024 Canonical Ltd.
# All rights reserved.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
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
#        Omar Abou Selo <omar.selo@canonical.com>
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
"""The module for working with C3 API"""

import os
import requests

from sqlalchemy.orm import Session

from hwapi.data_models import models
from hwapi.data_models.repository import get_or_create
from hwapi.external.c3 import response_models, urls


class C3Api:
    # Store the token as class attribute so we can reuse it in multiple class instances
    _bearer_token = ""

    def __init__(self, db: Session):
        self.db = db
        self._client_id = os.environ.get("C3_CLIENT_ID", "")
        self._client_secret = os.environ.get("C3_CLIENT_SECRET", "")

    def fetch_certified_configurations(self):
        """
        Retrieve certified configurations from C3 and create corresponding models
        """
        response = requests.get(urls.CERTIFIED_CONFIGURATIONS_URL + urls.LIMIT_OFFSET)
        if not response.ok:
            return
        objects = response.json()["results"]
        for obj in objects:
            certified_config = response_models.CertifiedConfiguration(**obj)
            vendor = get_or_create(self.db, models.Vendor, name=certified_config.make)
            platform = get_or_create(
                self.db,
                models.Platform,
                name=certified_config.model,
                vendor_id=vendor.id,
            )
            configuration = get_or_create(
                self.db,
                models.Configuration,
                name=certified_config.model,
                platform_id=platform.id,
            )
            get_or_create(
                self.db,
                models.Machine,
                canonical_id=certified_config.canonical_id,
                configuration_id=configuration.id,
            )

    def _authenticate_and_send(self, request: requests.Request) -> requests.Response:
        prepared_request = request.prepare()
        prepared_request.headers["Authorization"] = f"Bearer {C3Api._bearer_token}"
        session = requests.session()
        response = session.send(prepared_request)
        if response.status_code in (401, 403):
            self._authenticate()
            prepared_request.headers["Authorization"] = f"Bearer {C3Api._bearer_token}"
            response = session.send(prepared_request)
        return response

    def _authenticate(self) -> None:
        response = requests.post(
            urls.TOKEN_URL,
            auth=(self._client_id, self._client_secret),
            data={"grant_type": "client_credentials", "scope": "read"},
        )

        if response.ok:
            C3Api._bearer_token = response.json().get("access_token", "")
