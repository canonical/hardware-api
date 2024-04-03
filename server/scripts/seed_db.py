#!/usr/bin/env python3
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
"""Populate SQLite DB with dummy data"""


from datetime import datetime, timedelta
from sqlalchemy.orm import Session
from hwapi.data_models import models
from hwapi.data_models.setup import engine


def create_vendors(session: Session) -> list[models.Vendor]:
    vendors = ["Dell", "Lenovo"]
    created_vendors = []
    for vendor_name in vendors:
        vendor: models.Vendor = models.Vendor(name=vendor_name)
        session.add(vendor)
        created_vendors.append(vendor)
    session.commit()
    return created_vendors


def create_platforms(
    session: Session, vendors: list[models.Vendor]
) -> list[models.Platform]:
    if len(vendors) < 2:
        raise ValueError("Too few vendors. Specify at least 2")
    platforms = [("ChengMing 3980", vendors[0]), ("P3 Tower", vendors[1])]
    created_platforms = []
    for platform_name, vendor in platforms:
        platform = models.Platform(name=platform_name, vendor=vendor)
        session.add(platform)
        created_platforms.append(platform)
    session.commit()
    return created_platforms


def create_configurations(
    session: Session, platforms: list[models.Platform]
) -> list[models.Configuration]:
    if len(platforms) < 2:
        raise ValueError("Too few platforms. Specify at least 2")
    configurations = [("i3-9100", platforms[0]), ("i9-13900K", platforms[1])]
    created_configurations = []
    for configuration_name, platform in configurations:
        configuration = models.Configuration(name=configuration_name, platform=platform)
        session.add(configuration)
        created_configurations.append(configuration)
    session.commit()
    return created_configurations


def create_machines(
    session: Session, configurations: list[models.Configuration]
) -> list[models.Machine]:
    if len(configurations) < 2:
        raise ValueError("Too few configurations. Specify at least 2")
    machines = [
        ("201811-26620", configurations[0]),
        ("202308-31972", configurations[1]),
    ]
    created_machines = []
    for canonical_id, configuration in machines:
        machine = models.Machine(canonical_id=canonical_id, configuration=configuration)
        session.add(machine)
        created_machines.append(machine)
    session.commit()
    return created_machines


def create_releases(session: Session) -> list[models.Release]:
    releases = [
        {
            "codename": "focal",
            "release": "20.04",
            "release_date": datetime.now().date(),
            "supported_until": datetime.now().date() + timedelta(days=365),
        },
        {
            "codename": "jammy",
            "release": "22.04",
            "release_date": datetime.now().date() - timedelta(days=365),
            "supported_until": datetime.now().date() + timedelta(days=365 * 2),
        },
    ]
    created_releases = []
    for release_data in releases:
        release = models.Release(**release_data)
        session.add(release)
        created_releases.append(release)
    session.commit()
    return created_releases


def create_certificates(
    session: Session, machines: list[models.Machine], releases: list[models.Release]
) -> list[models.Certificate]:
    if len(machines) < 2:
        raise ValueError("Too few machines. Specify at least 2")
    if len(releases) < 2:
        raise ValueError("Too few releases. Specify at least 2")

    certificates = ((machines[0], releases[0]), (machines[1], releases[1]))
    created_certificates = []
    for machine, release in certificates:
        certificate = models.Certificate(
            machine=machine,
            created_at=datetime.now(),
            release=releases[0],
            name=f"Certificate for {machine.canonical_id} with {release.codename}",
            completed=datetime.now() + timedelta(days=10),
        )
        session.add(certificate)
        created_certificates.append(certificate)
    session.commit()
    return created_certificates


def create_reports(
    session: Session, vendor: models.Vendor, certificates: list[models.Certificate]
) -> list[models.Report]:
    kernel = models.Kernel(
        name="Linux", version="5.4.0-42-generic", signature="0000000"
    )
    bios = models.Bios(
        firmware_version="1.0.0",
        release_date=datetime.now() - timedelta(days=365),
        revision="A01",
        vendor=vendor,
        version="1.0.2",
    )
    session.add(kernel)
    session.add(bios)
    session.commit()

    created_reports = []
    for certificate in certificates:
        report = models.Report(
            created_at=datetime.now(), kernel=kernel, bios=bios, certificate=certificate
        )
        session.add(report)
        created_reports.append(report)
    session.commit()
    return created_reports


if __name__ == "__main__":
    session = Session(bind=engine)
    vendors = create_vendors(session)
    platforms = create_platforms(session, vendors)
    configurations = create_configurations(session, platforms)
    machines = create_machines(session, configurations)
    releases = create_releases(session)
    certificates = create_certificates(session, machines, releases)
    create_reports(session, vendors[1], certificates)
    print("Database initialized.")
