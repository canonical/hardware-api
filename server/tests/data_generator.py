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
        codename: str = "jammy",
        release: str = "22.04",
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
        self, machine: models.Machine, release: models.Release
    ) -> models.Certificate:
        certificate = models.Certificate(
            machine=machine,
            created_at=datetime.now(),
            release=release,
            name=f"Certificate for {machine.canonical_id} with {release.codename}",
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
        self, certificate: models.Certificate, kernel: models.Kernel, bios: models.Bios
    ) -> models.Report:
        report = models.Report(
            architecture="amd64", kernel=kernel, bios=bios, certificate=certificate
        )
        self.db_session.add(report)
        self.db_session.commit()
        return report
