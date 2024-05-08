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

from fastapi.testclient import TestClient

from hwapi.data_models.enums import CertificationStatus
from tests.data_generator import DataGenerator


def test_certified_status(generator: DataGenerator, test_client: TestClient):
    """
    We should get Certified response if a machine with the specified vendor name (exact
    match) and platform name (the value without data in parenthesis must be a substring
    of a model)
    """
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Precision 3690 (ik12)")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration)
    certificate = generator.gen_certificate(machine, generator.gen_release())
    report = generator.gen_report(
        certificate, generator.gen_kernel(), generator.gen_bios(vendor)
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Precision 3690 aaaa",
            "architecture": "amd64",
            "board": {
                "manufacturer": "Dell Inc.",
                "product_name": "sample board",
                "version": "1.1.1",
            },
        },
    )

    assert response.status_code == 200
    expected_response = {
        "status": CertificationStatus.CERTIFIED.value,
        "os": {
            "distributor": "Canonical Ltd.",
            "version": certificate.release.release,
            "codename": certificate.release.codename,
            "kernel": {
                "name": report.kernel.name,
                "version": report.kernel.version,
                "signature": report.kernel.signature,
            },
            "loaded_modules": [],
        },
        "bios": {
            "firmware_revision": None,
            "release_date": (report.bios.release_date).strftime("%Y-%m-%d"),
            "revision": report.bios.revision,
            "vendor": report.bios.vendor.name,
            "version": report.bios.version,
        },
    }
    assert response.json() == expected_response


def test_vendor_model_not_found(test_client: TestClient):
    """
    If we cannot find such vendor and model in the DB, we should return Not Seen response
    """
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": "Unexsting vendor",
            "model": "Some model",
            "architecture": "amd64",
            "board": {
                "manufacturer": "Dell Inc.",
                "product_name": "sample board",
                "version": "1.1.1",
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}
