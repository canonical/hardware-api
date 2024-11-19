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

from pytest import LogCaptureFixture
from fastapi.testclient import TestClient
from tests.data_generator import DataGenerator, CertificationRequest


def test_vendor_not_found(generator: DataGenerator, test_client: TestClient):
    request = CertificationRequest.create_default_request(vendor_name="Unknown vendor")
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}


def test_board_mismatch(
    caplog: LogCaptureFixture,
    generator: DataGenerator,
    test_client: TestClient,
):
    """Setup where vendor exists but board does not match"""
    vendor = generator.gen_vendor()
    generator.gen_board(
        vendor,
        identifier="dmi:0001",
        name="Different Board",
        version="Different Version",
    )

    request = CertificationRequest.create_default_request()
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}
    assert "Hardware cannot be found" in caplog.text
    assert f"board model: {request['board']['product_name']}" in caplog.text


def test_bios_mismatch(
    caplog: LogCaptureFixture,
    generator: DataGenerator,
    test_client: TestClient,
):
    """
    Setup where board model and vendor match but bios does not match.
    The mismatch should be logged
    """
    vendor = generator.gen_vendor()
    board = generator.gen_board(
        vendor,
        identifier="dmi:0001",
        name="FW00Q",
        version="0001A",
    )
    generator.gen_bios(vendor=vendor, version="1.0.12")

    request = CertificationRequest.create_default_request(
        vendor_name=vendor.name,
        board_name=f"{board.vendor.name} Ltd.",
        bios_vendor=vendor.name,
        bios_version="1.0.15",
        bios_revision="1",
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}
    assert "Hardware cannot be found" in caplog.text
    assert f"bios version: {request['bios']['version']}" in caplog.text


def test_disqualifying_hardware(generator: DataGenerator, test_client: TestClient):
    """A scenario where CPU codename is mismatched"""
    vendor = generator.gen_vendor()
    bios = generator.gen_bios(vendor, version="1.0")
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(certificate, generator.gen_kernel(), bios)
    generator.gen_processor(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        identifier="dmi:1111",
        codename="Ivy Bridge",
        reports=[report],
    )
    generator.gen_cpuid_object("0x306a", "Ivy Bridge")
    generator.gen_cpuid_object("0x806e9", "Amber Lake")
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        bios_vendor=vendor.name,
        bios_version=bios.version,
        bios_revision=bios.revision,
        # Amber Lake CPU ID
        processor_id=[0xE9, 0x06, 0x08, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
        processor_version="Intel(R) Xeon(R) CPU D-1548 @ 1.40GHz",
    )

    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Related Certified System Exists",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "bios": {
            "vendor": bios.vendor.name,
            "version": bios.version,
            "revision": bios.revision,
            "firmware_revision": bios.firmware_revision,
            "release_date": bios.release_date.strftime("%Y-%m-%d"),
        },
        "chassis": None,
        "gpu": None,
        "audio": None,
        "video": None,
        "network": None,
        "wireless": None,
        "pci_peripherals": [],
        "usb_peripherals": [],
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": release.release,
                "codename": release.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }


def test_correct_hardware_unmatching_os(
    generator: DataGenerator, test_client: TestClient
):
    """Setup where hardware matches but the OS version does not match any certified images"""
    vendor = generator.gen_vendor()
    bios = generator.gen_bios(vendor, version="1.0")
    focal = generator.gen_release(codename="focal", release="20.04")
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, focal)
    report = generator.gen_report(certificate, generator.gen_kernel(), bios)
    processor = generator.gen_processor(
        vendor=generator.gen_vendor(name="Intel Corp."),
        codename="Knights Landing",
        reports=[report],
    )
    generator.gen_cpuid_object("0x5067", processor.codename)
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        bios_vendor=bios.vendor.name,
        bios_version=bios.version,
        bios_revision=bios.revision,
        # Knights Landing CPU ID
        processor_id=[0x71, 0x06, 0x05, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified Image Exists",
        "architecture": "amd64",
        "bios": {
            "vendor": bios.vendor.name,
            "version": bios.version,
            "revision": bios.revision,
            "release_date": bios.release_date.strftime("%Y-%m-%d"),
            "firmware_revision": bios.firmware_revision,
        },
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": focal.release,
                "codename": focal.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }


