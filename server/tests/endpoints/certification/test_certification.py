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

from hwapi.data_models.enums import CertificationStatus, DeviceCategory, BusType
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


def test_partially_certified_status(generator: DataGenerator, test_client: TestClient):
    """
    If some components of the system were seen on other machines but system itself wasn't seen,
    we should get partially certified response with the list of these devices that are present
    on the certified system
    """
    dell_vendor = generator.gen_vendor()
    platform = generator.gen_platform(dell_vendor, name="Precision 3690 (ik12)")
    configuration = generator.gen_configuration(platform)
    machine = generator.gen_machine(configuration)
    certificate = generator.gen_certificate(machine, generator.gen_release())
    report = generator.gen_report(
        certificate,
        generator.gen_kernel(),
        generator.gen_bios(dell_vendor),
        architecture="amd64",
    )
    processor = generator.gen_device(
        vendor=generator.gen_vendor(name="Advanced Micro Devices [AMD]"),
        name="AMD EPYC 3251 8-Core Processor",
        bus=BusType.dmi,
        category=DeviceCategory.PROCESSOR,
        identifier="dmi:AMDEPYC32518-CoreProcessor",
        reports=[report],
    )
    network_adapter = generator.gen_device(
        vendor=generator.gen_vendor(name="Cisco Systems"),
        name="VIC Ethernet NIC",
        identifier="1137:0043",
        reports=[report],
    )

    response = test_client.post(
        "/v1/certification/status",
        json={
            "vendor": "Unexsting vendor",
            "model": "Some model",
            "architecture": report.architecture,
            "board": {
                "manufacturer": "Dell Inc.",
                "product_name": "sample board",
                "version": "1.1.1",
            },
            "processor": [
                {
                    "version": processor.name,
                    "family": "EPYC",
                    "frequency": 2.5,
                    "manufacturer": f"{processor.vendor.name} Inc.",
                }
            ],
            "gpu": [
                {
                    "manufacturer": "nVidia",
                    "version": "GA107M [GeForce MX570]",
                    "identifier": "10DE:25A6",
                },
                {
                    "manufacturer": "Intel Corp.",
                    "version": "1ADSF",
                    "identifier": "8086:012b",
                },
            ],
            "network": [
                {
                    "bus": network_adapter.bus,
                    "identifier": network_adapter.identifier,
                    "model": network_adapter.name,
                    "vendor": "AMD",
                    "capacity": 1000,
                },
                {
                    "bus": BusType.pci,
                    "identifier": "8086:1572",
                    "model": "Ethernet Controller X710 for 10GbE SFP+",
                    "vendor": "Intel Corp.",
                    "capacity": 10000,
                },
            ],
        },
    )
    assert response.status_code == 200
    expected_response = {
        "status": "Partially Certified",
        "architecture": "amd64",
        "board": None,
        "chassis": None,
        "processor": [
            {
                "family": "EPYC",
                "frequency": 2.5,
                "manufacturer": "Advanced Micro Devices [AMD]",
                "version": "AMD EPYC 3251 8-Core Processor",
            }
        ],
        "gpu": [],
        "audio": [],
        "video": [],
        "network": [
            {
                "bus": "pci",
                "identifier": "1137:0043",
                "model": "VIC Ethernet NIC",
                "vendor": "Cisco Systems",
                "capacity": 1000,
            },
        ],
        "wireless": [],
        "pci_peripherals": [],
        "usb_peripherals": [],
    }

    assert response.json() == expected_response


def test_partially_certified_invalid_release(
    generator: DataGenerator, test_client: TestClient
):
    """
    If OS release is an invalid ubuntu release, we should get 400 response
    """
    generator.gen_release()
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
            "os": {
                "distributor": "Ubuntu",
                "version": "0.0",
                "codename": "aaaaa",
                "kernel": {"version": "dsds", "name": None, "signature": None},
                "loaded_modules": [],
            },
        },
    )
    assert response.status_code == 400
    assert "No matching release found" in response.json()["detail"]
