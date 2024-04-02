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
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>


from datetime import timedelta, datetime
from fastapi.testclient import TestClient


def test_certified_status(test_client: TestClient):
    # Patch the db session or the specific query method used in your endpoint
    response = test_client.post(
        "/v1/certification/status",
        json={"vendor": "Dell", "model": "ChengMing 3980 (i3-9100)"},
    )

    assert response.status_code == 200
    expected_response = {
        "status": "Certified",
        "os": {
            "distributor": "Canonical Ltd.",
            "description": "",
            "version": "20.04",
            "codename": "focal",
            "kernel": {
                "name": "Linux",
                "version": "5.4.0-42-generic",
                "signature": "0000000",
            },
            "loaded_modules": [],
        },
        "bios": {
            "firmware_revision": "1.0.0",
            "release_date": (datetime.now().date() - timedelta(days=365)).strftime(
                "%Y-%m-%d"
            ),
            "revision": "A01",
            "vendor": "Lenovo",
            "version": "1.0.2",
        },
    }
    assert response.json() == expected_response
