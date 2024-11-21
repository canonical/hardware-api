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
from typing import Callable, Type
from pydantic import BaseModel

from sqlite3 import IntegrityError as SQLite3IntegrityError
from sqlalchemy.orm import Session
from sqlalchemy.exc import IntegrityError

from hwapi.data_models import models, enums
from hwapi.data_models.repository import (
    get_or_create,
    get_vendor_by_name,
    get_machine_by_canonical_id,
    get_certificate_by_name,
)
from hwapi.external.c3 import response_models, urls
from hwapi.external.c3.helpers import progress_bar


logger = logging.getLogger(__name__)


class C3Client:
    """Class v2 API client"""

    def __init__(self, db: Session):
        self.db = db

    def load_hardware_data(self):
        """Orchestrator that calls the loaders in the correct order"""
        # Load CPU IDs
        logger.info("Importing CPU IDs and codenames from %s", urls.C3_URL)
        url = urls.CPU_IDS_URL
        self._import_cpu_ids(url)

        # Load certified configurations
        logger.info(
            "Importing certified configurations and machines from %s", urls.C3_URL
        )
        url = urls.PUBLIC_CERTIFICATES_URL + urls.get_limit_offset()
        self._import_from_c3(
            url,
            self._load_certified_configurations_from_response,
            response_models.PublicCertificate,
        )

        # Load devices
        LIMIT = 1000
        logger.info("Importing devices from %s", urls.C3_URL)
        url = urls.PUBLIC_DEVICES_URL + urls.get_limit_offset(LIMIT)
        self._import_from_c3(
            url,
            self._load_devices_from_response,
            response_models.PublicDeviceInstance,
        )

    def _import_cpu_ids(self, url: str):
        response = requests.get(url, timeout=60)
        response.raise_for_status()
        response_json = response.json()
        for codename, ids in response_json.items():
            for cpu_id in ids:
                get_or_create(
                    self.db, models.CpuId, id_pattern=cpu_id, codename=codename
                )
        self.db.commit()

    def _import_from_c3(self, url: str, loader: Callable, resp_model: Type[BaseModel]):
        """
        A general method to load some kind of data from the specified C3 endpoint

        :param url: C3 API endpoint (full URL)
        :param loader: the private method to use for loading data from response
        :param resp_model: pydantic model for response objects
        """
        next_url = url
        counter = 0
        previous_ratio = -1
        while next_url is not None:
            logging.debug(f"Retrieving {next_url}")
            response = requests.get(next_url, timeout=90)
            response.raise_for_status()
            response_json = response.json()
            if counter == 0:
                # since count is always the same, update total only the first time
                # we retrieve the data from C3
                total = response_json["count"]
            next_url = response_json["next"]
            objects = response_json["results"]
            for obj in objects:
                instance = resp_model(**obj)
                try:
                    loader(instance)
                    counter += 1
                    # Don't print progress bar with the same percentage, because
                    # it slows down the script
                    if 1000 * counter // total != previous_ratio:
                        progress_bar(counter, total)
                        previous_ratio = 1000 * counter // total
                    if total == counter:
                        print()
                except (IntegrityError, SQLite3IntegrityError):
                    logging.error(
                        "A DB error occurred while importing data from C3",
                        exc_info=True,
                    )
                    # Without this the sqlalchemy.exc.PendingRollbackError exception occurs
                    self.db.rollback()
                    continue
                except Exception as exc:
                    logging.error(
                        "An error occured while importing the data from C3: %s",
                        str(exc),
                    )
                    continue

    def _load_certified_configurations_from_response(
        self, response: response_models.PublicCertificate
    ):
        vendor, _ = get_or_create(self.db, models.Vendor, name=response.vendor)
        platform, _ = get_or_create(
            self.db,
            models.Platform,
            name=response.platform,
            vendor_id=vendor.id,
        )
        configuration, _ = get_or_create(
            self.db,
            models.Configuration,
            name=response.configuration,
            platform_id=platform.id,
        )
        machine, _ = get_or_create(
            self.db,
            models.Machine,
            canonical_id=response.canonical_id,
            configuration_id=configuration.id,
        )
        logger.debug(
            "Vendor: %s\nConfiguration: %s\nMachine: %s\n",
            vendor.name,
            configuration.name,
            machine.canonical_id,
        )
        kernel = None
        if response.kernel_version:
            kernel, _ = get_or_create(
                self.db,
                models.Kernel,
                version=response.kernel_version,
            )
        bios = None
        if response.bios is not None:
            bios_vendor = get_vendor_by_name(self.db, response.bios.vendor)
            if bios_vendor is None:
                bios_vendor, _ = get_or_create(
                    self.db, models.Vendor, name=response.bios.vendor
                )
            bios, _ = get_or_create(
                self.db,
                models.Bios,
                firmware_revision=response.firmware_revision,
                version=(
                    response.bios.version
                    if response.bios.version
                    else response.bios.name
                ),
                vendor=bios_vendor,
            )
        release, _ = get_or_create(
            self.db,
            models.Release,
            codename=response.release.codename,
            release=response.release.release.replace("LTS", "").strip(),
            release_date=response.release.release_date,
            i_version=response.release.i_version,
            supported_until=response.release.supported_until,
        )
        certificate, _ = get_or_create(
            self.db,
            models.Certificate,
            name=response.name,
            completed=response.completed,
            created_at=response.created_at,
            machine=machine,
            release=release,
        )
        get_or_create(
            self.db,
            models.Report,
            architecture=response.architecture or "",
            kernel=kernel,
            bios=bios,
            certificate=certificate,
        )

    def _load_devices_from_response(
        self, device_instance: response_models.PublicDeviceInstance
    ):
        machine = get_machine_by_canonical_id(
            self.db, device_instance.machine_canonical_id
        )
        if machine is None:
            raise ValueError(
                f"Machine with canonical ID {device_instance.machine_canonical_id}"
                " does not exist"
            )
        certificate = get_certificate_by_name(
            self.db, machine.id, device_instance.certificate_name
        )
        if certificate is None:
            raise ValueError(
                f"Certificate with name {device_instance.certificate_name} does not"
                " exist"
            )

        device_data = device_instance.device
        vendor, _ = get_or_create(self.db, models.Vendor, name=device_data.vendor)
        device, created = get_or_create(
            self.db,
            models.Device,
            defaults={
                "subproduct_name": (
                    device_data.subproduct_name if device_data.subproduct_name else ""
                ),
                "device_type": (
                    device_data.device_type if device_data.device_type else ""
                ),
                "codename": device_data.codename,
                "identifier": device_data.identifier,
            },
            name=device_data.name if device_data.name else "",
            version=device_data.version if device_data.version else "",
            vendor_id=vendor.id,
            subsystem=device_data.subsystem if device_data.subsystem else "",
            bus=device_data.bus.value,
            category=(
                device_data.category
                if device_data.category is not None
                else enums.DeviceCategory.OTHER
            ).value,
        )
        logger.debug(
            "Device: %s, %s. Created: %r",
            device_data.name,
            device_data.identifier,
            created,
        )

        if (
            device.category == enums.DeviceCategory.PROCESSOR
            and device_instance.cpu_codename
        ):
            # If there is already device.codename filled, don't overwrite it with Unknown value
            if not (device_instance.cpu_codename == "Unknown" and device.codename):
                device.codename = device_instance.cpu_codename

        report, created = get_or_create(
            self.db,
            models.Report,
            certificate_id=certificate.id,
            defaults={"architecture": ""},
        )
        if created or device not in report.devices:
            report.devices.append(device)
        self.db.commit()
