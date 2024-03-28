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


vendors = ["Dell", "Lenovo"]
platforms = [("ChengMing 3980", vendors[0]), ("P3 Tower", vendors[1])]
configurations = [("i3-9100", platforms[0][0]), ("i9-13900K", platforms[1][0])]
machines = [
    ("201811-26620", configurations[0][0]),
    ("202308-31972", configurations[1][0]),
]

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


def create_certificates(session):
    models.Base.metadata.bind = engine
    models.Base.metadata.create_all(engine)
    machine1 = (
        session.query(models.Machine).filter_by(canonical_id=machines[0][0]).first()
    )
    machine2 = (
        session.query(models.Machine).filter_by(canonical_id=machines[1][0]).first()
    )

    release_focal = (
        session.query(models.Release)
        .filter_by(codename=releases[0]["codename"])
        .first()
    )
    release_jammy = (
        session.query(models.Release)
        .filter_by(codename=releases[1]["codename"])
        .first()
    )

    if machine1 and machine2 and release_focal and release_jammy:
        certificate1 = models.Certificate(
            hardware=machine1,
            created_at=datetime.now(),
            release=release_focal,
            name="Certificate for Machine 1 with Focal",
            completed=datetime.now() + timedelta(days=10),
        )

        certificate2 = models.Certificate(
            hardware=machine2,
            created_at=datetime.now(),
            release=release_jammy,
            name="Certificate for Machine 2 with Jammy",
            completed=datetime.now() + timedelta(days=10),
        )

        session.add(certificate1)
        session.add(certificate2)
        session.commit()


if __name__ == "__main__":
    session = Session(bind=engine)

    for vendor_name in vendors:
        vendor = models.Vendor(name=vendor_name)
        session.add(vendor)
    session.commit()

    for platform_name, vendor_name in platforms:
        vendor = (
            session.query(models.Vendor)
            .filter(models.Vendor.name == vendor_name)
            .first()
        )
        platform = models.Platform(name=platform_name, vendor=vendor)
        session.add(platform)
    session.commit()

    for configuration_name, platform_name in configurations:
        platform = (
            session.query(models.Platform)
            .filter(models.Platform.name == platform_name)
            .first()
        )
        configuration = models.Configuration(name=configuration_name, platform=platform)
        session.add(configuration)
    session.commit()

    for canonical_id, configuration_name in machines:
        configuration = (
            session.query(models.Configuration)
            .filter(models.Configuration.name == configuration_name)
            .first()
        )
        machine = models.Machine(canonical_id=canonical_id, configuration=configuration)
        session.add(machine)
    session.commit()

    for release_data in releases:
        release = models.Release(**release_data)
        session.add(release)
    session.commit()

    create_certificates(session)

    print("Database initialized.")
