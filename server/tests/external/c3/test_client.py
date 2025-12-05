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

from datetime import date, timedelta
from unittest.mock import patch

import pytest
import requests.exceptions
from fastapi.testclient import TestClient
from requests_mock import Mocker
from sqlalchemy.orm import Session

from hwapi.data_models import models
from hwapi.external.c3.client import C3Client
from hwapi.external.c3.urls import (
    CPU_IDS_URL,
    PUBLIC_CERTIFICATES_URL,
    PUBLIC_DEVICES_URL,
    get_limit_offset,
)
from tests.data_generator import DataGenerator


def test_load_certificates(db_session: Session, requests_mock: Mocker):
    """Test that certificates and related hardware data are imported correctly."""
    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
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
                        "supported_until": (
                            date.today() + timedelta(days=365)
                        ).strftime("%Y-%m-%d"),
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

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()

    assert db_session.query(models.Vendor).count() == 1
    assert db_session.query(models.Platform).count() == 1
    assert db_session.query(models.Configuration).count() == 1
    assert db_session.query(models.Machine).count() == 1
    assert db_session.query(models.Kernel).count() == 1
    assert db_session.query(models.Bios).count() == 1
    assert db_session.query(models.Release).count() == 1
    assert db_session.query(models.Certificate).count() == 1
    assert db_session.query(models.Report).count() == 1


def test_load_certificates_with_missing_kernel_bios(
    db_session: Session, requests_mock: Mocker
):
    """Test handling of missing kernel or BIOS data."""
    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
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
                        "supported_until": (
                            date.today() + timedelta(days=365)
                        ).strftime("%Y-%m-%d"),
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

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()

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

    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={
            "count": 0,
            "next": None,
            "previous": None,
            "results": [],
        },
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={
            "count": 1,
            "next": f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
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
                    "cpu_codename": "",
                }
            ],
        },
    )
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
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
                    "cpu_codename": "",
                }
            ],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()

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

    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={
            "count": 0,
            "next": None,
            "previous": None,
            "results": [],
        },
    )

    requests_mock.get(
        PUBLIC_DEVICES_URL,
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
                    "cpu_codename": "",
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
                    "cpu_codename": "",
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
                    "cpu_codename": "",
                },
            ],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()

    assert db_session.query(models.Device).count() == 2
    assert (
        db_session.query(models.Device).filter_by(subsystem="1458:1000").first()
        is not None
    )
    assert (
        db_session.query(models.Device).filter_by(subsystem="17aa:1036").first()
        is not None
    )


def test_load_devices_cpu_codename(
    db_session: Session,
    requests_mock: Mocker,
    test_client: TestClient,
    generator: DataGenerator,
):
    """
    Test that if a device has type PROCESSOR, we update device codename
    """
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Precision 3690 (ik12)")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration, canonical_id="202106-8087")
    certificate = generator.gen_certificate(
        machine, generator.gen_release(), name="2204-10681"
    )

    requests_mock.get(CPU_IDS_URL, json={})

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={
            "count": 0,
            "next": None,
            "previous": None,
            "results": [],
        },
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={
            "count": 1,
            "next": None,
            "previous": None,
            "results": [
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Intel CPU",
                        "subproduct_name": None,
                        "vendor": "Intel Corp.",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "1111:2222",
                        "subsystem": "102a:028c",
                        "version": None,
                        "category": "PROCESSOR",
                        "codename": "",
                    },
                    "driver_name": "bnx2",
                    "cpu_codename": "Skylake",
                }
            ],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()
    processor = (
        db_session.query(models.Device).filter_by(identifier="1111:2222").first()
    )

    assert processor is not None
    assert processor.codename == "Skylake"


def test_import_cpuids(
    db_session: Session, requests_mock: Mocker, test_client: TestClient
):
    requests_mock.get(
        CPU_IDS_URL,
        json={"Coffee Lake": ["0x806ea", "0x906ea"]},
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={
            "count": 0,
            "next": None,
            "previous": None,
            "results": [],
        },
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={
            "count": 1,
            "next": None,
            "previous": None,
            "results": [],
        },
    )

    c3_client = C3Client(db=db_session)
    c3_client.load_hardware_data()

    assert db_session.query(models.CpuId).count() == 2

    cpu_id = db_session.query(models.CpuId).filter_by(id_pattern="0x806ea").first()
    assert cpu_id is not None
    assert cpu_id.codename == "Coffee Lake"

    cpu_id = db_session.query(models.CpuId).filter_by(id_pattern="0x906ea").first()
    assert cpu_id is not None
    assert cpu_id.codename == "Coffee Lake"


