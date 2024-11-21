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

from datetime import datetime, timedelta, date
from sqlalchemy.orm import Session
from hwapi.data_models import models
from hwapi.data_models.enums import BusType, DeviceCategory


class DataGenerator:
    def __init__(self, db_session: Session):
        self.db_session = db_session

    def gen_vendor(self, name: str = "Dell") -> models.Vendor:
        vendor = models.Vendor(name=name)
        self.db_session.add(vendor)
        self.db_session.commit()
        return vendor

    def gen_platform(
        self, vendor: models.Vendor, name: str = "ChengMing 3980"
    ) -> models.Platform:
        platform = models.Platform(vendor=vendor, name=name)
        self.db_session.add(platform)
        self.db_session.commit()
        return platform

    def gen_configuration(
        self, platform: models.Platform, name: str = "i3-9100"
    ) -> models.Configuration:
        configuration = models.Configuration(platform=platform, name=name)
        self.db_session.add(configuration)
        self.db_session.commit()
        return configuration

    def gen_machine(
        self, configuration: models.Configuration, canonical_id: str = "202401-10000"
    ) -> models.Machine:
        machine = models.Machine(configuration=configuration, canonical_id=canonical_id)
        self.db_session.add(machine)
        self.db_session.commit()
        return machine

    def gen_release(
        self,
        codename: str = "noble",
        release: str = "24.04",
        release_date: date = datetime.now().date() - timedelta(days=365),
        supported_until: date = datetime.now().date() + timedelta(days=3650),
    ) -> models.Release:
        created_release = models.Release(
            codename=codename,
            release=release,
            release_date=release_date,
            supported_until=supported_until,
        )
        self.db_session.add(created_release)
        self.db_session.commit()
        return created_release

    def gen_certificate(
        self, machine: models.Machine, release: models.Release, name: str | None = None
    ) -> models.Certificate:
        certificate = models.Certificate(
            machine=machine,
            created_at=datetime.now(),
            release=release,
            name=name
            or f"Certificate for {machine.canonical_id} with {release.codename}",
            completed=datetime.now() + timedelta(days=10),
        )
        self.db_session.add(certificate)
        self.db_session.commit()
        return certificate

    def gen_kernel(
        self,
        name: str = "Linux",
        version: str = "5.4.0-42-generic",
        signature: str = "0000000",
    ) -> models.Kernel:
        kernel = models.Kernel(name=name, version=version, signature=signature)
        self.db_session.add(kernel)
        self.db_session.commit()
        return kernel

    def gen_bios(
        self,
        vendor: models.Vendor,
        release_date: date = datetime.now() - timedelta(days=365),
        revision: str = "A01",
        version: str = "1.0.2",
    ) -> models.Bios:
        bios = models.Bios(
            release_date=release_date,
            revision=revision,
            vendor=vendor,
            version=version,
        )
        self.db_session.add(bios)
        self.db_session.commit()
        return bios

    def gen_report(
        self,
        certificate: models.Certificate,
        kernel: models.Kernel,
        bios: models.Bios | None = None,
        architecture: str = "amd64",
    ) -> models.Report:
        report = models.Report(
            architecture=architecture, kernel=kernel, bios=bios, certificate=certificate
        )
        self.db_session.add(report)
        self.db_session.commit()
        return report

    def gen_device(
        self,
        vendor: models.Vendor,
        identifier: str,
        name: str,
        subproduct_name: str = "",
        device_type: str = "",
        bus: models.BusType = models.BusType.pci,
        version: str = "1.0",
        subsystem: str = "",
        category: models.DeviceCategory = models.DeviceCategory.OTHER,
        codename: str = "",
        reports: list[models.Report] | None = None,
    ) -> models.Device:
        device = models.Device(
            vendor=vendor,
            identifier=identifier,
            name=name,
            subproduct_name=subproduct_name,
            device_type=device_type,
            bus=bus,
            version=version,
            subsystem=subsystem,
            category=category,
            codename=codename,
        )

        if reports:
            device.reports.extend(reports)

        self.db_session.add(device)
        self.db_session.commit()
        return device

    def gen_cpuid_object(self, id_pattern: str, codename: str):
        obj = models.CpuId(id_pattern=id_pattern, codename=codename)
        self.db_session.add(obj)
        self.db_session.commit()
        return obj

    def gen_board(
        self,
        vendor: models.Vendor,
        identifier: str = "dmi:0001",
        name: str = "Test Board",
        version: str = "v1.0",
        reports: list[models.Report] | None = None,
    ) -> models.Device:
        return self.gen_device(
            vendor=vendor,
            identifier=identifier,
            name=name,
            version=version,
            bus=BusType.dmi,
            category=DeviceCategory.BOARD,
            reports=reports,
        )

    def gen_processor(
        self,
        vendor: models.Vendor,
        identifier: str = "dmi:1111",
        name: str = "Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
        version: str = "1.0",
        codename: str = "Raptor Lake",
        reports: list[models.Report] | None = None,
    ) -> models.Device:
        return self.gen_device(
            vendor=vendor,
            identifier=identifier,
            name=name,
            version=version,
            bus=BusType.dmi,
            category=DeviceCategory.PROCESSOR,
            codename=codename,
            reports=reports,
        )


