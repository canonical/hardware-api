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
from requests_mock import Mocker
from sqlalchemy.orm import Session

from hwapi.data_models import models
from hwapi.external.c3 import api as c3_api


def test_successful_fetch_certficates(
    db_session: Session, requests_mock: Mocker, test_client: TestClient
):
    """Test that certificates and hardware data are imported correctly"""
    requests_mock.get(
        "https://c3_url/api/v2/public-certificates/",
        json={
            "count": 1,
            "next": None,
            "previous": None,
            "results": [
                {
                    "canonical_id": "201806-26288",
                    "vendor": "Dell",
                    "platform": "Z8 G4 Workstation",
                    "configuration": "Z8 G4 Workstation (i3)",
                    "created_at": "2022-08-15T17:31:54.008447+01:00",
                    "completed": "2022-08-15T17:31:54.124274+01:00",
                    "name": "2208-11729",
                    "release": {
                        "codename": "bionic",
                        "release": "18.04 LTS",
                        "release_date": "2018-04-26",
                        "supported_until": "2028-04-25",
                        "i_version": 1804,
                    },
                    "architecture": "amd64",
                    "kernel_version": "4.15.0-55-generic",
                    "bios": {
                        "name": "Dell Inc.: U32",
                        "vendor": "Dell Inc.",
                        "version": "U3",
                        "firmware_type": "",
                    },
                    "firmware_revision": "1.40",
                }
            ],
        },
    )

    c3api = c3_api.C3Api(db=db_session)
    c3api.fetch_certified_configurations()

    # Verify vendors
    assert db_session.query(models.Vendor).count() == 1
    vendor = db_session.query(models.Vendor).filter_by(name="Dell").first()
    assert vendor is not None

    # Verify plarforms
    platform = (
        db_session.query(models.Platform).filter_by(name="Z8 G4 Workstation").first()
    )
    assert platform is not None
    assert platform.vendor == vendor

    # Verify configurations
    configuration = (
        db_session.query(models.Configuration)
        .filter_by(name="Z8 G4 Workstation (i3)")
        .first()
    )
    assert configuration is not None
    assert configuration.platform == platform

    # Verify machines
    machine = (
        db_session.query(models.Machine).filter_by(canonical_id="201806-26288").first()
    )
    assert machine is not None
    assert machine.configuration == configuration

    # Verify kernels
    kernel = (
        db_session.query(models.Kernel).filter_by(version="4.15.0-55-generic").first()
    )
    assert kernel is not None

    # Verify bioses
    bios = db_session.query(models.Bios).filter_by(version="U3").first()
    assert bios is not None
    assert bios.vendor == vendor

    # Verify releases
    release = db_session.query(models.Release).filter_by(codename="bionic").first()
    assert release is not None
    assert release.release == "18.04 LTS"
    assert str(release.release_date) == "2018-04-26"
    assert str(release.supported_until) == "2028-04-25"
    assert release.i_version == 1804

    # Verify certificates
    certificate = (
        db_session.query(models.Certificate).filter_by(name="2208-11729").first()
    )
    assert certificate is not None
    assert certificate.machine == machine
    assert certificate.release == release

    # Verify reports
    report = db_session.query(models.Report).filter_by(certificate=certificate).first()
    assert report is not None
    assert report.kernel == kernel
    assert report.bios == bios


def test_import_with_missing_kernel_bios(
    db_session: Session, requests_mock: Mocker, test_client: TestClient
):
    """Test handling of missing kernel or BIOS data."""
    requests_mock.get(
        "https://c3_url/api/v2/public-certificates/",
        json={
            "count": 1,
            "next": None,
            "previous": None,
            "results": [
                {
                    "canonical_id": "201806-26288",
                    "vendor": "HP",
                    "platform": "Z8 G4 Workstation",
                    "configuration": "Z8 G4 Workstation",
                    "created_at": "2022-08-15T17:31:54.008447+01:00",
                    "completed": "2022-08-15T17:31:54.124274+01:00",
                    "name": "2208-11729",
                    "release": {
                        "codename": "bionic",
                        "release": "18.04 LTS",
                        "release_date": "2018-04-26",
                        "supported_until": "2028-04-25",
                        "i_version": 1804,
                    },
                    "architecture": "amd64",
                    "kernel_version": None,
                    "bios": None,
                    "firmware_revision": None,
                }
            ],
        },
    )

    c3api = c3_api.C3Api(db=db_session)
    c3api.fetch_certified_configurations()
    assert db_session.query(models.Kernel).count() == 0
    assert db_session.query(models.Bios).count() == 0
