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
"""The module for working with C3 API"""

import requests
import logging

from sqlalchemy.orm import Session
from sqlalchemy.exc import IntegrityError

from hwapi.data_models import models
from hwapi.data_models.repository import get_or_create
from hwapi.external.c3 import response_models, urls


logger = logging.getLogger(__name__)


class C3Api:
    """Class to work with C3 RESTAPI"""

    def __init__(self, db: Session):
        self.db = db

    def load_certified_configurations(self):
        """
        Retrieve certified configurations from C3 and create corresponding models
        """
        logger.info(
            f"Importing certified configurations and machines from {urls.C3_URL}"
        )
        response = requests.get(
            urls.CERTIFIED_CONFIGURATIONS_URL + urls.LIMIT_OFFSET, timeout=120
        )
        response.raise_for_status()
        objects = response.json()["results"]
        for obj in objects:
            try:
                public_cert = response_models.PublicCertificate(**obj)
                vendor = get_or_create(self.db, models.Vendor, name=public_cert.vendor)
                platform = get_or_create(
                    self.db,
                    models.Platform,
                    name=public_cert.platform,
                    vendor_id=vendor.id,
                )
                configuration = get_or_create(
                    self.db,
                    models.Configuration,
                    name=public_cert.configuration,
                    platform_id=platform.id,
                )
                machine = get_or_create(
                    self.db,
                    models.Machine,
                    canonical_id=public_cert.canonical_id,
                    configuration_id=configuration.id,
                )
                kernel = None
                if public_cert.kernel_version:
                    kernel = get_or_create(
                        self.db,
                        models.Kernel,
                        version=public_cert.kernel_version,
                    )
                bios = None
                if public_cert.bios is not None:
                    bios_vendor = (
                        self.db.query(models.Vendor)
                        .filter(models.Vendor.name.ilike(public_cert.bios.vendor))
                        .first()
                    )
                    # Remove Inc/Inc. from vendor name to avoid duplicate vendors
                    if bios_vendor is None and (
                        public_cert.bios.vendor.endswith("Inc")
                        or public_cert.bios.vendor.endswith("Inc.")
                    ):
                        bios_vendor_name = (
                            public_cert.bios.vendor[
                                : public_cert.bios.vendor.rfind("Inc")
                            ]
                            .replace(",", "")
                            .strip()
                        )
                        bios_vendor = (
                            self.db.query(models.Vendor)
                            .filter(models.Vendor.name.ilike(bios_vendor_name))
                            .first()
                        )
                    if bios_vendor is None:
                        bios_vendor = get_or_create(
                            self.db, models.Vendor, name=public_cert.bios.vendor
                        )
                    bios = get_or_create(
                        self.db,
                        models.Bios,
                        firmware_revision=public_cert.firmware_revision,
                        version=(
                            public_cert.bios.version
                            if public_cert.bios.version
                            else public_cert.bios.name
                        ),
                        vendor=bios_vendor,
                    )
                release = get_or_create(
                    self.db,
                    models.Release,
                    codename=public_cert.release.codename,
                    release=public_cert.release.release,
                    release_date=public_cert.release.release_date,
                    i_version=public_cert.release.i_version,
                    supported_until=public_cert.release.supported_until,
                )
                certificate = get_or_create(
                    self.db,
                    models.Certificate,
                    name=public_cert.name,
                    completed=public_cert.completed,
                    created_at=public_cert.created_at,
                    machine=machine,
                    release=release,
                )
                get_or_create(
                    self.db,
                    models.Report,
                    architecture=public_cert.architecture,
                    kernel=kernel,
                    bios=bios,
                    certificate=certificate,
                )
            except IntegrityError:
                logging.error(
                    "Got an error while importing certificates", exc_info=True
                )
                continue
