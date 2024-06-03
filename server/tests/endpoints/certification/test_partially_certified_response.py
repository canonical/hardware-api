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

from hwapi.data_models.enums import DeviceCategory, BusType
from tests.data_generator import DataGenerator


def test_correct_architecture(generator: DataGenerator, test_client: TestClient):
    """
    We shall select devices only from machines that have the same achitecture even if
    a device in the request has been seen on a certified system but it has a different
    architecture
    """
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    arm_machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="arm64 config",
            platform=generator.gen_platform(name="arm64 platform", vendor=vendor),
        ),
    )
    arm_certificate = generator.gen_certificate(arm_machine, release)
    arm_report = generator.gen_report(
        arm_certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="arm64",
    )
    network_card_on_arm = generator.gen_device(
        vendor=vendor,
        name="Network card on arm device",
        bus=BusType.pci,
        category=DeviceCategory.NETWORK,
        identifier="0000:1111",
        reports=[arm_report],
    )

    amd_machine = generator.gen_machine(
        canonical_id="202401-00002",
        configuration=generator.gen_configuration(
            name="amd64 config",
            platform=generator.gen_platform(name="amd64 platform", vendor=vendor),
        ),
    )
    amd_certificate = generator.gen_certificate(amd_machine, release)
    amd_report = generator.gen_report(
        amd_certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    board = generator.gen_device(
        vendor=generator.gen_vendor(name="Dell Inc."),
        identifier="dmi:OFOW8W",
        name="0F0W8W",
        version="A00",
        bus=BusType.dmi,
        category=DeviceCategory.BOARD,
        reports=[amd_report],
    )
    network_card_on_amd = generator.gen_device(
        vendor=vendor,
        name="Network card on amd device",
        bus=BusType.pci,
        category=DeviceCategory.NETWORK,
        identifier="0000:0001",
        reports=[amd_report],
    )
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": amd_report.architecture,
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "network": [
                {
                    "bus": network_card_on_arm.bus,
                    "identifier": network_card_on_arm.identifier,
                    "model": network_card_on_arm.name,
                    "vendor": network_card_on_arm.vendor.name,
                    "capacity": 1000,
                },
                {
                    "bus": network_card_on_amd.bus,
                    "identifier": network_card_on_amd.identifier,
                    "model": network_card_on_amd.name,
                    "vendor": network_card_on_amd.vendor.name,
                    "capacity": 10000,
                },
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partial Fail",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "processor": [],
        "gpu": [],
        "audio": [],
        "video": [],
        "network": [
            {
                "bus": network_card_on_amd.bus,
                "identifier": network_card_on_amd.identifier,
                "model": network_card_on_amd.name,
                "vendor": network_card_on_amd.vendor.name,
                "capacity": 10000,
            },
        ],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response


def test_matching_release(generator: DataGenerator, test_client: TestClient):
    """
    If OS release is specified, we shall return devices from the machines that
    were certified for the same OS release.
    """
    vendor = generator.gen_vendor()
    jammy_release = generator.gen_release(codename="jammy", release="22.04")
    jammy_machine = generator.gen_machine(
        canonical_id="202401-00001",
        configuration=generator.gen_configuration(
            name="config1",
            platform=generator.gen_platform(name="platform1", vendor=vendor),
        ),
    )
    jammy_certificate = generator.gen_certificate(jammy_machine, jammy_release)
    jammy_report = generator.gen_report(
        jammy_certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    processor_on_jammy = generator.gen_device(
        vendor=generator.gen_vendor(name="Advanced Micro Devices [AMD]"),
        name="AMD EPYC 3251 8-Core Processor",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:AMDEPYC32518-CoreProcessor",
        reports=[jammy_report],
    )

    noble_release = generator.gen_release(codename="noble", release="24.04")
    noble_machine = generator.gen_machine(
        canonical_id="202401-00002",
        configuration=generator.gen_configuration(
            name="config2",
            platform=generator.gen_platform(name="platform2", vendor=vendor),
        ),
    )
    noble_certificate = generator.gen_certificate(noble_machine, noble_release)
    noble_report = generator.gen_report(
        noble_certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    gpu_on_noble = generator.gen_device(
        vendor=generator.gen_vendor(name="nVidia"),
        name="Some GPU",
        identifier="10de:25bc",
        bus=BusType.pci,
        category=DeviceCategory.VIDEO,
        reports=[noble_report],
    )

    board = generator.gen_device(
        vendor=generator.gen_vendor(name="Dell Inc."),
        identifier="dmi:OFOW8W",
        name="0F0W8W",
        version="A00",
        bus=BusType.dmi,
        category=DeviceCategory.BOARD,
        reports=[noble_report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": noble_report.architecture,
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": "24.04",
                "codename": "noble",
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
                "loaded_modules": [],
            },
            "processor": [
                {
                    "family": "some family",
                    "frequency": 2.4,
                    "manufacturer": processor_on_jammy.vendor.name,
                    "version": processor_on_jammy.name,
                }
            ],
            "gpu": [
                {
                    "manufacturer": gpu_on_noble.vendor.name,
                    "version": gpu_on_noble.name,
                    "identifier": gpu_on_noble.identifier,
                }
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partial Success",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "processor": [],
        "gpu": [
            {
                "family": None,
                "manufacturer": gpu_on_noble.vendor.name,
                "version": gpu_on_noble.name,
                "identifier": gpu_on_noble.identifier,
            }
        ],
        "audio": [],
        "video": [],
        "network": [],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response


def test_no_matching_board(generator: DataGenerator, test_client: TestClient):
    """
    If we cannot find the same motherboard, we should return not certified response
    """
    vendor = generator.gen_vendor()
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": "amd64",
            "board": {
                "manufacturer": "unexisting vendor",
                "product_name": "sample board",
                "version": "1.1.1",
            },
            "os": None,
        },
    )
    assert response.status_code == 200
    assert response.json() == {"status": "Not Seen"}


def test_partially_certified_processor_match(
    generator: DataGenerator, test_client: TestClient
):
    vendor = generator.gen_vendor()
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
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    board = generator.gen_device(
        vendor=generator.gen_vendor(name="Dell Inc."),
        identifier="dmi:OFOW8W",
        name="0F0W8W",
        version="A00",
        bus=BusType.dmi,
        category=DeviceCategory.BOARD,
        reports=[report],
    )
    intel_vendor = generator.gen_vendor(name="Intel Corp.")
    processor = generator.gen_device(
        vendor=intel_vendor,
        name="Intel(R) Xeon(R) Gold 6338N CPU @ 2.20GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:Intel(R)Xeon(R)Gold6338NCPU@2.20GHz",
        reports=[report],
    )
    generator.gen_device(
        vendor=intel_vendor,
        name="Intel(R) Xeon(R) Gold 6328H CPU @ 2.80GHz",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:Intel(R)Xeon(R)Gold6328HCPU@2.80GHz",
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": report.architecture,
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
                "loaded_modules": [],
            },
            "processor": [
                {
                    "family": "3rd Generation Xeon",
                    "frequency": 2.20,
                    "manufacturer": "Intel(R)",
                    "version": "Intel(R) Xeon(R) Gold 6338N CPU",
                }
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partial Success",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "processor": [
            {
                "family": "3rd Generation Xeon",
                "frequency": 2.20,
                "manufacturer": processor.vendor.name,
                "version": processor.name,
            }
        ],
        "gpu": [],
        "audio": [],
        "video": [],
        "network": [],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response


def test_gpu_match(generator: DataGenerator, test_client: TestClient):
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    board = generator.gen_device(
        vendor=generator.gen_vendor(name="Dell Inc."),
        identifier="dmi:OFOW8W",
        name="0F0W8W",
        version="A00",
        bus=BusType.dmi,
        category=DeviceCategory.BOARD,
        reports=[report],
    )
    nvidia_vendor = generator.gen_vendor(name="nVidia")
    gpu = generator.gen_device(
        vendor=nvidia_vendor,
        name="GA104M [GeForce RTX 3070 Mobile / Max-Q]",
        bus=BusType.pci,
        category=DeviceCategory.VIDEO,
        identifier="10de:249d",
        reports=[report],
    )
    generator.gen_device(
        vendor=nvidia_vendor,
        name="GA104M [GeForce RTX 3080 Mobile / Max-Q 8GB/16GB]",
        bus=BusType.pci,
        category=DeviceCategory.VIDEO,
        identifier="10de:249c",
        reports=[report],
    )
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": report.architecture,
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
                "loaded_modules": [],
            },
            "gpu": [
                {
                    "manufacturer": "NVIDIA",
                    "version": gpu.name,
                    "identifier": gpu.identifier.upper(),
                },
                {
                    "manufacturer": "NVIDIA",
                    "version": "Another GPU",
                    "identifier": "10de:0011",
                },
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partial Success",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "gpu": [
            {
                "family": None,
                "manufacturer": gpu.vendor.name,
                "version": gpu.name,
                "identifier": gpu.identifier,
            }
        ],
        "processor": [],
        "audio": [],
        "video": [],
        "network": [],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response


def test_network_device_match(generator: DataGenerator, test_client: TestClient):
    vendor = generator.gen_vendor()
    release = generator.gen_release()
    machine = generator.gen_machine(
        configuration=generator.gen_configuration(
            platform=generator.gen_platform(vendor=vendor),
        ),
    )
    certificate = generator.gen_certificate(machine, release)
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        generator.gen_bios(vendor),
        architecture="amd64",
    )
    board = generator.gen_device(
        vendor=generator.gen_vendor(name="Dell Inc."),
        identifier="dmi:OFOW8W",
        name="0F0W8W",
        version="A00",
        bus=BusType.dmi,
        category=DeviceCategory.BOARD,
        reports=[report],
    )
    intel_vendor = generator.gen_vendor(name="Intel Corp.")
    network_correct_category = generator.gen_device(
        vendor=intel_vendor,
        name="Ethernet Controller I225-LM",
        bus=BusType.pci,
        category=DeviceCategory.NETWORK,
        identifier="8086:15f2",
        reports=[report],
    )
    network_other_category = generator.gen_device(
        vendor=intel_vendor,
        name="Ethernet Controller I225-IT",
        bus=BusType.pci,
        category=DeviceCategory.OTHER,
        identifier="8086:125d",
        reports=[report],
    )
    generator.gen_device(
        vendor=intel_vendor,
        name="Ethernet Controller I226-V",
        bus=BusType.pci,
        category=DeviceCategory.NETWORK,
        identifier="8086:249c",
        reports=[report],
    )
    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": vendor.name,
            "model": "Some model",
            "architecture": report.architecture,
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "os": {
                "distributor": "Canonical Ltd.",
                "version": release.release,
                "codename": release.codename,
                "kernel": {"name": None, "version": "5.7.1-generic", "signature": None},
                "loaded_modules": [],
            },
            "network": [
                {
                    "bus": network_correct_category.bus,
                    "identifier": network_correct_category.identifier,
                    "model": network_correct_category.name,
                    "vendor": "Intel(R)",
                    "capacity": 1000,
                },
                {
                    "bus": network_other_category.bus,
                    "identifier": network_other_category.identifier,
                    "model": network_other_category.name,
                    "vendor": "Intel(R)",
                    "capacity": 1000,
                },
                {
                    "bus": "pci",
                    "identifier": "8086:112a",
                    "model": "Another network device",
                    "vendor": "Intel(R)",
                    "capacity": 1000,
                },
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partial Success",
        "architecture": "amd64",
        "board": {
            "manufacturer": board.vendor.name,
            "product_name": board.name,
            "version": board.version,
        },
        "chassis": None,
        "processor": [],
        "gpu": [],
        "audio": [],
        "video": [],
        "network": [
            {
                "bus": network_correct_category.bus,
                "identifier": network_correct_category.identifier,
                "model": network_correct_category.name,
                "vendor": network_correct_category.vendor.name,
                "capacity": 1000,
            },
            {
                "bus": network_other_category.bus,
                "identifier": network_other_category.identifier,
                "model": network_other_category.name,
                "vendor": network_other_category.vendor.name,
                "capacity": 1000,
            },
        ],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response