class CertificationStatusTestHelper:
    @staticmethod
    def create_default_request(
        vendor_name: str = "Dell",
        board_name: str = "Test Board",
        board_version: str = "v1.0",
        bios_vendor: str | None = None,
        bios_version: str | None = None,
        bios_revision: str | None = None,
        bios_release_date: str | None = None,
        bios_firmware_revision: str | None = None,
        os_version: str = "24.04",
        os_codename: str = "noble",
        kernel_version: str = "5.7.1-generic",
        processor_vendor: str = "Intel Corp.",
        processor_id: list[int] | None = None,
        processor_version: str = "Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz",
    ) -> dict:
        request = {
            "vendor": vendor_name,
            "model": "Test Model",
            "architecture": "amd64",
            "board": {
                "manufacturer": vendor_name,
                "product_name": board_name,
                "version": board_version,
            },
            "os": {
                "distributor": "Ubuntu",
                "version": os_version,
                "codename": os_codename,
                "kernel": {"name": None, "version": kernel_version, "signature": None},
            },
            "processor": {
                "identifier": processor_id,
                "frequency": 2000,
                "manufacturer": processor_vendor,
                "version": processor_version,
            },
        }

        if bios_vendor:
            request["bios"] = {
                "vendor": bios_vendor,
                "version": bios_version,
                "revision": bios_revision,
                "release_date": bios_release_date,
                "firmware_revision": bios_firmware_revision,
            }

        return request

    @staticmethod
    def assert_not_seen_response(response) -> None:
        assert response.status_code == 200
        assert response.json() == {"status": "Not Seen"}

    @staticmethod
    def assert_certified_response(
        response,
        board: models.Device,
        bios: models.Bios | None,
        release: models.Release,
        kernel: models.Kernel,
    ) -> None:
        assert response.status_code == 200
        assert response.json() == {
            "status": "Certified",
            "architecture": "amd64",
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": (
                CertificationStatusTestHelper._create_bios_response(bios)
                if bios
                else None
            ),
            "chassis": None,
            "available_releases": [
                {
                    "distributor": "Ubuntu",
                    "version": release.release,
                    "codename": release.codename,
                    "kernel": {
                        "name": kernel.name,
                        "version": kernel.version,
                        "signature": kernel.signature,
                        "loaded_modules": [],
                    },
                }
            ],
        }

    @staticmethod
    def assert_certified_image_exists_response(
        response,
        board: models.Device,
        bios: models.Bios | None,
        release: models.Release,
        kernel: models.Kernel,
    ) -> None:
        assert response.status_code == 200
        assert response.json() == {
            "status": "Certified Image Exists",
            "architecture": "amd64",
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": (
                CertificationStatusTestHelper._create_bios_response(bios)
                if bios
                else None
            ),
            "chassis": None,
            "available_releases": [
                {
                    "distributor": "Ubuntu",
                    "version": release.release,
                    "codename": release.codename,
                    "kernel": {
                        "name": kernel.name,
                        "version": kernel.version,
                        "signature": kernel.signature,
                        "loaded_modules": [],
                    },
                }
            ],
        }

    @staticmethod
    def assert_related_certified_system_exists_response(
        response,
        board: models.Device,
        bios: models.Bios | None,
        release: models.Release,
        kernel: models.Kernel,
    ) -> None:
        assert response.status_code == 200
        assert response.json() == {
            "status": "Related Certified System Exists",
            "architecture": "amd64",
            "board": {
                "manufacturer": board.vendor.name,
                "product_name": board.name,
                "version": board.version,
            },
            "bios": (
                CertificationStatusTestHelper._create_bios_response(bios)
                if bios
                else None
            ),
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
                        "name": kernel.name,
                        "version": kernel.version,
                        "signature": kernel.signature,
                        "loaded_modules": [],
                    },
                }
            ],
        }

    @staticmethod
    def _create_bios_response(bios: models.Bios) -> dict[str, str | None]:
        return {
            "vendor": bios.vendor.name,
            "version": bios.version,
            "revision": bios.revision,
            "release_date": bios.release_date.strftime("%Y-%m-%d"),
            "firmware_revision": bios.firmware_revision,
        }