def test_retry_on_read_timeout(db_session: Session, requests_mock: Mocker):
    """Test that client retries on ReadTimeout and eventually succeeds."""
    # First two requests timeout, third succeeds
    requests_mock.get(
        CPU_IDS_URL,
        [
            {"exc": requests.exceptions.ReadTimeout("Read timeout occurred")},
            {"exc": requests.exceptions.ReadTimeout("Read timeout occurred")},
            {"json": {"Coffee Lake": ["0x806ea"]}},
        ],
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)

    # Should succeed despite initial timeouts
    with patch("time.sleep") as mock_sleep:  # Speed up test by mocking sleep
        c3_client.load_hardware_data()

    # Verify it made retry attempts (2 sleeps for 2 failed attempts)
    assert mock_sleep.call_count == 2

    # Verify data was loaded successfully
    assert db_session.query(models.CpuId).count() == 1


def test_retry_on_connection_error(db_session: Session, requests_mock: Mocker):
    """Test that client retries on ConnectionError."""
    requests_mock.get(
        CPU_IDS_URL,
        [
            {"exc": requests.exceptions.ConnectionError("Connection failed")},
            {"json": {"Skylake": ["0x506e3"]}},
        ],
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    assert db_session.query(models.CpuId).count() == 1


def test_retry_on_server_errors(db_session: Session, requests_mock: Mocker):
    """Test that client retries on 5xx server errors."""
    requests_mock.get(
        CPU_IDS_URL,
        [
            {"status_code": 503, "text": "Service Unavailable"},
            {"status_code": 502, "text": "Bad Gateway"},
            {"json": {"Ice Lake": ["0x706e5"]}},
        ],
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    assert db_session.query(models.CpuId).count() == 1


def test_retry_on_rate_limit(db_session: Session, requests_mock: Mocker):
    """Test that client retries on 429 (Too Many Requests)."""
    requests_mock.get(
        CPU_IDS_URL,
        [
            {"status_code": 429, "text": "Too Many Requests"},
            {"json": {"Tiger Lake": ["0x806c1"]}},
        ],
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    assert db_session.query(models.CpuId).count() == 1


def test_no_retry_on_client_errors(db_session: Session, requests_mock: Mocker):
    """Test that client does NOT retry on 4xx client errors (except 429)."""
    requests_mock.get(CPU_IDS_URL, status_code=404, text="Not Found")

    c3_client = C3Client(db=db_session)

    # Should raise HTTPError immediately without retries
    with pytest.raises(requests.exceptions.HTTPError):
        c3_client._import_cpu_ids(CPU_IDS_URL)


def test_max_retries_exceeded(db_session: Session, requests_mock: Mocker):
    """Test that client eventually gives up after max retries."""
    # All 5 requests (1 initial + 4 retries) will timeout
    requests_mock.get(
        CPU_IDS_URL, exc=requests.exceptions.ReadTimeout("Persistent timeout")
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep") as mock_sleep:
        with pytest.raises(requests.exceptions.ReadTimeout):
            c3_client._import_cpu_ids(CPU_IDS_URL)

        # Should have made exactly 4 retry sleep calls (5 attempts - 1 initial = 4 retries)
        assert mock_sleep.call_count == 4


def test_exponential_backoff_timing(db_session: Session):
    """Test that exponential backoff delays increase correctly."""
    c3_client = C3Client(db=db_session)

    with (
        patch("time.sleep") as mock_sleep,
        patch.object(c3_client.session, "get") as mock_get,
    ):
        # Configure mock to always raise timeout
        mock_get.side_effect = requests.exceptions.ReadTimeout("Timeout")

        with pytest.raises(requests.exceptions.ReadTimeout):
            c3_client._make_request_with_retries("https://test.com")

        # Check that sleep was called with increasing delays: 2, 4, 8, 16
        # (4 retries for 5 total attempts)
        expected_delays = [2, 4, 8, 16]
        actual_delays = [call[0][0] for call in mock_sleep.call_args_list]
        assert actual_delays == expected_delays


def test_retry_with_pagination(
    db_session: Session, requests_mock: Mocker, generator: DataGenerator
):
    """Test retry logic works correctly with paginated responses."""
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Test Platform")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration, canonical_id="test-123")
    certificate = generator.gen_certificate(
        machine, generator.gen_release(), name="test-cert"
    )

    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    # First page succeeds immediately
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={
            "count": 2,
            "next": f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
            "previous": None,
            "results": [
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Test Device 1",
                        "subproduct_name": None,
                        "vendor": "Test Vendor",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "1111:2222",
                        "subsystem": "1111:3333",
                        "version": None,
                        "category": "OTHER",
                        "codename": "",
                    },
                    "driver_name": "test_driver",
                    "cpu_codename": "",
                }
            ],
        },
    )

    # Second page fails once, then succeeds
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
        [
            {"exc": requests.exceptions.ReadTimeout("Page 2 timeout")},
            {
                "json": {
                    "count": 2,
                    "next": None,
                    "previous": None,
                    "results": [
                        {
                            "machine_canonical_id": machine.canonical_id,
                            "certificate_name": certificate.name,
                            "device": {
                                "name": "Test Device 2",
                                "subproduct_name": None,
                                "vendor": "Test Vendor",
                                "device_type": None,
                                "bus": "pci",
                                "identifier": "3333:4444",
                                "subsystem": "3333:5555",
                                "version": None,
                                "category": "OTHER",
                                "codename": "",
                            },
                            "driver_name": "test_driver2",
                            "cpu_codename": "",
                        }
                    ],
                }
            },
        ],
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    # Both devices should be loaded despite the timeout on page 2
    assert db_session.query(models.Device).count() == 2
    assert (
        db_session.query(models.Device).filter_by(identifier="1111:2222").first()
        is not None
    )
    assert (
        db_session.query(models.Device).filter_by(identifier="3333:4444").first()
        is not None
    )


def test_retry_logging(db_session: Session, requests_mock: Mocker, caplog):
    """Test that retry attempts are properly logged."""
    requests_mock.get(
        CPU_IDS_URL,
        [
            {"exc": requests.exceptions.ReadTimeout("First timeout")},
            {"exc": requests.exceptions.ReadTimeout("Second timeout")},
            {"json": {"Test": ["0x12345"]}},
        ],
    )

    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    # Check that retry warnings were logged
    warning_logs = [
        record for record in caplog.records if record.levelname == "WARNING"
    ]
    assert len(warning_logs) == 2
    assert "Retrying in" in warning_logs[0].message
    assert "attempt 1/5" in warning_logs[0].message
    assert "attempt 2/5" in warning_logs[1].message


def test_session_reuse(db_session: Session):
    """Test that the same session with retry configuration is reused."""
    c3_client = C3Client(db=db_session)

    assert c3_client.session is not None
    assert hasattr(c3_client.session, "adapters")

    # Make multiple calls and ensure same session is used
    session1 = c3_client.session
    session2 = c3_client.session
    assert session1 is session2


def test_intermittent_failures_recovery(
    db_session: Session, requests_mock: Mocker, generator: DataGenerator
):
    """Test recovery from intermittent failures during large data imports."""
    # Set up test data
    vendor = generator.gen_vendor()
    platform = generator.gen_platform(vendor, name="Test Platform")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration, canonical_id="test-456")
    certificate = generator.gen_certificate(
        machine, generator.gen_release(), name="test-cert-2"
    )

    requests_mock.get(CPU_IDS_URL, json={})
    requests_mock.get(
        PUBLIC_CERTIFICATES_URL,
        json={"count": 0, "next": None, "previous": None, "results": []},
    )

    # Simulate intermittent failures: success, failure, success pattern
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}",
        [
            # First request succeeds
            {
                "json": {
                    "count": 3,
                    "next": f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
                    "results": [
                        {
                            "machine_canonical_id": machine.canonical_id,
                            "certificate_name": certificate.name,
                            "device": {
                                "name": "Device 1",
                                "subproduct_name": None,
                                "vendor": "Vendor 1",
                                "device_type": None,
                                "bus": "pci",
                                "identifier": "aaaa:bbbb",
                                "subsystem": "aaaa:cccc",
                                "version": None,
                                "category": "OTHER",
                                "codename": "",
                            },
                            "driver_name": "driver1",
                            "cpu_codename": "",
                        }
                    ],
                }
            }
        ],
    )

    # Second page: fails then succeeds
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=1000",
        [
            {
                "exc": requests.exceptions.ConnectTimeout(
                    "Intermittent connection issue"
                )
            },
            {
                "json": {
                    "count": 3,
                    "next": f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=2000",
                    "results": [
                        {
                            "machine_canonical_id": machine.canonical_id,
                            "certificate_name": certificate.name,
                            "device": {
                                "name": "Device 2",
                                "subproduct_name": None,
                                "vendor": "Vendor 2",
                                "device_type": None,
                                "bus": "pci",
                                "identifier": "cccc:dddd",
                                "subsystem": "cccc:eeee",
                                "version": None,
                                "category": "OTHER",
                                "codename": "",
                            },
                            "driver_name": "driver2",
                            "cpu_codename": "",
                        }
                    ],
                }
            },
        ],
    )

    # Third page: succeeds immediately
    requests_mock.get(
        f"{PUBLIC_DEVICES_URL}{get_limit_offset(1000)}&offset=2000",
        json={
            "count": 3,
            "next": None,
            "results": [
                {
                    "machine_canonical_id": machine.canonical_id,
                    "certificate_name": certificate.name,
                    "device": {
                        "name": "Device 3",
                        "subproduct_name": None,
                        "vendor": "Vendor 3",
                        "device_type": None,
                        "bus": "pci",
                        "identifier": "eeee:ffff",
                        "subsystem": "eeee:aaaa",
                        "version": None,
                        "category": "OTHER",
                        "codename": "",
                    },
                    "driver_name": "driver3",
                    "cpu_codename": "",
                }
            ],
        },
    )

    c3_client = C3Client(db=db_session)

    with patch("time.sleep"):
        c3_client.load_hardware_data()

    # All devices should be successfully imported despite intermittent failure
    assert db_session.query(models.Device).count() == 3
    device_identifiers = [d.identifier for d in db_session.query(models.Device).all()]
    assert "aaaa:bbbb" in device_identifiers
    assert "cccc:dddd" in device_identifiers
    assert "eeee:ffff" in device_identifiers
