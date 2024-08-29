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

from hwapi.data_models.enums import BusType, DeviceCategory

from tests.data_generator import DataGenerator


def test_vendor_not_found(generator: DataGenerator, test_client: TestClient):
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": "Nonexistent Vendor",
            "model": "Unknown Model",
            "architecture": "amd64",
            "board": {
                "manufacturer": "Dell Inc.",
                "product_name": "sample board",
                "version": "1.1.1",
            },
            "bios": {
                "vendor": "Dell",
                "version": "1.0",
                "revision": "1",
                "release_date": None,
                "firmware_revision": None,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": "24.04",
                "codename": "noble",
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                "identifier": [],
                "frequency": 2400,
                "manufacturer": "AMD",
                "version": "AMD EPYC 3251 8-Core Processor",
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}


def test_hardware_mismatch(generator: DataGenerator, test_client: TestClient):
    """Setup where vendor exists but board and bios do not match"""
    vendor = generator.gen_vendor(name="Known Vendor")
    generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Different Board",
        version="Different Version",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
    )
    generator.gen_bios(vendor, version="Different BIOS Version")

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some Model",
            "architecture": "amd64",
            "board": {
                "manufacturer": "Dell Inc.",
                "product_name": "sample board",
                "version": "1.1.1",
            },
            "bios": {
                "vendor": vendor.name,
                "version": "1.0",
                "revision": "1",
                "release_date": None,
                "firmware_revision": None,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": "24.04",
                "codename": "noble",
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                "identifier": [],
                "frequency": 2400,
                "manufacturer": "AMD",
                "version": "AMD EPYC 3251 8-Core Processor",
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}


def test_disqualifying_hardware(generator: DataGenerator, test_client: TestClient):
    """A scenario where CPU codename is mismatched"""
    vendor = generator.gen_vendor(name="Known Vendor")
    bios = generator.gen_bios(vendor, version="1.0")
    release = generator.gen_release()
    machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config",
            platform=generator.gen_platform(name="platform", vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        bios,
    )
    # generate processor
    generator.gen_device(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:1111",
        codename="Ivy Bridge",
        reports=[report],
    )
    generator.gen_cpuid_object("0x306a", "Ivy Bridge")
    generator.gen_cpuid_object("0x806e9", "Amber Lake")
    board = generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Matching Board",
        version="v1.0",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Model with Disqualifying Hardware",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": {
                "vendor": bios.vendor.name,
                "version": bios.version,
                "revision": bios.revision,
                "release_date": bios.release_date.strftime("%Y-%m-%d"),
                "firmware_revision": bios.firmware_revision,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": "24.04",
                "codename": "noble",
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                # Amber Lake CPU ID
                "identifier": [0xE9, 0x06, 0x08, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
                "frequency": 2000,
                "manufacturer": "Intel Corp.",
                "version": "Intel(R) Xeon(R) CPU D-1548 @ 20000GHz",
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {
        "status": "Related Certified System Exists",
        "architecture": "amd64",
        "board": {
            "manufacturer": "Known Vendor",
            "product_name": "Matching Board",
            "version": "v1.0",
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


def test_correct_hardware_incorrect_os(
    generator: DataGenerator, test_client: TestClient
):
    """Setup where hardware matches but the OS version does not match any certified images"""
    vendor = generator.gen_vendor(name="Known Vendor")
    bios = generator.gen_bios(vendor, version="1.0")
    focal = generator.gen_release(codename="focal", release="20.04")
    machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config",
            platform=generator.gen_platform(name="platform", vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, focal)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        bios,
    )
    processor = generator.gen_device(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:1111",
        codename="Knights Landing",
        reports=[report],
    )
    generator.gen_cpuid_object("0x5067", processor.codename)
    board = generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Matching Board",
        version="v1.0",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Model with Disqualifying Hardware",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": {
                "vendor": bios.vendor.name,
                "version": bios.version,
                "revision": bios.revision,
                "release_date": bios.release_date.strftime("%Y-%m-%d"),
                "firmware_revision": bios.firmware_revision,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": "24.04",
                "codename": "noble",
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                # Knights Landing CPU ID
                "identifier": [0x71, 0x06, 0x05, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
                "frequency": 2000,
                "manufacturer": "Intel Corp.",
                "version": "Intel(R) Core(TM) i7-7600U CPU @ 24000GHz",
            },
        },
    )

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
            "manufacturer": "Known Vendor",
            "product_name": "Matching Board",
            "version": "v1.0",
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
    vendor = generator.gen_vendor(name="Known Vendor")
    bios = generator.gen_bios(vendor, version="1.0")
    release = generator.gen_release()
    machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config",
            platform=generator.gen_platform(name="platform", vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        bios,
    )
    processor = generator.gen_device(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:1111",
        codename="Raptor Lake",
        reports=[report],
    )
    generator.gen_cpuid_object("0xb0671", processor.codename)
    board = generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Matching Board",
        version="v1.0",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Model with Disqualifying Hardware",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": {
                "vendor": bios.vendor.name,
                "version": bios.version,
                "revision": bios.revision,
                "release_date": bios.release_date.strftime("%Y-%m-%d"),
                "firmware_revision": bios.firmware_revision,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                # Raptor Lake CPU ID
                "identifier": [0x71, 0x06, 0x0B, 0x00, 0xFF, 0xFB, 0xEB, 0xBF],
                "frequency": 2000,
                "manufacturer": "Intel Corp.",
                "version": "Intel(R) Core(TM) i7-7600U CPU @ 24000GHz",
            },
        },
    )

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
            "manufacturer": "Known Vendor",
            "product_name": "Matching Board",
            "version": "v1.0",
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
    vendor = generator.gen_vendor(name="Known Vendor")
    release = generator.gen_release()
    machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config",
            platform=generator.gen_platform(name="platform", vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
    )
    processor = generator.gen_device(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:1111",
        codename="AMD Genoa",
        reports=[report],
    )
    generator.gen_cpuid_object("0xa10f11", processor.codename)
    board = generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Matching Board",
        version="v1.0",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Model with Disqualifying Hardware",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                # AMD Genoa CPU ID
                "identifier": [0x11, 0x0F, 0xA1, 0x00, 0xFF, 0xFB, 0x8B, 0x17],
                "frequency": 2000,
                "manufacturer": "Intel Corp.",
                "version": "Intel(R) Core(TM) i7-7600U CPU @ 24000GHz",
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": None,
        "board": {
            "manufacturer": "Known Vendor",
            "product_name": "Matching Board",
            "version": "v1.0",
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
    vendor = generator.gen_vendor(name="Known Vendor")
    release = generator.gen_release()
    machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config",
            platform=generator.gen_platform(name="platform", vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
    )
    processor = generator.gen_device(
        vendor=generator.gen_vendor(name="Intel Corp."),
        name="Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:1111",
        codename="",
        reports=[report],
    )
    board = generator.gen_device(
        vendor,
        identifier="dmi:0001",
        name="Matching Board",
        version="v1.0",
        category=DeviceCategory.BOARD,
        bus=BusType.dmi,
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Model with Disqualifying Hardware",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
            },
            "processor": {
                # AMD Genoa CPU ID
                "identifier": None,
                "frequency": 2000,
                "manufacturer": "Intel Corp.",
                "version": processor.version,
            },
        },
    )

    assert response.status_code == 200
    assert response.json() == {
        "status": "Certified",
        "architecture": "amd64",
        "bios": None,
        "board": {
            "manufacturer": "Known Vendor",
            "product_name": "Matching Board",
            "version": "v1.0",
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
