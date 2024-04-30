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
from hwapi.external.c3.client import C3Client
from tests.data_generator import DataGenerator


def test_successful_load_certficates(
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

    c3client = C3Client(db=db_session)
    c3client.load_certified_configurations()

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


def test_load_certificates_with_missing_kernel_bios(
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

    c3client = C3Client(db=db_session)
    c3client.load_certified_configurations()
    assert db_session.query(models.Kernel).count() == 0
    assert db_session.query(models.Bios).count() == 0


def test_load_devices(
    db_session: Session,
    requests_mock: Mocker,
    test_client: TestClient,
    generator: DataGenerator,
):
    """
    Test that devices are imported correctly even if there are more than one URL to retrieve
    """
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Precision 3690 (ik12)")
    configuration = generator.gen_configuration(platform)
    machine1 = generator.gen_machine(configuration, canonical_id="202106-8086")
    certificate1 = generator.gen_certificate(
        machine1, generator.gen_release(), name="2204-10686"
    )
    machine2 = generator.gen_machine(configuration, canonical_id="202306-8086")
    certificate2 = generator.gen_certificate(
        machine2, generator.gen_release(), name="2404-10612"
    )
    requests_mock.get(
        "https://c3_url/api/v2/public-device-instances/?pagination=limitoffset&limit=1000",
        json={
            "count": 1,
            "next": (
                "https://c3_url/api/v2/public-device-instances/"
                "?pagination=limitoffset&limit=1000&offset=1000"
            ),
            "previous": None,
            "results": [
                {
                    "machine_canonical_id": machine1.canonical_id,
                    "certificate_name": certificate1.name,
                    "device": {
                        "name": "NetXtreme II BCM5716 Gigabit Ethernet",
                        "subproduct_name": "PowerEdge R410 BCM5716 Gigabit Ethernet",
                        "vendor": "Broadcom Corp.",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "14e4:163b",
                        "subsystem": "1028:028c",
                        "version": None,
                        "category": "NETWORK",
                        "codename": "sample codfename",
                    },
                    "driver_name": "bnx2",
                }
            ],
        },
    )
    requests_mock.get(
        (
            "https://c3_url/api/v2/public-device-instances/?pagination=limitoffset"
            "&limit=1000&offset=1000"
        ),
        json={
            "count": 1,
            "next": None,
            "previous": None,
            "results": [
                {
                    "machine_canonical_id": machine2.canonical_id,
                    "certificate_name": certificate2.name,
                    "device": {
                        "name": "SB7x0/SB8x0/SB9x0 USB OHCI0 Controller",
                        "subproduct_name": None,
                        "vendor": "Advanced Micro Devices, Inc. [AMD/ATI]",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "1002:4397",
                        "subsystem": None,
                        "version": None,
                        "category": "USB",
                        "codename": "",
                    },
                    "driver_name": "ohci-pci",
                }
            ],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_devices()

    assert db_session.query(models.Device).count() == 2
    assert db_session.query(models.Report).count() == 2

    ethernet_device = (
        db_session.query(models.Device).filter_by(identifier="14e4:163b").first()
    )
    usb_device = (
        db_session.query(models.Device).filter_by(identifier="1002:4397").first()
    )

    assert ethernet_device is not None
    assert usb_device is not None

    assert ethernet_device.name == "NetXtreme II BCM5716 Gigabit Ethernet"
    assert usb_device.name == "SB7x0/SB8x0/SB9x0 USB OHCI0 Controller"

    report1 = (
        db_session.query(models.Report)
        .filter_by(certificate_id=certificate1.id)
        .first()
    )
    assert report1 is not None
    assert ethernet_device in report1.devices

    report2 = (
        db_session.query(models.Report)
        .filter_by(certificate_id=certificate2.id)
        .first()
    )
    assert report2 is not None
    assert usb_device in report2.devices


def test_load_devices_duplicate_names(
    db_session: Session,
    requests_mock: Mocker,
    test_client: TestClient,
    generator: DataGenerator,
):
    """
    Test that devices with the same name, vendor, subsystem, bus, version,
    and category are added only one
    """
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Precision 3690 (ik12)")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration, canonical_id="202106-8086")
    certificate = generator.gen_certificate(
        machine, generator.gen_release(), name="2204-10686"
    )

    requests_mock.get(
        "https://c3_url/api/v2/public-device-instances/",
        json={
            "count": 3,
            "next": None,
            "previous": None,
            "results": [
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Sky Lake-E CHA Registers",
                        "subproduct_name": None,
                        "vendor": "Intel Corp.",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "8086:208e",
                        "subsystem": "1458:1000",
                        "version": None,
                        "category": "OTHER",
                        "codename": "",
                    },
                    "driver_name": "unknown",
                },
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Sky Lake-E CHA Registers",
                        "subproduct_name": None,
                        "vendor": "Intel Corp.",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "8086:208e",
                        "subsystem": "1458:1000",
                        "version": None,
                        "category": "OTHER",
                        "codename": "",
                    },
                    "driver_name": "unknown",
                },
                # different subsystem identifier
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Sky Lake-E CHA Registers",
                        "subproduct_name": None,
                        "vendor": "Intel Corp.",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "8086:208e",
                        "subsystem": "17aa:1036",
                        "version": None,
                        "category": "OTHER",
                        "codename": "",
                    },
                    "driver_name": "unknown",
                },
            ],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_devices()

    assert db_session.query(models.Device).count() == 2
    assert (
        db_session.query(models.Device).filter_by(subsystem="1458:1000").first()
        is not None
    )
    assert (
        db_session.query(models.Device).filter_by(subsystem="17aa:1036").first()
        is not None
    )