def test_all_criteria_matched(generator: DataGenerator, test_client: TestClient):
    """Setup where everything matches including the OS"""
    vendor = generator.gen_vendor()
    bios = generator.gen_bios(vendor, version="1.0")
    release = generator.gen_release(release="22.04", codename="jammy")
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(certificate, generator.gen_kernel(), bios)
    processor = generator.gen_processor(
        vendor=generator.gen_vendor(name="Intel Corp."), reports=[report]
    )
    generator.gen_cpuid_object("0xb0671", processor.codename)
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        bios_vendor=bios.vendor.name,
        bios_version=bios.version,
        bios_revision=bios.revision,
        os_version=release.release,
        os_codename=release.codename,
        # Raport Lake CPU ID
        processor_id=[0x71, 0x06, 0x0B, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": {
            "vendor": bios.vendor.name,
            "version": bios.version,
            "revision": bios.revision,
            "release_date": bios.release_date.strftime("%Y-%m-%d"),
            "firmware_revision": bios.firmware_revision,
        },
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": release.release,
                "codename": release.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }


def test_bios_is_none(generator: DataGenerator, test_client: TestClient):
    """
    Test that Bios information is not required to be sent (some devices may not have it)
    """
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(certificate, generator.gen_kernel())
    processor = generator.gen_processor(
        vendor=generator.gen_vendor(name="AMD"), codename="AMD Genoa", reports=[report]
    )
    generator.gen_cpuid_object("0xa10f11", processor.codename)
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        # AMD Genoa CPU ID
        processor_id=[0x11, 0x0F, 0xA1, 0x00, 0xFF, 0xFB, 0x8B, 0x17],
        processor_version="4th Gen AMD EPYC 9354 / 9124",
        processor_vendor=processor.vendor.name,
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": None,
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": release.release,
                "codename": release.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }


def test_cpu_id_is_none(generator: DataGenerator, test_client: TestClient):
    """
    Test if CPU IS is not specified but the version matches, we get the certified response
    """
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(certificate, generator.gen_kernel())
    processor = generator.gen_processor(
        vendor=generator.gen_vendor(name="Intel Corp."), codename="", reports=[report]
    )
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        processor_version=processor.version
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": None,
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": release.release,
                "codename": release.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }


def test_hardware_matches_multiple_bios(
    generator: DataGenerator, test_client: TestClient
):
    """
    Test where multiple BIOSes of the same vendor and version exist in the DB
    but only one of them corresponds the machine with the matching board
    """
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    # Generate multiple BIOS versions
    generator.gen_bios(vendor, version="1.2.0", revision="1.1")
    generator.gen_bios(vendor, version="1.2.0", revision="1.2")
    bios = generator.gen_bios(vendor, version="1.2.0", revision="1.4")
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(certificate, generator.gen_kernel(), bios)
    processor = generator.gen_processor(
        vendor=generator.gen_vendor(name="Intel Corp."),
        codename="Coffee Lake",
        reports=[report],
    )
    generator.gen_cpuid_object("0x906ea", processor.codename)
    board = generator.gen_board(vendor, reports=[report])

    request = CertificationRequest.create_default_request(
        bios_vendor=bios.vendor.name,
        bios_version=bios.version,
        bios_revision=bios.revision,
        # Coffeee Lake CPU ID
        processor_id=[0xEA, 0x06, 0x09, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
    )
    response = test_client.post("/v1/certification/status", json=request)

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": {
            "vendor": bios.vendor.name,
            "version": bios.version,
            "revision": bios.revision,
            "release_date": bios.release_date.strftime("%Y-%m-%d"),
            "firmware_revision": bios.firmware_revision,
        },
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "available_releases": [
            {
                "distributor": "Ubuntu",
                "version": release.release,
                "codename": release.codename,
                "kernel": {
                    "name": report.kernel.name,
                    "version": report.kernel.version,
                    "signature": report.kernel.signature,
                    "loaded_modules": [],
                },
            }
        ],
    }
